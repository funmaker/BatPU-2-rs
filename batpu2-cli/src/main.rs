#![feature(try_blocks)]
#![feature(array_chunks)]
#![feature(hash_extract_if)]

use std::process::exit;

mod arguments;
mod run;
mod asm;

use arguments::{Arguments, Command};

fn main() {
	let args: Vec<String> = std::env::args().collect();
	let program = &args[0];
	let mut arguments = Arguments::new();
	
	if let Err(err) = arguments.parse(&args[1..]) {
		eprintln!("{err}");
		arguments.print_usage(program, true);
		exit(-1);
	}
	
	let result = match &arguments.command {
		Command::Help => Ok(arguments.print_usage(program, false)),
		Command::Run{ filename } => run::cmd(filename, &arguments),
		Command::Asm{ input, output } => asm::cmd(input, output),
	};
	
	if let Err(err) = result {
		eprintln!("{err:?}");
		exit(-1);
	}
}
