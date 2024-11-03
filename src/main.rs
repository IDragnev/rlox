mod error;

use rlox::{
    expression::{
        self,
        Visitor,
    },
    parser,
    scanner::scan
};
use std::{
    env, 
    path::PathBuf,
};

use error::Error;

struct PrintVisitor {}

impl Visitor<String> for PrintVisitor {
    fn visit_literal(&self, _: &Box<dyn Visitor<String>>, e: &expression::Literal) -> String {
        use expression::Literal;

        match e {
            Literal::Number(n) => n.to_string(),
            Literal::String(s) => s.clone(),
            Literal::True => "true".to_owned(),
            Literal::False => "false".to_owned(),
            Literal::Nil => "nil".to_owned(),
        }
    }

    fn visit_unary(&self, v: &Box<dyn Visitor<String>>, e: &expression::Unary) -> String {
        format!("({} {})",
                e.operator.lexeme,
                e.right.accept_string(v),
        )
    }

    fn visit_binary(&self, v: &Box<dyn Visitor<String>>, e: &expression::Binary) -> String {
        format!("({} {} {})",
                e.operator.lexeme,
                e.left.accept_string(v),
                e.right.accept_string(v),
        )
    }

    fn visit_ternary(&self, v: &Box<dyn Visitor<String>>, e: &expression::Ternary) -> String {
        format!("({} {} {})",
                e.cond.accept_string(v),
                e.left.accept_string(v),
                e.right.accept_string(v),
        )
    }

    fn visit_grouping(&self, v: &Box<dyn Visitor<String>>, e: &expression::Grouping) -> String {
        format!("(group {})", e.0.accept_string(v))
    }
}

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

        println!("scanning...");
        match scan(&input) {
            Ok(tokens) => {
                // println!("tokens: {:#?}", tokens);
                println!("parsing...");
                let parser = parser::Parser::new(&tokens);
                match parser.parse() {
                    Ok(expr) => {
                        let visitor: Box<dyn Visitor<String>> = Box::new(PrintVisitor{});
                        let s = expr.accept_string(&visitor);
                        println!("expr: {}", s);
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