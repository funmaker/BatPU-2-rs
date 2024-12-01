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
		  MOV 1 2 3000 # comment text
	    .second
		  LDI r1 ' '
		  TEST < start";
		
		let file_ast = ast::File::from_text(code, None);
		assert!(file_ast.is_ok());
		
		let assembled = assembler::assemble(file_ast.unwrap());
		assert!(assembled.is_ok());
	}
}
