use std::collections::HashMap;
use crate::asm::ast::{File, Operand};
use crate::Char;

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
	
	map
}

pub fn assemble(code: File) -> Result<Vec<u16>, String> {
	let mut symbols = default_symbols();
	let mut pc = 0;
	
	for (line_number, line) in code.lines.iter().enumerate() {
		if let Some(label) = line.label.clone() {
			symbols.insert(label, pc);
		}
		if let Some(instruction) = &line.instruction {
			if instruction.mnemonic == "define" {
				if instruction.operands.len() != 2 {
					return Err(format!("{}Expected 2 arguments for define, got {}", code.error_prefix(line_number), instruction.operands.len()));
				}
				if let Operand::Number(value) = instruction.operands[1] {
					if value >= 1024 || value < -128 {
						return Err(format!("{}Value out of acceptable bounds, got {} (0x{:2x})", code.error_prefix(line_number), value, value));
					}
					symbols.insert(instruction.operands[0].to_string(), value as u16);
				}
			} else {
				pc += 1;
			}
		}
	}
	
	let machine_code = Vec::with_capacity(pc as usize);
	
	// todo
	
	Ok(machine_code)
}
