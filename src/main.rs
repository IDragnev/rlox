mod error;

use rlox::{
    eval::{
        ExprEvalVisitor, RuntimeError, RuntimeValue
    },
    expression::Visitor,
    parser,
    scanner::scan,
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
    // else if argc == 2 {
    //     let filename = args[1].clone();
    //     println!("Loading file {}...", filename);

    //     let contents = read_file(&PathBuf::from(filename))?;
    //     println!("{}", contents);
    // }

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
    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        input = input.trim().to_string();
        
        if input == "q" {
            break;
        }

        match scan(&input) {
            Ok(tokens) => {
                let parser = parser::Parser::new(&tokens);
                match parser.parse() {
                    Ok(expr) => {
                        let visitor: Box<dyn Visitor<Result<RuntimeValue, RuntimeError>>> = Box::new(ExprEvalVisitor{});
                        let val = expr.accept_rt_value(&visitor);
                        match val {
                            Ok(v) => {
                                let s = stringify(&v);
                                println!("{}", s);
                            },
                            Err(e) => println!("runtime error: {:#?}", e),
                        };
                    },
                    Err(err) => {
                        println!("parse error: {:?}", err);
                    }
                }
            },
            Err(e) => println!("scan error: {:?}", e),
        }
    }

    Ok(())
}

fn stringify(val: &RuntimeValue) -> String {
    match val {
        RuntimeValue::Nil => "nil".to_owned(),
        RuntimeValue::Number(n) => n.to_string(),
        RuntimeValue::Bool(b) => b.to_string(),
        RuntimeValue::String(s) => format!("\"{}\"", s),
    }
}