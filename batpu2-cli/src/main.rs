#![feature(try_blocks)]
#![feature(array_chunks)]
#![feature(hash_extract_if)]

use std::fs;
use std::io::Write;
use std::marker::PhantomData;
use std::process::exit;
use std::time::{Duration, Instant};
use anyhow::{bail, Context, ensure, Result};

mod arguments;

use arguments::{Arguments, Command};
use batpu2::BatPU2;
use batpu2::vm::io::embedded::{EmbeddedIO, Controller};

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
	
	fn reset(&mut self) {
		self.value = None;
	}
}

fn run(code: &[u16], arguments: &Arguments) -> Result<()> {
	use crossterm::terminal::ClearType;
	use crossterm::style::Color;
	use crossterm::event::{ Event, KeyEvent, KeyCode, KeyEventKind, KeyModifiers };
	use crossterm::*;
	use std::io;
	
	let mut vm = BatPU2::new(code);
	
	terminal::enable_raw_mode()?;
	
	execute!(io::stdout(),
	         terminal::EnterAlternateScreen,
	         terminal::Clear(ClearType::Purge),
	         cursor::Hide,
	         style::SetBackgroundColor(Color::Rgb { r: 0x2d, g: 0x17, b: 0x10 }),
	         style::SetForegroundColor(Color::Rgb { r: 0xf0, g: 0xd4, b: 0xac }),
	         cursor::MoveTo(5, 5))?;
	
	let mut last_sec = Instant::now();
	let mut steps = 0;
	
	let mut screen = Watch::new(|vm: &VM| vm.io.screen.output);
	let mut char_display = Watch::new(|vm: &VM| vm.io.char_display.output);
	let mut number_display = Watch::new(|vm: &VM| vm.io.number_display);
	let mut buttons = Watch::new(|vm: &VM| vm.io.controller.state);
	
	let mut btn_times = std::collections::HashMap::<u8, Instant>::new();
	
	macro_rules! binds {
		($event: expr, $vm: expr; $( $code: pat => $bit: expr ),* $(,)?) => {
			match $event {
				$(
					Event::Key(KeyEvent {
						code: $code,
						kind: KeyEventKind::Press, ..
					}) => {
						btn_times.insert($bit, Instant::now());
						$vm.io.controller.set_button($bit)
					},
					// Event::Key(KeyEvent {
					// 	code: $code,
					// 	kind: KeyEventKind::Release, ..
					// }) => {
					// 	$vm.io.controller.clear_button($bit)
					// },
				)*
				_ => {},
			}
		};
	}
	
	loop {
		if event::poll(Duration::ZERO)? {
			match event::read()? {
				Event::Key(KeyEvent {
					           code: KeyCode::Char('c'),
					           modifiers: KeyModifiers::CONTROL,
					           kind: KeyEventKind::Press,
					           ..
				           }) => {
					break;
				},
				Event::Resize(_, _) => {
					screen.reset();
					char_display.reset();
					number_display.reset();
					
					queue!(io::stdout(), terminal::Clear(ClearType::Purge))?;
				},
				event => {
					binds!(event, vm;
						KeyCode::Left => Controller::B_LEFT,
						KeyCode::Down => Controller::B_DOWN,
						KeyCode::Right => Controller::B_RIGHT,
						KeyCode::Up => Controller::B_UP,
						KeyCode::Char('z') => Controller::B_B,
						KeyCode::Char('x') => Controller::B_A,
						KeyCode::Esc => Controller::B_SELECT,
						KeyCode::Enter => Controller::B_START,
						KeyCode::Char('a') => Controller::B_LEFT,
						KeyCode::Char('s') => Controller::B_DOWN,
						KeyCode::Char('d') => Controller::B_RIGHT,
						KeyCode::Char('w') => Controller::B_UP,
						KeyCode::Char('k') => Controller::B_B,
						KeyCode::Char('j') => Controller::B_A,
						KeyCode::Char('t') => Controller::B_SELECT,
						KeyCode::Char('y') => Controller::B_START,
					)
				},
			}
		}
		
		for (bit, _) in btn_times.extract_if(|_, time| time.elapsed().as_secs_f32() > 0.5) {
			vm.io.controller.clear_button(bit);
		}
		
		let steps_target = (last_sec.elapsed().as_secs_f32() * arguments.tickrate) as usize;
		if steps_target > steps {
			steps += vm.step_multiple((steps_target - steps).min(arguments.tickrate.max(10.0) as usize));
		}
		
		if last_sec.elapsed().as_secs_f32() > 1.0 {
			last_sec = Instant::now();
			steps = 0;
		}
		
		let mut queued = false;
		
		if let Some(char_display) = char_display.changed(&vm) {
			let str: String = char_display.iter().map(|x| x.to_char().unwrap_or('#')).collect();
			queue!(io::stdout(), cursor::MoveTo(1, 1), style::Print(str))?;
			queued = true;
		}
		
		if let Some(number_display) = number_display.changed(&vm) {
			queue!(io::stdout(), cursor::MoveTo(29, 1), style::Print(format!("{number_display:<4}")))?;
			queued = true;
		}
		
		if let Some(screen) = screen.changed(&vm) {
			for (y, [lower, upper]) in screen.array_chunks().rev().enumerate() {
				let mut line = String::with_capacity(32 * 3); // 3 bytes per characters in utf-8
				
				for bit in (0..32).map(|b| 1 << b) {
					let char = match (upper & bit != 0, lower & bit != 0) {
						(false, false) => ' ',
						(false, true ) => '▄',
						(true,  false) => '▀',
						(true,  true ) => '█',
					};
					line.push(char);
				}
				
				queue!(io::stdout(), cursor::MoveTo(1, y as u16 + 3), style::Print(line))?;
			}
			
			queued = true;
		}
		
		if let Some(buttons) = buttons.changed(&vm) {
			let x_start = 4_i16;
			let y_mid = 21_i16;
			
			let elements: [(i16, i16, &str, &str); 8] = [
				(0, 0, "◁", "◀"),
				(2, 1, "▽", "▼"),
				(4, 0, "▷", "▶"),
				(2, -1, "△", "▲"),
				(25, 0, "B", "B"),
				(22, 0, "A", "A"),
				(7, 0, "SELECT", "SELECT"),
				(15, 0, "START", "START"),
			];
			
			fn draw_controller_background(x_start: u16, y_start: u16, w: u16, h: u16) -> Result<()> {
				let str: String = (0..w).map(|_| ' ').collect();
				for y in y_start..(y_start + h) {
					queue!(io::stdout(),
						cursor::MoveTo(x_start, y),
						style::SetBackgroundColor(Color::Rgb { r: 0x24, g: 0x24, b: 0x24 }),
						style::Print(&str)
					)?;
				}
				Ok(())
			}
			
			draw_controller_background(x_start as u16 - 1, y_mid as u16 - 1, 28, 3)?;
			
			for (i, (x, y, off_str, on_str)) in elements.iter().copied().enumerate() {
				let x = (x_start + x) as u16;
				let y = (y_mid + y) as u16;
				if (buttons & (1 << i)) != 0 {
					queue!(io::stdout(),
						cursor::MoveTo(x, y),
						style::SetForegroundColor(Color::Rgb { r: 0xff, g: 0xff, b: 0xff }),
						style::SetBackgroundColor(Color::Rgb { r: 0x20, g: 0x20, b: 0x20 }),
						style::Print(on_str),
					)?;
				}else{
					queue!(io::stdout(),
						cursor::MoveTo(x, y),
						style::SetForegroundColor(Color::Rgb { r: 0xaa, g: 0xaa, b: 0xaa }),
						style::SetBackgroundColor(Color::Rgb { r: 0x20, g: 0x20, b: 0x20 }),
						style::Print(off_str),
					)?;
				}
			}
			
			queue!(io::stdout(),
				style::SetBackgroundColor(Color::Rgb { r: 0x2d, g: 0x17, b: 0x10 }),
				style::SetForegroundColor(Color::Rgb { r: 0xf0, g: 0xd4, b: 0xac }),
			)?;
			
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
		event::PopKeyboardEnhancementFlags,
		cursor::Show,
		terminal::LeaveAlternateScreen,
	));
	
	if terminal::is_raw_mode_enabled().unwrap_or(true) {
		print_err!(terminal::disable_raw_mode());
	}
}
