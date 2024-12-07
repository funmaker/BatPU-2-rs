use std::collections::HashMap;

use crate::asm::AsmError;
use crate::asm::ast::Line;
use crate::isa::{Instruction, MAX_CODE_LEN, Mnemonic};
type Symbols<'a> = HashMap<&'a str, i16>;

pub fn assemble<'lines>(lines: &'lines [Line]) -> impl Iterator<Item=Result<Instruction, AsmError<'lines>>> {
	let mut symbols = Symbols::new();
	
	lines.iter()
	     .map(defining_map(&mut symbols))
	     .flat_map(|result| result.err().map(Err)) // discard Ok(())
	     .chain(lines.iter()
	                 .flat_map(move |line| assemble_line(line, &symbols).transpose()))
}

pub fn assemble_line<'line, 's>(line: &'line Line, symbols: &'s Symbols) -> Result<Option<Instruction>, AsmError<'line>> {
	let mnemonic_token = match line.mnemonic {
		None => return Ok(None),
		Some(mnemonic) if &mnemonic == "define" => return Ok(None),
		Some(mnemonic) => mnemonic,
	};
	
	let mnemonic = Mnemonic::try_from(mnemonic_token.span)
	                        .map_err(|_| AsmError::UnknownMnemonic { line_number: line.line_number, token: mnemonic_token })?;
	
	
	unimplemented!()
}

fn defining_map<'s, 'l>(symbols: &'s mut Symbols<'l>) -> impl for<'a> FnMut(&'a Line) -> Result<(), AsmError<'a>> {
	let mut pc = 0;
	
	move |line: &Line| {
		if let Some(label) = line.label {
			if pc as usize >= MAX_CODE_LEN {
				return Err(AsmError::TooManyInstructions {
					line_number: line.line_number,
					token: label,
				})
			}
			
			symbols.insert(label.span, pc);
		}
		
		if Some("define") == line.mnemonic.as_deref() {
			let [key, value] = line.args.as_ref()
			                       .try_into()
			                       .map_err(|_| AsmError::WrongArgumentsCount {
				                       line_number: line.line_number,
				                       expected: 2,
				                       args: line.args.clone(),
				                       mnemonic: line.mnemonic.unwrap()
			                       })?;
			
			let value = value.parse()
			                 .map_err(|source| AsmError::IntParseError {
				                 line_number: line.line_number,
				                 token: value,
				                 source,
			                 })?;
			
			symbols.insert(key.span, value);
		} else if line.mnemonic.is_some() {
			pc += 1;
		}
		
		Ok(())
	}
}


fn default_symbols(symbol: &str) -> Option<u16> {
	Some(match symbol {
		"pixel_x"             => 240,
		"pixel_y"             => 241,
		"draw_pixel"          => 242,
		"clear_pixel"         => 243,
		"load_pixel"          => 244,
		"buffer_screen"       => 245,
		"clear_screen_buffer" => 246,
		"write_char"          => 247,
		"buffer_chars"        => 248,
		"clear_chars_buffer"  => 249,
		"show_number"         => 250,
		"clear_number"        => 251,
		"signed_mode"         => 252,
		"unsigned_mode"       => 253,
		"rng"                 => 254,
		"controller_input"    => 255,
		
		"nop" => 0x0,
		"hlt" => 0x1,
		"add" => 0x2,
		"sub" => 0x3,
		"nor" => 0x4,
		"and" => 0x5,
		"xor" => 0x6,
		"rsh" => 0x7,
		"ldi" => 0x8,
		"adi" => 0x9,
		"jmp" => 0xA,
		"brh" => 0xB,
		"cal" => 0xC,
		"ret" => 0xD,
		"lod" => 0xE,
		"str" => 0xF,
		
		"r0"  => 0,
		"r1"  => 1,
		"r2"  => 2,
		"r3"  => 3,
		"r4"  => 4,
		"r5"  => 5,
		"r6"  => 6,
		"r7"  => 7,
		"r8"  => 8,
		"r9"  => 9,
		"r10" => 10,
		"r11" => 11,
		"r12" => 12,
		"r13" => 13,
		"r14" => 14,
		"r15" => 15,
		
		"eq" | "="  | "z"  | "zero"     => 0,
		"ne" | "!=" | "nz" | "notzero"  => 1,
		"ge" | ">=" | "c"  | "carry"    => 2,
		"lt" | "<"  | "nc" | "notcarry" => 3,
		
		_ => return None,
	})
}

