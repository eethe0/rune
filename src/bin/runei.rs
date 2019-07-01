extern crate rune;

use std::io::Read;

fn main() -> Result<(), Box<std::error::Error>> {
    let mut bytes = vec![];
    std::fs::File::open("test.rn")?.read_to_end(&mut bytes)?;

    let code = std::str::from_utf8(bytes.as_slice()).unwrap();
    let tokens = rune::lexer::tokenize(code)?;
    let module = rune::parser::parse_module(tokens.as_slice())?;
    let scope = rune::interpreter::eval_module(&module);

    println!("{:#?}", scope);

    Ok(())
}

