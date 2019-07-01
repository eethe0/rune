mod parser;
mod lexer;
mod interpreter;

use std::io::Read;

fn main() {
    let mut bytes = vec![];
    std::fs::File::open("test.rn")
        .expect("Could not open file.")
        .read_to_end(&mut bytes)
        .expect("Could not read file.");

    let code = std::str::from_utf8(bytes.as_slice()).unwrap();

    match lexer::tokenize(code) {
        Ok(tokens) => {
            //println!("{:#?}", tokens);
            match parser::parse(tokens.as_slice()) {
                Ok(module) => {
                    println!("{:#?}", module);
                    println!("{:#?}", interpreter::eval(&module));
                }
                Err(err) => println!("{:#?}", err),
            }
        }
        Err(err) => println!("{:#?}", err),
    }
}

