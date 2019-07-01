use crate::lexer::*;

type Iter<'a> = std::iter::Peekable<std::slice::Iter<'a, Token<'a>>>;

pub fn parse_module<'a>(tokens: &'a [Token<'a>]) -> Result<Module<'a>, ParseError<'a>> {
    Module::parse(&mut tokens.iter().peekable())
}

pub fn parse_expr<'a>(tokens: &'a [Token<'a>]) -> Result<Expression<'a>, ParseError<'a>> {
    Expression::parse(&mut tokens.iter().peekable())
}

fn try_match<'a>(iter: &mut Iter<'a>, ty: TokenType) -> Result<&'a Token<'a>, ParseError<'a>> {
    if let Some(token) = iter.peek() {
        if token.variant == ty {
            let r = Ok(*token);
            iter.next();
            r
        } else {
            Err(ParseError::NoMatch)
        }
    } else {
        Err(ParseError::NoMatch)
    }
}

fn expect<'a>(iter: &mut Iter<'a>, ty: TokenType) -> Result<&'a Token<'a>, ParseError<'a>> {
    if let Some(token) = iter.next() {
        if token.variant == ty {
            Ok(token)
        } else {
            Err(ParseError::UnexpectedToken(token, ty))
        }
    } else {
        Err(ParseError::UnexpectedEndOfFile)
    }
}

#[derive(Debug)]
pub struct Module<'a> {
    pub declarations: Vec<Declaration<'a>>,
}

impl<'a> Module<'a> {
    fn parse(iter: &mut Iter<'a>) -> Result<Self, ParseError<'a>> {
        let mut module = Module {
            declarations: vec![],
        };
        loop {
            match Declaration::parse(iter) {
                Ok(decl) => module.declarations.push(decl),
                Err(err) => match err {
                    ParseError::NoMatch => break,
                    _ => return Err(err),
                },
            }
        }
        if let Some(token) = iter.next() {
            Err(ParseError::ExpectingOther(
                token,
                "declaration or end of file",
            ))
        } else {
            Ok(module)
        }
    }
}

#[derive(Debug)]
pub struct Declaration<'a> {
    pub identifier: &'a str,
    pub initializer: Expression<'a>,
}

