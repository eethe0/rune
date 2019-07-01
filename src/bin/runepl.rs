extern crate rune;

use std::io::Write;

fn main() -> Result<(), Box<std::error::Error>> {
    loop {
        println!("{:?}", repl()?);
    }
}

fn repl() -> Result<rune::interpreter::Value, Box<std::error::Error>> {
    let mut buf = String::new();
    print!("> ");
    std::io::stdout().flush()?;
    std::io::stdin().read_line(&mut buf)?;

    let code = buf.as_ref();
    let tokens = rune::lexer::tokenize(code)?;
    let expr = rune::parser::parse_expr(tokens.as_slice())?;
    let value = rune::interpreter::eval_expression(&expr, rune::interpreter::Scope::new());

    Ok(value)
}