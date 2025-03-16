mod error;

use rlox::{
    interpreter::Interpreter,
    parser::Parser,
    resolver::Resolver,
    statement::Stmt,
    scanner,
    statement,
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

        if let Some(mut stmts) = scan_parse(&contents) {
            let mut resolver = Resolver::new();
            if resolve(&mut resolver, &mut stmts) == false {
                std::process::exit(1);
            }

            let mut interp = Interpreter::new();
            if let Err(e) =  interp.execute(&stmts) {
                println!("Runtime error: {:#?}", e);
                std::process::exit(70);
            }
        }
    }

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
    let mut resolver = Resolver::new();

    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        input = input.trim().to_string();
        
        if input == "q" {
            break;
        }

        if let Some(tokens) = scan_input(&input) {
            // Try to parse an expression first.
            // If this fails, try to parse statements.
            let parser = Parser::new(&tokens);
            match parser.parse_single_expr() {
                Ok(mut expr) => {
                    if let Err(e) = resolver.resolve_single_expr(&mut expr) {
                        println!("Error: {:#?}", e);
                    }
                    else {
                        match interp.evaluate_expr(&expr) {
                            Ok(v) => {
                                println!("{}", &v);
                            },
                            Err(e) => {
                                println!("Error: {:#?}", e);
                            }
                        }
                    }
                }
                Err(_) => {
                    match parser.parse() {
                        Ok(mut statements) => {
                            if resolve(&mut resolver, &mut statements) {
                                if let Err(e) =  interp.execute(&statements) {
                                    println!("Error: {:#?}", e);
                                }
                            }
                        },
                        Err(errs) => {
                            report_parse_errors(&errs);
                        }
                    }
                }
            }
        }

    }

    Ok(())
}

fn resolve(r: &mut Resolver, stmts: &mut Vec<Box<dyn Stmt>>) -> bool {
    let result = r.resolve(stmts);

    if let Some(warning) = result.warnings {
        println!("Warnings: {:#?}", warning);
    }

    if let Some(errs) = result.errors {
        println!("Compile errors: {:#?}", errs);
        return false;
    }
    
    true
}

fn scan_parse(input: &str) -> Option<Vec<Box<dyn statement::Stmt>>> {
    if let Some(tokens) = scan_input(&input) {
        let parser = Parser::new(&tokens);
        match parser.parse() {
            Ok(statements) => {
                return Some(statements)
            },
            Err(errs) => {
                report_parse_errors(&errs);
            }
        }
    }

    None
}

fn scan_input(input: &str) -> Option<Vec<scanner::Token>> {
    match scanner::scan(&input) {
        Ok(tokens) => Some(tokens),
        Err(e) => {
            report_scan_errors(&e);
            None
        }
    }
}

fn report_scan_errors(e: &rlox::scanner::ScanError) {
    use scanner::ScanError;
    use scanner::TokenErrorType;

    println!("Scanner error.");

    match e {
        ScanError::NonAsciiCharacterFound => {
            println!("Only ASCII characters are supported.");
        },
        ScanError::TokenError(token_errs) => {
            for te in token_errs {
                let err_type = match te.error {
                    TokenErrorType::UnexpectedCharacter => "Unexpected character found.",
                    TokenErrorType::UnterminatedString => "Unterminated string.",
                };
                println!("Error at line {}, column {}: {}", te.line, te.column, err_type);
            }
        },
    }
}

fn report_parse_errors(errs: &Vec<rlox::parser::ParseError>) {
    use rlox::parser::ParseErrorType;

    println!("Parse error.");

    for e in errs {
        let mut line = None;
        let mut column = None;
        let mut err_type = None;
        let _ = err_type.is_some(); // silence warning

        match e.error_type {
            ParseErrorType::ExpectedExpression => {
                if let Some(t) = &e.token {
                    line = Some(t.line);
                    column = Some(t.column);
                }
                err_type = Some("Expected expression.".to_owned());
            },
            ParseErrorType::ExpectedForLoopInitializerOrSemiColon => {
                if let Some(t) = &e.token {
                    line = Some(t.line);
                    column = Some(t.column + 1);
                }
                err_type = Some("Expected for loop initializer or semicolon.".to_owned());
            },
            ParseErrorType::ExpectedForLoopConditionOrSemiColon => {
                if let Some(t) = &e.token {
                    line = Some(t.line);
                    column = Some(t.column);
                }
                err_type = Some("Expected for loop condition or semicolon after initializer.".to_owned());
            },
            ParseErrorType::ExpectedStatement => {
                if let Some(t) = &e.token {
                    line = Some(t.line);
                    column = Some(t.column);
                }
                err_type = Some("Expected statement.".to_owned());
            },
            ParseErrorType::ExpectedToken { expected, found } => {
                if let Some(t) = &e.token {
                    line = Some(t.line);
                    column = Some(t.column);
                }

                if found.is_some() {
                    err_type = Some(format!("Expected {:?}, found {:?}.", expected, found.unwrap()));
                }
                else {
                    err_type = Some(format!("Expected {:?}.", expected));
                }
            },
            ParseErrorType::InvalidAssignment => {
                if let Some(t) = &e.token {
                    line = Some(t.line);
                    column = Some(t.column);
                }
                err_type = Some("Invalid assignment.".to_owned());
            },
        }

        if let Some(msg) = err_type {
            if line.is_some() && column.is_some() {
                println!(
                    "Error at line {}, column {}: {}",
                    line.unwrap(),
                    column.unwrap(),
                    msg,
                );
            }
            else {
                println!("Error: {}", msg);
            }
        }
    }
}