impl<'a> Declaration<'a> {
    fn parse(iter: &mut Iter<'a>) -> Result<Self, ParseError<'a>> {
        try_match(iter, TokenType::LetKeyword)?;
        let id = expect(iter, TokenType::Identifier)?.value;
        expect(iter, TokenType::OpAssign)?;
        let expr = Expression::parse(iter)?;
        Ok(Declaration {
            identifier: id,
            initializer: expr,
        })
        //expect(iter, TokenType::Semicolon)?;
    }
}

#[derive(Debug)]
pub enum Statement<'a> {
    DeclarationStatement(Declaration<'a>),
    ExpressionStatement(Expression<'a>),
    ReturnStatement(Expression<'a>),
}

#[derive(Debug, Clone)]
pub enum Expression<'a> {
    IdentifierExpression(&'a str),
    NumberExpression(&'a str),
    StringExpression(&'a str),
    FunctionExpression(&'a str, Box<Expression<'a>>),
    CallExpression(Box<Expression<'a>>, Box<Expression<'a>>),
    BinaryExpression(Operator, Box<Expression<'a>>, Box<Expression<'a>>),
    UnaryExpression(Operator, Box<Expression<'a>>),
    BlockExpression,
}

/// http://www.engr.mun.ca/~theo/Misc/exp_parsing.htm#climbing
impl<'a> Expression<'a> {
    fn parse(iter: &mut Iter<'a>) -> Result<Self, ParseError<'a>> {
        Self::exp(iter, 0)
    }

    fn exp(iter: &mut Iter<'a>, p: u8) -> Result<Self, ParseError<'a>> {
        let mut t = Self::p(iter)?;
        while let Some(token) = iter.peek() {
            if let Some(op) = Operator::binary(&token.variant) {
                if op as u8 >= p {
                    iter.next();
                    let q = match op.associativity() {
                        Associativity::Right => op as u8,
                        Associativity::Left => 1 + op as u8,
                    };
                    let t1 = Self::exp(iter, q)?;
                    t = Expression::BinaryExpression(op, Box::new(t), Box::new(t1));
                } else {
                    break;
                }
            } else {
                // Left associative but highest precedence
                if let Ok(t1) = Self::exp(iter, 255) {
                    t = Expression::CallExpression(Box::new(t), Box::new(t1));
                } else {
                    break;
                }
            }
        }
        Ok(t)
    }

    fn p(iter: &mut Iter<'a>) -> Result<Self, ParseError<'a>> {
        if let Some(token) = iter.peek() {
            if let Some(op) = Operator::unary(&token.variant) {
                iter.next();
                let q = op as u8;
                Ok(Expression::UnaryExpression(
                    op,
                    Box::new(Self::exp(iter, q)?),
                ))
            } else {
                match token.variant {
                    TokenType::Identifier => {
                        let id = token.value;
                        let mut t = Expression::IdentifierExpression(id);
                        iter.next();
                        if let Ok(_) = try_match(iter, TokenType::OpFunction) {
                            t = Expression::FunctionExpression(
                                id,
                                Box::new(Expression::parse(iter)?),
                            );
                        }
                        Ok(t)
                    }
                    TokenType::NumberLiteral => {
                        let t = Expression::NumberExpression(token.value);
                        iter.next();
                        Ok(t)
                    }
                    TokenType::StringLiteral => {
                        let t = Expression::StringExpression(token.value);
                        iter.next();
                        Ok(t)
                    }
                    _ => Err(ParseError::UnexpectedTokenMultiple(
                        token,
                        &[
                            TokenType::Identifier,
                            TokenType::NumberLiteral,
                            TokenType::StringLiteral,
                        ],
                    )),
                }
            }
        } else {
            Err(ParseError::UnexpectedEndOfFile)
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub enum Operator {
    //Assign,
    //Function,
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    UnaryPlus,
    UnaryMinus,
}

#[derive(Debug)]
enum Associativity {
    Right,
    Left,
}

impl Operator {
    fn unary(ty: &TokenType) -> Option<Self> {
        match ty {
            TokenType::OpPlus => Some(Operator::UnaryPlus),
            TokenType::OpMinus => Some(Operator::UnaryMinus),
            _ => None,
        }
    }

    fn binary(ty: &TokenType) -> Option<Self> {
        match ty {
            TokenType::OpPlus => Some(Operator::Add),
            TokenType::OpMinus => Some(Operator::Subtract),
            TokenType::OpMultiply => Some(Operator::Multiply),
            TokenType::OpDivide => Some(Operator::Divide),
            TokenType::OpModulo => Some(Operator::Modulo),
            //TokenType::OpAssign => Some(Operator::Assign),
            //TokenType::OpFunction => Some(Operator::Function),
            _ => None,
        }
    }

    fn associativity(&self) -> Associativity {
        match self {
            //Operator::Assign => Associativity::Right,
            //Operator::Function => Associativity::Right,
            Operator::Add => Associativity::Left,
            Operator::Divide => Associativity::Left,
            Operator::Modulo => Associativity::Left,
            Operator::Multiply => Associativity::Left,
            Operator::Subtract => Associativity::Left,
            Operator::UnaryMinus => Associativity::Right,
            Operator::UnaryPlus => Associativity::Right,
        }
    }
}

#[derive(Debug)]
pub enum ParseError<'a> {
    UnexpectedToken(&'a Token<'a>, TokenType),
    UnexpectedTokenMultiple(&'a Token<'a>, &'a [TokenType]),
    ExpectingOther(&'a Token<'a>, &'a str),
    UnexpectedEndOfFile,
    NoMatch,
}

impl<'a> std::fmt::Display for ParseError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<'a> std::error::Error for ParseError<'a> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
