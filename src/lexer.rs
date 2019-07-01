#[derive(Debug, Clone)]
pub struct Token {
    pub variant: TokenType,
    pub value: String,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    LetKeyword,
    Identifier,
    NumberLiteral,
    StringLiteral,
    Semicolon,
    OpenBrace,
    CloseBrace,
    OpPlus,
    OpMinus,
    OpMultiply,
    OpDivide,
    OpModulo,
    OpAssign,
    OpFunction,
}

pub fn tokenize(b: &str) -> Result<Vec<Token>, TokenizeError> {
    let mut iter = b.bytes().enumerate().peekable();
    let mut tokens = vec![];
    while let Some((i, c)) = iter.next() {
        match c {
            b'{' => tokens.push(Token {
                variant: TokenType::OpenBrace,
                value: b[i..i + 1].to_owned(),
            }),
            b'}' => tokens.push(Token {
                variant: TokenType::CloseBrace,
                value: b[i..i + 1].to_owned(),
            }),
            b'=' => tokens.push(Token {
                variant: TokenType::OpAssign,
                value: b[i..i + 1].to_owned(),
            }),
            b'+' => tokens.push(Token {
                variant: TokenType::OpPlus,
                value: b[i..i + 1].to_owned(),
            }),
            b'-' => {
                let mut token = Token {
                    variant: TokenType::OpMinus,
                    value: b[i..i + 1].to_owned(),
                };
                if let Some(&(i, c)) = iter.peek() {
                    if c == b'>' {
                        iter.next();
                        token = Token {
                            variant: TokenType::OpFunction,
                            value: b[i - 1..i + 1].to_owned(),
                        };
                    }
                }
                tokens.push(token);
            }
            b'*' => tokens.push(Token {
                variant: TokenType::OpMultiply,
                value: b[i..i + 1].to_owned(),
            }),
            b'/' => tokens.push(Token {
                variant: TokenType::OpDivide,
                value: b[i..i + 1].to_owned(),
            }),
            b'%' => tokens.push(Token {
                variant: TokenType::OpModulo,
                value: b[i..i + 1].to_owned(),
            }),
            b';' => tokens.push(Token {
                variant: TokenType::Semicolon,
                value: b[i..i + 1].to_owned(),
            }),
            _ if c.is_ascii_alphabetic() || c == b'_' => {
                let start = i;
                let mut end = i + 1;
                while let Some(&(i, c)) = iter.peek() {
                    if c.is_ascii_alphanumeric() || c == b'_' {
                        iter.next();
                        end = i + 1;
                    } else {
                        break;
                    }
                }
                let value = &b[start..end];
                let variant = {
                    match value {
                        "let" => TokenType::LetKeyword,
                        _ => TokenType::Identifier,
                    }
                };
                tokens.push(Token {
                    variant: variant,
                    value: value.to_owned(),
                });
            }
            _ if c.is_ascii_digit() => {
                let start = i;
                let mut end = i + 1;
                while let Some(&(i, c)) = iter.peek() {
                    if c.is_ascii_digit() {
                        iter.next();
                        end = i + 1;
                    } else {
                        break;
                    }
                }
                tokens.push(Token {
                    variant: TokenType::NumberLiteral,
                    value: b[start..end].to_owned(),
                });
            }
            b'"' => {
                let start = i;
                while let Some((i, c)) = iter.next() {
                    if c == b'"' {
                        tokens.push(Token {
                            variant: TokenType::StringLiteral,
                            value: b[start..i + 1].to_owned(),
                        });
                        break;
                    }
                }
            }
            _ if c.is_ascii_whitespace() => {}
            _ => return Err(TokenizeError::UnexpectedCharacter(c as char)),
        }
    }
    Ok(tokens)
}


#[derive(Debug)]
pub enum TokenizeError {
    UnexpectedCharacter(char),
}


impl std::fmt::Display for TokenizeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for TokenizeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}