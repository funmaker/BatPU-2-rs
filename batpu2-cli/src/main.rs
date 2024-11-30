#![feature(try_blocks)]

use std::fs;
use std::io::Write;
use std::marker::PhantomData;
use std::process::exit;
use std::time::{Duration, Instant};
use anyhow::{bail, Context, ensure, Result};

mod arguments;

use arguments::{Arguments, Command};
use batpu2::BatPU2;
use batpu2::embedded::EmbeddedIO;

type VM<'a> = BatPU2<&'a [u16], EmbeddedIO>;

fn main() -> Result<()> {
	let args: Vec<String> = std::env::args().collect();
	let program = &args[0];
	let mut arguments = Arguments::new();
	
	if let Err(err) = arguments.parse(&args[1..]) {
		eprintln!("{err}");
		arguments.print_usage(program, true);
		exit(-1);
	}
	
	let result: Result<_> = try {
		match &arguments.command {
			Command::Help => arguments.print_usage(program, false),
			Command::Run(path) => {
				let file = fs::read_to_string(path)
				              .with_context(|| format!("Failed to open: \"{path}\""))?;
				let code = parse_mc(&file)?;
				
				run(&code, &arguments)?;
			}
		}
	};
	
	if result.is_err() {
		recover_term();
	}
	
	result
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

struct Watch<T, R, W> {
	value: Option<R>,
	func: W,
	phantom_data: PhantomData<T>,
}

impl<T, R, W> Watch<T, R, W>
where W: Fn(&T) -> R,
      R: PartialEq {
	fn new(func: W) -> Self {
		Self {
			value: None,
			func,
			phantom_data: PhantomData::default(),
		}
	}
	
	fn changed(&mut self, arg: &T) -> Option<&R> {
		let old = std::mem::replace(&mut self.value, Some((self.func)(arg)));
		
		if self.value != old {
			self.value.as_ref()
		} else {
			None
		}
	}
}

fn run(code: &[u16], arguments: &Arguments) -> Result<()> {
	use crossterm::terminal::ClearType;
	use crossterm::*;
	use std::io;
	
	let mut vm = BatPU2::new(code);
	
	terminal::enable_raw_mode()?;
	
	execute!(io::stdout(),
	         terminal::EnterAlternateScreen,
	         terminal::Clear(ClearType::All),
	         cursor::Hide,
	         cursor::MoveTo(5, 5))?;
	
	let mut last_sec = Instant::now();
	let mut steps = 0;
	
	let mut screen = Watch::new(|vm: &VM| vm.io.screen.output);
	let mut char_display = Watch::new(|vm: &VM| vm.io.char_display.output);
	let mut number_display = Watch::new(|vm: &VM| vm.io.number_display);
	
	loop {
		let steps_target = (last_sec.elapsed().as_secs_f32() * arguments.tickrate) as usize;
		if steps_target > steps {
			steps += vm.step_multiple((steps_target - steps).min(arguments.tickrate.max(10.0) as usize));
		}
		
		if last_sec.elapsed().as_secs_f32() > 10.0 {
			last_sec = Instant::now();
			steps = 0;
			break;
		}
		
		let mut queued = false;
		
		if let Some(screen) = screen.changed(&vm) {
			// queue!(io::stdout(), style::Print("screen!"))?;
			queued = true;
		}
		
		if let Some(char_display) = char_display.changed(&vm) {
			let str: String = char_display.iter().map(|x| x.to_char().unwrap_or('#')).collect();
			queue!(io::stdout(), cursor::MoveTo(1, 1), style::Print(str))?;
			queued = true;
		}
		
		if let Some(number_display) = number_display.changed(&vm) {
			queue!(io::stdout(), cursor::MoveTo(20, 1), style::Print(format!("{number_display:<4}")))?;
			queued = true;
		}
		
		if queued {
			io::stdout().flush()?;
		}
		
		std::thread::sleep(Duration::from_secs_f32(0.01));
	}
	
	execute!(io::stdout(),
	         cursor::Show,
	         terminal::LeaveAlternateScreen)?;
	
	Ok(())
}

fn recover_term() {
	use crossterm::*;
	
	macro_rules! print_err {
	    ($e: expr) => {
		    if let Err(err) = $e { eprintln!("{err}") }
	    };
	}
	
	print_err!(execute!(
		std::io::stdout(),
		cursor::Show,
		terminal::LeaveAlternateScreen,
	));
	
	if terminal::is_raw_mode_enabled().unwrap_or(true) {
		print_err!(terminal::disable_raw_mode());
	}
}
