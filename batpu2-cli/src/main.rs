
use std::fs;
use std::process::exit;
use anyhow::{Context, Result};

mod arguments;

use arguments::{Arguments, Command};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let program = &args[0];
    let mut arguments = Arguments::new();
    
    if let Err(err) = arguments.parse(&args[1..]) {
        eprintln!("{err}");
        arguments.print_usage(program, true);
        exit(-1);
    }
    
    match arguments.command {
        Command::Help => arguments.print_usage(program, false),
        Command::Run(path) => {
            let file = fs::read(&path).with_context(|| format!("Failed to open: \"{path}\""))?;
            
            print!("{}", String::from_utf8_lossy(&file));
        }
    }
    
    Ok(())
}