use std::ops::Deref;
use anyhow::{bail, Result};
use getopts::Options;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Command {
	Help,
	Run{ filename: String },
	Asm{ input: String, output: String },
}

pub struct Arguments {
	opts: Options,
	pub command: Command,
	pub help: bool,
	pub tickrate: f32,
}

impl Arguments {
	pub fn new() -> Self {
		let mut opts = Options::new();
		
		opts.optflag("h", "help", "print this message");
		opts.optopt("s", "speed", "number of instructions executed per second", "100.0");
		
		Self {
			opts,
			command: Command::Help,
			help: false,
			tickrate: 100.0,
		}
	}
	
	pub fn parse(&mut self, args: &[String]) -> Result<()> {
		let matches = self.opts.parse(args)?;
		
		self.help = matches.opt_get("help")?.unwrap_or(self.help);
		self.tickrate = matches.opt_get("speed")?.unwrap_or(self.tickrate);
		
		if !self.help {
			self.command = match matches.free.first().map(Deref::deref) {
				None => bail!("Missing command"),
				Some("help") => Command::Help,
				Some("run") => {
					let [_, filename] = expect_free_args(&matches.free, ["", "filename"])?;
					
					Command::Run{ filename: filename.clone() }
				}
				Some("asm") => {
					let [_, input, output] = expect_free_args(&matches.free, ["", "input", "output"])?;
					
					Command::Asm{ input: input.clone(), output: output.clone() }
				}
				Some(cmd) => bail!("Unknown command: {cmd}"),
			}
		}
		
		Ok(())
	}
	
	pub fn print_usage(&self, program: &str, error: bool) {
		let brief = format!("\
Usage: {program} <Command> [options]

Commands:
    run <filename>        execute a file on the emulator
    asm <input> <output>  compile .asm file to .mc\
");
		let usage = self.opts.usage(&brief);
		
		if error {
			eprintln!("\n{usage}");
		} else {
			println!("{usage}");
		}
	}
}

fn expect_free_args<'a, const N: usize>(args: &'a [String], names: [&str; N]) -> Result<&'a [String; N]> {
	if args.len() < names.len() {
		bail!("Missing {}", names[args.len()])
	} else if args.len() > names.len() {
		bail!("Unexpected command argument \"{}\"", args[names.len()])
	} else {
		Ok(args.try_into()?)
	}
}
