mod error;

use rlox::{
    interpreter::Interpreter,
    parser::Parser,
    scanner::scan, statement,
};
use std::{
    env, 
    path::PathBuf,
};

use error::Error;

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    let argc = args.len();

    if argc > 2 {
        println!("Usage {} [script]", args[0]);
        std::process::exit(64);
    }

    if argc == 1 {
        repl()?;
    }
    else if argc == 2 {
        let filename = args[1].clone();

        let contents = read_file(&PathBuf::from(filename))?;
        if let Some(stmts) = scan_parse(&contents) {
            let mut interp = Interpreter::new();
            interp.interpret_statements(&stmts);
        }
    }

    // if runtime error in file mode, exit with code=70
    Ok(())
}

fn read_file(filename: &PathBuf) -> Result<String, Error> {
    use std::fs::File;
    use std::io::prelude::*;

    let mut file = File::open(filename)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    Ok(contents)
}

fn repl() -> Result<(), Error> {
    let mut interp = Interpreter::new();

    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        input = input.trim().to_string();
        
        if input == "q" {
            break;
        }
        if let Some(statements) = scan_parse(&input) {
            interp.interpret_statements(&statements);
        }
    }

    Ok(())
}

fn scan_parse(input: &str) -> Option<Vec<Box<dyn statement::Stmt>>> {
    match scan(&input) {
        Ok(tokens) => {
            let parser = Parser::new(&tokens);
            match parser.parse() {
                Ok(statements) => {
                    return Some(statements)
                },
                Err(errs) => {
                    println!("parse error: {:?}", errs);
                }
            }
        },
        Err(e) => {
            println!("scan error: {:?}", e);
        }
    }

    None
}