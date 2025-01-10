use std::fs;
use anyhow::{bail, Context, Result};
use batpu2::{asm, isa, utils};

pub fn cmd(input_path: &str, output_path: &str) -> Result<()> {
	let asm = fs::read_to_string(input_path).with_context(|| format!("Failed to open: \"{input_path}\""))?;
	
	let code = assemble(&asm, input_path)?;
	let code = utils::into_mc(&code);
	
	fs::write(output_path, code).with_context(|| format!("Failed to create: \"{output_path}\""))?;
	
	Ok(())
}

pub fn assemble(input: &str, input_path: &str) -> Result<Vec<isa::Instruction>> {
	let code = collect_asm(asm::parse_lines(&input), input_path, &input)?;
	let code = collect_asm(asm::assemble(&code), input_path, &input)?;
	
	Ok(code)
}

fn collect_asm<'a, T>(iter: impl Iterator<Item=std::result::Result<T, asm::AsmError<'a>>>,
                      input_path: &str,
                      input: &str)
                      -> Result<Vec<T>> {
	let mut output = Vec::with_capacity(iter.size_hint().1.unwrap_or(32));
	let mut error_count = 0;
	
	for result in iter {
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
					
					eprint!("{:pad$} | ", "");
					let mut apos = 0;
					for arg in err.tokens() {
						let apad = arg.char_number.saturating_sub(apos + 1);
						let alen = arg.len();
						eprint!("{:apad$}", "");
						for _ in 0..alen {
							eprint!("{}", if arg == token { "^" } else { "~" });
						}
						apos += apad + alen;
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

