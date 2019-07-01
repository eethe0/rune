extern crate rune;

use std::io::Write;

fn main() {
    loop {
        let mut buf = String::new();
        print!("> ");
        std::io::stdout()
            .flush()
            .ok()
            .expect("Couldn't flush stdout");
        std::io::stdin()
            .read_line(&mut buf)
            .expect("Couldn't read from stdin");

        let code = buf.as_ref();

        match rune::lexer::tokenize(code) {
            Ok(tokens) => {
                //println!("{:#?}", tokens);
                match rune::parser::parse_expr(tokens.as_slice()) {
                    Ok(expr) => {
                        //println!("{:#?}", module);
                        println!(
                            "{:#?}",
                            rune::interpreter::eval_expression(
                                &expr,
                                rune::interpreter::Scope::new()
                            )
                        );
                    }
                    Err(err) => println!("{:#?}", err),
                }

            }
            Err(err) => println!("{:#?}", err),
        }
    }
}
