pub mod ast;
pub mod parser;




#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	#[cfg(feature = "embedded_io")]
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
}