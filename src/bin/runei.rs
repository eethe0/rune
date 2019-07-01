extern crate rune;

use std::io::Read;

fn main() {
    let mut bytes = vec![];
    std::fs::File::open("test.rn")
        .expect("Could not open file.")
        .read_to_end(&mut bytes)
        .expect("Could not read file.");

    let code = std::str::from_utf8(bytes.as_slice()).unwrap();

    match rune::lexer::tokenize(code) {
        Ok(tokens) => {
            //println!("{:#?}", tokens);
            match rune::parser::parse_module(tokens.as_slice()) {
                Ok(module) => {
                    //println!("{:#?}", module);
                    println!("{:#?}", rune::interpreter::eval_module(&module));
                }
                Err(err) => println!("{:#?}", err),
            }
        }
        Err(err) => println!("{:#?}", err),
    }
}

