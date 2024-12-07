pub mod ast;
pub mod assembler;

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn ast_parses() {
		let code = r"
		.start
		  MOV 1 2 3  # comment text
		  LDI r1 ' '
		  TEST < start";
		
		let file_ast = ast::File::from_text(code, None);
		assert!(file_ast.is_ok());
		
		let display = format!("{}", file_ast.unwrap());
		assert_eq!(display, ".start\nMOV 1 2 3 ; comment text\nLDI r1 ' '\nTEST < start\n");
	}
	
	#[test]
	fn assembler_assembles() {
		let code = r"
		.start
		  MOV 1 5 # comment text
	    .second
		  LDI r1 ' '
		  BRH < start";
		
		let file_ast = ast::File::from_text(code, None);
		assert!(file_ast.is_ok());
		
		let assembled = assembler::assemble(file_ast.unwrap());
		assert!(assembled.is_ok());
	}
	
	#[test]
	fn assembler_assembles_2() {
		let code = r"
		ldi r15 buffer_chars
		define write -1
		LDI r4 'D'
		STR r15 r4 write";
		
		let file_ast = ast::File::from_text(code, None);
		assert!(file_ast.is_ok());
		
		let assembled = assembler::assemble(file_ast.unwrap());
		
		match assembled {
			Ok(_) => {}
			Err(_) => {
				println!("{:?}", assembled);
				assert!(false);
			}
		}
		assert!(assembled.is_ok());
		
		let code = assembled.unwrap();
		
		assert_eq!(code.len(), 3);
		
		let expected = "0b1000111111111000 0b1000010000000100 0b1111111101001111";
		
		assert_eq!(
			format!(
				"{:#018b} {:#018b} {:#018b}",
				code.get(0).unwrap(), code.get(1).unwrap(), code.get(2).unwrap()
			),
			expected
		);
	}
}
