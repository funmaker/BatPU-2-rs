use thiserror::Error;
use std::num::ParseIntError;

use crate::isa::Instruction;

fn from_asm(code: &str) -> Vec<Instruction> {
	unimplemented!()
}

fn from_mc(code: &str) -> Result<Vec<Instruction>, FromMcError> {
	code.lines()
		.enumerate()
		.filter(|(_, line)| !line.is_empty())
	    .map(|(line_number, line)|
		    u16::from_str_radix(line, 2)
		        .map(Into::into)
		        .map_err(|source| FromMcError { line_number, line: line.to_owned(), source }))
	    .collect()
}

#[derive(Error, Debug)]
#[error("{line_number}: Cannot parse \"{line}\"")]
struct FromMcError {
	line_number: usize,
	line: String,
	#[source] source: ParseIntError,
}

fn into_mc(instructions: &[Instruction]) -> String {
	let mut output = String::with_capacity(instructions.len() * 17);
	
	for instruction in instructions.iter() {
		output.push_str(&format!("{:b}\n", instruction.as_u16()))
	}
	
	output
}
