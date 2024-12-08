use crate::asm::AsmError;
use crate::asm::ast::{Line, Token};

pub fn parse_lines(code: &str) -> impl Iterator<Item=Result<Line, AsmError>> {
	code.lines()
	    .enumerate()
	    .filter(|(_, line)| !line.trim().is_empty())
	    .map(|(line_number, line)| parse_line(line_number + 1, line))
}

pub fn parse_line(line_number: usize, line: &str) -> Result<Line, AsmError> {
	let (line, comment) = line.find([';', '/', '#'])
	                          .map(|pos| (&line[..pos], Some(Token::new(pos, &line[pos..]))))
	                          .unwrap_or((line, None));
	
	let mut tokens = tokenize(line).peekable();
	
	let label = tokens.next_if(|token| token.starts_with('.'));
	let mnemonic = tokens.next();
	let args = (&mut tokens).take(3).collect();
	
	if let Some(token) = tokens.next() {
		return Err(AsmError::TooManyTokens { line_number, token } )
	}
	
	Ok(Line {
		line_number,
		label,
		mnemonic,
		args,
		comment,
	})
}

fn tokenize(mut line: &str) -> impl Iterator<Item=Token> {
	let original = line;
	
	std::iter::from_fn(move || {
		line = line.trim_start();
		
		if line.is_empty() {
			None
		} else if let Some((token, rest)) = split_whitespace_quote(line) {
			line = rest;
			Some(Token::new(original.len() - line.len() - token.len(), token))
		} else {
			let (token, rest) = line.split_once(char::is_whitespace).unwrap_or((line, ""));
			line = rest;
			Some(Token::new(original.len() - line.len() - token.len(), token))
		}
	})
}

fn split_whitespace_quote(line: &str) -> Option<(&str, &str)> {
	for (pos, c) in line.chars().enumerate() {
		// ( ͡° ͜ʖ ͡°)
		return if pos == 0 && !matches!(c, '\'' | '"') {
			None
		} else if pos > 1 && matches!(c, '\'' | '"') {
			Some((&line[..pos+1], &line[pos+1..]))
		} else if pos > 0 && !c.is_whitespace() {
			None
		} else {
			continue
		}
	}
	
	None
}
