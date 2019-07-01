extern crate rune;

use rune::interpreter::*;
use rune::lexer::*;
use rune::parser::*;
use std::io::Write;

fn main() -> Result<(), Box<std::error::Error>> {
    let mut scope = Scope::new();
    loop {
        println!("{:?}", repl(&mut scope)?);
    }
}

fn repl(scope: &mut Scope) -> Result<Value, Box<std::error::Error>> {
    let mut buf = String::new();
    print!("> ");
    std::io::stdout().flush()?;
    std::io::stdin().read_line(&mut buf)?;

    let code = buf.as_ref();
    let tokens = tokenize(code)?;
    let value = match parse_single_decl(tokens.as_slice()) {
        Ok(decl) => Ok(eval_declaration(&decl, scope)),
        Err(err) => match err {
            ParseError::NoMatch => {
                let expr = parse_single_expr(tokens.as_slice())?;
                Ok(eval_expression(&expr, scope.clone()))
            }
            _ => Err(err),
        },
    }?;

    Ok(value)
}