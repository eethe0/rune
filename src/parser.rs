use crate::lexer::*;

type Iter<'a> = std::iter::Peekable<std::slice::Iter<'a, Token>>;

pub fn parse_module(tokens: &[Token]) -> Result<Module, ParseError> {
    Module::parse(&mut tokens.iter().peekable())
}

pub fn parse_single_decl(tokens: &[Token]) -> Result<Declaration, ParseError> {
    let mut iter = tokens.iter().peekable();
    let decl = Declaration::parse(&mut iter)?;
    if let Some(_) = iter.next() {
        Err(ParseError::ExpectingEndOfFile)
    } else {
        Ok(decl)
    }
}

pub fn parse_single_expr(tokens: &[Token]) -> Result<Expression, ParseError> {
    let mut iter = tokens.iter().peekable();
    let expr = Expression::parse(&mut iter)?;
    if let Some(_) = iter.next() {
        Err(ParseError::ExpectingEndOfFile)
    } else {
        Ok(expr)
    }
}

fn try_match(iter: &mut Iter, ty: TokenType) -> Result<Token, ParseError> {
    if let Some(token) = iter.peek() {
        if token.variant == ty {
            let r = Ok((*token).clone());
            iter.next();
            r
        } else {
            Err(ParseError::NoMatch)
        }
    } else {
        Err(ParseError::NoMatch)
    }
}

fn expect(iter: &mut Iter, ty: TokenType) -> Result<Token, ParseError> {
    if let Some(token) = iter.next() {
        if token.variant == ty {
            Ok(token.clone())
        } else {
            Err(ParseError::UnexpectedToken(token.clone(), ty))
        }
    } else {
        Err(ParseError::UnexpectedEndOfFile)
    }
}

#[derive(Debug)]
pub struct Module {
    pub declarations: Vec<Declaration>,
}

impl Module {
    fn parse(iter: &mut Iter) -> Result<Self, ParseError> {
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
                token.clone(),
                "declaration or end of file".to_owned(),
            ))
        } else {
            Ok(module)
        }
    }
}

#[derive(Debug)]
pub struct Declaration {
    pub identifier: String,
    pub initializer: Expression,
}

impl Declaration {
    fn parse(iter: &mut Iter) -> Result<Self, ParseError> {
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
pub enum Statement {
    DeclarationStatement(Declaration),
    ExpressionStatement(Expression),
    ReturnStatement(Expression),
}

#[derive(Debug, Clone)]
pub enum Expression {
    IdentifierExpression(String),
    NumberExpression(String),
    StringExpression(String),
    FunctionExpression(String, Box<Expression>),
    CallExpression(Box<Expression>, Box<Expression>),
    BinaryExpression(Operator, Box<Expression>, Box<Expression>),
    UnaryExpression(Operator, Box<Expression>),
    BlockExpression,
}

/// http://www.engr.mun.ca/~theo/Misc/exp_parsing.htm#climbing
impl Expression {
    fn parse(iter: &mut Iter) -> Result<Self, ParseError> {
        Self::exp(iter, 0)
    }

    fn exp(iter: &mut Iter, p: u8) -> Result<Self, ParseError> {
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

    fn p(iter: &mut Iter) -> Result<Self, ParseError> {
        if let Some(token) = iter.peek() {
            if let Some(op) = Operator::unary(&token.variant) {
                iter.next();
                let q = op as u8;
                Ok(Expression::UnaryExpression(
                    op,
                    Box::new(Self::exp(iter, q)?),
                ))
            } else if token.variant == TokenType::OpenParen {
                iter.next();
                let t = Self::exp(iter, 0)?;
                expect(iter, TokenType::CloseParen)?;
                Ok(t)
            } else {
                match token.variant {
                    TokenType::Identifier => {
                        let id = &token.value;
                        let mut t = Expression::IdentifierExpression(id.clone());
                        iter.next();
                        if let Ok(_) = try_match(iter, TokenType::OpFunction) {
                            t = Expression::FunctionExpression(
                                id.clone(),
                                Box::new(Expression::parse(iter)?),
                            );
                        }
                        Ok(t)
                    }
                    TokenType::NumberLiteral => {
                        let t = Expression::NumberExpression(token.value.clone());
                        iter.next();
                        Ok(t)
                    }
                    TokenType::StringLiteral => {
                        let t = Expression::StringExpression(token.value.clone());
                        iter.next();
                        Ok(t)
                    }
                    _ => Err(ParseError::UnexpectedTokenMultiple(
                        (*token).clone(),
                        vec![
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
pub enum ParseError {
    UnexpectedToken(Token, TokenType),
    UnexpectedTokenMultiple(Token, Vec<TokenType>),
    ExpectingOther(Token, String),
    ExpectingEndOfFile,
    UnexpectedEndOfFile,
    NoMatch,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
