use std::collections::HashMap;
use crate::asm::ast::{File, Operand, ResolvedInstruction};
use crate::{isa, Char};

fn default_symbols() -> HashMap<String, u16> {
	let mut map = HashMap::new();
	
	for (i, port) in [
		"pixel_x", "pixel_y", "draw_pixel", "clear_pixel",
		"load_pixel", "buffer_screen", "clear_screen_buffer", "write_char",
		"buffer_chars", "clear_chars_buffer", "show_number", "clear_number",
		"signed_mode", "unsigned_mode", "rng", "controller_input"
	].iter().enumerate() {
		map.insert(port.to_string(), (240 + i) as u16);
	}
	
	for (i, opcode) in [
		"nop", "hlt", "add", "sub", "nor", "and", "xor", "rsh",
		"ldi", "adi", "jmp", "brh", "cal", "ret", "lod", "str"
	].iter().enumerate() {
		map.insert(opcode.to_string(), i as u16);
	}
	
	for i in 0..16 {
		map.insert(format!("r{}", i), i as u16);
	}
	
	for (i, chr) in Char::TABLE.iter().enumerate() {
		// I cry a little
		map.insert(format!("'{}'", chr), i as u16);
		map.insert(format!("\"{}\"", chr), i as u16);
	}
	
	for conditions in [
		["eq", "ne", "ge", "lt"],
		["=", "!=", ">=", "<"],
		["z", "nz", "c", "nc"],
		["zero", "notzero", "carry", "notcarry"],
	] {
		for (i, name) in conditions.into_iter().enumerate() {
			map.insert(name.to_string(), i as u16);
		}
	}
	
	map
}

pub fn assemble(code: File) -> Result<Vec<u16>, String> {
	let mut symbols = default_symbols();
	let mut pc = 0;
	
	for line in code.lines.iter() {
		if let Some(label) = line.label.clone() {
			symbols.insert(label, pc);
		}
		if let Some(instruction) = &line.instruction {
			if instruction.mnemonic == "define" {
				if instruction.operands.len() != 2 {
					return Err(format!("{}Expected 2 arguments for define, got {}", code.error_prefix(line.line_number), instruction.operands.len()));
				}
				if let Operand::Number(value) = instruction.operands[1] {
					if value >= 1024 || value < -128 {
						return Err(format!("{}Value out of acceptable bounds, got {} (0x{:2x})", code.error_prefix(line.line_number), value, value));
					}
					symbols.insert(instruction.operands[0].to_string(), value as u16);
				}else{
					return Err(format!("{}Invalid argument for define: {:?}", code.error_prefix(line.line_number), instruction.operands[1]));
				}
			} else {
				pc += 1;
			}
		}
	}
	
	let resolved_instructions =
		code.lines.into_iter()
		    .flat_map(|line| line.instruction.map(|i| (line.line_number, i)))
			.filter(|(_, i)| i.mnemonic != "define")
		    .map(|(line_number, instruction)| Ok(ResolvedInstruction {
			    mnemonic: instruction.mnemonic.to_ascii_uppercase(),
			    operands: instruction.operands.into_iter().map(|operand| match operand {
				    Operand::Number(val) => { Ok(val as u16) }
				    Operand::Character(chr) => { Ok(chr.as_u8() as u16) }
				    Operand::Symbol(name) => { symbols.get(&name).map(|value| Ok(*value)).unwrap_or(Err(format!("{}: Unknown symbol '{}'", line_number, name))) }
			    }).collect::<Result<Vec<u16>, String>>()?,
		    })).collect::<Result<Vec<ResolvedInstruction>, String>>()?;
	
	let machine_code =
		resolved_instructions.iter()
		                     .map(isa::Instruction::try_from)
		                     .map(|i| i.map(|i| i.as_u16()))
		                     .collect::<Result<Vec<u16>, String>>()?;
	
	Ok(machine_code)
}
