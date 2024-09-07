mod error;

use rlox::scanner::scan;

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
        println!("Loading file {}...", filename);

        let contents = read_file(&PathBuf::from(filename))?;
        println!("{}", contents);
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
    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        input = input.trim().to_string();
        
        if input == "q" {
            break;
        }

        println!("you entered {}", input);
        match scan(&input) {
            Ok(tokens) => println!("tokens: {:?}", tokens),
            Err(e) => println!("{:?}", e),
        }
    }

    Ok(())
}