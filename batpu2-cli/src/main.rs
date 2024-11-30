
use std::fs;
use std::process::exit;
use anyhow::{bail, Context, ensure, Result};

mod arguments;

use arguments::{Arguments, Command};

fn main() -> Result<()> {
	let args: Vec<String> = std::env::args().collect();
	let program = &args[0];
	let mut arguments = Arguments::new();
	
	if let Err(err) = arguments.parse(&args[1..]) {
		eprintln!("{err}");
		arguments.print_usage(program, true);
		exit(-1);
	}
	
	match arguments.command {
		Command::Help => arguments.print_usage(program, false),
		Command::Run(path) => {
			let file = fs::read_to_string(&path)
			                     .with_context(|| format!("Failed to open: \"{path}\""))?;
			let code = parse_mc(&file)?;
			
			print!("{code:?}");
			
			run(&code)?;
		}
	}
	
	Ok(())
}

fn parse_line(pos: usize, line: &str) -> Result<u16> {
	if let Some(char) = line.chars().find(|&char| char != '0' && char != '1') {
		bail!("Unexpected character '{char}' at line {}", pos + 1);
	}
	
	ensure!(line.len() == 16, "Unexpected length {} of line {}, expected 16", line.len(), pos + 1);
	
	u16::from_str_radix(line, 2)
	    .with_context(|| format!("Failed to parse line {}", pos + 1))
}

fn parse_mc(code: &str) -> Result<Vec<u16>> {
	code.lines()
	    .enumerate()
	    .map(|(pos, line)| parse_line(pos, line))
	    .collect()
}

fn run(code: &[u16]) -> Result<()> {
	Ok(())
}
