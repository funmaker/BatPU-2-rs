use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct File {
	file_name: Option<String>,
	lines: Vec<Line>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Line {
	line_number: usize,
	label: Option<String>,
	instruction: Option<Instruction>,
	comment: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Instruction {
	mnemonic: String,
	operands: Vec<Operand>
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Operand {
	Number(i32),
	Character(String),
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
			           .collect::<Result<_, _>>()?,
		})
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
			     .transpose()?;
		
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
		if text.chars().all(char::is_numeric) {
			Ok(Operand::Number(text.parse().or(Err(format!("Cannot parse {text} as a number!")))?))
		}else if text.len() == 3 && text.starts_with(|c: char| { c == '"' || c == '\'' }) && text.ends_with(text.chars().next().unwrap()) {
			Ok(Operand::Character(String::from(text.chars().skip(1).next().unwrap())))
		}else if text.len() > 0 {
			Ok(Operand::Symbol(String::from(text)))
		}else{
			Err("Empty operand!".into())
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
