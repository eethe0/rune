mod parser;
mod tokenizer;
use std::io::Read;

fn main() {
    let mut bytes = vec![];
    std::fs::File::open("test.gb")
        .expect("Could not open file.")
        .read_to_end(&mut bytes)
        .expect("Could not read file.");

    let code = std::str::from_utf8(bytes.as_slice()).unwrap();

    match tokenizer::tokenize(code) {
        Ok(tokens) => {
            //println!("{:#?}", tokens);
            println!("{:#?}", parser::parse(tokens.as_slice()));
        }
        Err(err) => println!("{:#?}", err),
    }

}
