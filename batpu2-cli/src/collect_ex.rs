use anyhow::{bail, Result};
use batpu2::asm::AsmError;

pub trait CollectAsm {
	type Output;
	
	fn collect_asm(self, input_path: &str, input: &str) -> Result<Vec<Self::Output>>;
}

impl<'a, T, O> CollectAsm for T
where T: Iterator<Item=Result<O, AsmError<'a>>> {
	type Output = O;
	
	fn collect_asm(self, input_path: &str, input: &str) -> Result<Vec<Self::Output>> {
		let mut output = Vec::with_capacity(self.size_hint().1.unwrap_or(32));
		let mut error_count = 0;
		
		for result in self {
			match result {
				Err(err) => {
					if error_count == 0 {
						let pad = err.line_num().ilog10() as usize + 1;
						let token = err.token();
						let tpad = token.char_number.saturating_sub(1);
						
						eprintln!("error: {err}");
						eprintln!("{:pad$}--> {input_path}:{}:{}", "", err.line_num(), err.col_num());
						eprintln!("{:pad$} |", "");
						eprintln!("{:pad$} | {}", err.line_num(), input.lines().skip(err.line_num().saturating_sub(1)).next().unwrap_or("<EOF>"));
						
						eprint!("{:pad$} | {:tpad$}{}", "", "", "^".repeat(token.len()));
						if let AsmError::WrongOperandCount { args, .. } = &err {
							let mut apos = tpad + token.len();
							for arg in args {
								let apad = arg.char_number.saturating_sub(apos + 1);
								eprint!("{:apad$}{}", "", "~".repeat(arg.len()));
								apos += apad + arg.len();
							}
						}
						eprintln!();
						
						eprintln!("{:pad$} | {:tpad$}|", "", "");
						eprintln!("{:pad$} | {:tpad$}{err}", "", "");
						eprintln!("{:pad$} |", "");
					} else if error_count < 5 {
						eprintln!("{input_path}:{}:{} error: {err}", err.line_num(), err.col_num());
					}
					
					error_count += 1;
				},
				Ok(result) => {
					if error_count == 0 {
						output.push(result);
					}
				},
			}
		}
		
		if error_count > 5 {
			eprintln!();
			eprintln!("({} errors skipped...)", error_count - 5);
			eprintln!();
		}
		
		if error_count > 0 { bail!("Compilation aborted due to {error_count} errors.") }
		
		Ok(output)
	}
}
