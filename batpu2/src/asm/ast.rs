use std::fmt::{Display, Formatter};
use crate::Char;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct File {
	pub file_name: Option<String>,
	pub lines: Vec<Line>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Line {
	pub line_number: usize,
	pub label: Option<String>,
	pub instruction: Option<Instruction>,
	pub comment: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Instruction {
	pub mnemonic: String,
	pub operands: Vec<Operand>
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ResolvedInstruction {
	pub mnemonic: String,
	pub operands: Vec<u16>
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Operand {
	Number(i32),
	Character(Char),
	Symbol(String),
}

impl File {
	pub fn from_text(code: &str, file_name: Option<&str>) -> Result<File, String> {
		Ok(File {
			file_name: file_name.map(String::from),
			lines: code.split('\n')
			           .enumerate()
			           .filter(|(_, line)| !line.is_empty() && !line.chars().all(char::is_whitespace))
			           .map(|(line, code)| Line::from_text(code, line))
			           .collect::<Result<_, _>>()
			           .map_err(|err| if let Some(f_name) = file_name { format!("{}:{}", f_name, err) } else { err })?,
		})
	}
	
	pub fn error_prefix(&self, line_number: usize) -> String {
		if let Some(f_name) = &self.file_name {
			format!("{}:{}: ", f_name, line_number)
		} else {
			format!("{}: ", line_number)
		}
	}
}

impl Line {
	pub fn from_text(line_code: &str, line_number: usize) -> Result<Line, String> {
		let (code, comment) =
			line_code.split_once(|c| c == '/' || c == ';' || c == '#')
			         .map(|(code, comment)| (code, Some(comment.to_owned())))
			         .unwrap_or((line_code, None));
		
		let mut words =
			code.trim()
			    .split_whitespace()
			    .filter(|w| !w.is_empty())
			    .peekable();
		
		let label =
			words.next_if(|word| word.starts_with('.'))
			     .map(|word| &word[1..]);
		
		let instruction =
			words.peek()
			     .is_some()
			     .then(|| Instruction::from_text(words))
			     .transpose()
				 .map_err(|err| format!("{}: {}", line_number, err))?;
		
		Ok(Line {
			line_number,
			comment: comment.map(Into::into),
			label: label.map(Into::into),
			instruction,
		})
	}
}

impl Instruction {
	pub fn from_text<'a>(words: impl IntoIterator<Item = &'a str>) -> Result<Instruction, String> {
		let mut words = words.into_iter().peekable();

		let Some(mnemonic) = words.next() else { return Err("Missing mnemonic".into()) };
		
		let operands =
			std::iter::from_fn(move ||
				words.next()
				     .map(|current| (current == "'" || current == "\"")
					     .then(|| words.next_if(|&word| word == current)
					                   .map(|_| { "' '" }))
					     .flatten()
					     .unwrap_or(current))
			).map(Operand::from_text)
			 .collect::<Result<Vec<Operand>, String>>()?;

		Ok(Instruction {
			mnemonic: mnemonic.to_string(),
			operands,
		})
	}
}

impl Operand {
	pub fn from_text(text: &str) -> Result<Operand, String> {
		let first_char = text.chars().next();
		if first_char == None {
			return Err("Empty operand!".into())
		}
		let first_char = first_char.unwrap();
		
		if (first_char == '-' || first_char.is_numeric()) && text.chars().skip(1).all(char::is_numeric) {
			Ok(Operand::Number(text.parse().or(Err(format!("Cannot parse {text} as a number!")))?))
		}else if text.len() == 3 && text.starts_with(|c: char| { c == '"' || c == '\'' }) && text.ends_with(text.chars().next().unwrap()) {
			Ok(Operand::Character(text.chars().skip(1).next().unwrap().try_into()?))
		}else {
			Ok(Operand::Symbol(String::from(text)))
		}
	}
}

impl Display for File {
	fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
		for line in &self.lines {
			writeln!(fmt, "{}", line)?;
		}
		Ok(())
	}
}

impl Display for Line {
	fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
		let mut sep = "";
		if let Some(label) = &self.label {
			write!(fmt, ".{}", label)?;
			sep = " ";
		}
		if let Some(instruction) = &self.instruction {
			write!(fmt, "{}{}", sep, instruction)?;
			sep = " ";
		}
		if let Some(comment) = &self.comment {
			write!(fmt, "{};{}", sep, comment)?;
		}
		Ok(())
	}
}

impl Display for Instruction {
	fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
		fmt.write_str(&self.mnemonic)?;
		for operand in &self.operands {
			write!(fmt, " {}", operand)?;
		}
		Ok(())
	}
}

impl Display for Operand {
	fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
		match self {
			Operand::Number(num) =>     { write!(fmt, "{}",   num) }
			Operand::Character(str) => { write!(fmt, "'{}'", str) }
			Operand::Symbol(str) =>   { write!(fmt, "{}",   str) }
		}
	}
}
