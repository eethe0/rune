// https://stackoverflow.com/questions/51770808/how-exactly-does-ios-work-under-the-hood
use crate::parser::*;
use std::collections::HashMap;

#[derive(Clone)]
pub enum Value {
    Number(i64),
    String(String),
    Function(Expression, String, Scope),
    Error(String),
}

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Value::Number(n) => write!(f, "Number({})", n),
            Value::String(s) => write!(f, "String({})", s),
            Value::Function(_, _, _) => write!(f, "Function"),
            Value::Error(e) => write!(f, "Error({})", e),
        }
    }
}

pub type Scope = HashMap<String, Value>;

pub fn eval_module(module: &Module) -> Scope {
    let mut scope = Scope::new();
    for decl in &module.declarations {
        eval_declaration(decl, &mut scope);
    }
    scope
}

pub fn eval_declaration(decl: &Declaration, scope: &mut Scope) -> Value {
    let val = eval_expression(&decl.initializer, scope.clone());
    scope
        .entry(decl.identifier.to_owned())
        .or_insert(val.clone());
    val
}

pub fn eval_expression(expr: &Expression, scope: Scope) -> Value {
    match expr {
        Expression::BinaryExpression(op, l, r) => {
            if let Value::Number(a) = eval_expression(l, scope.clone()) {
                if let Value::Number(b) = eval_expression(r, scope.clone()) {
                    match op {
                        Operator::Add => Value::Number(a + b),
                        Operator::Subtract => Value::Number(a - b),
                        Operator::Multiply => Value::Number(a * b),
                        Operator::Divide => Value::Number(a / b),
                        Operator::Modulo => Value::Number(a % b),
                        _ => Value::Error("no".to_owned()),
                    }
                } else {
                    Value::Error("Operation on non-number".to_owned())
                }
            } else {
                Value::Error("Operation on non-number".to_owned())
            }
        }
        Expression::BlockExpression => Value::Number(0),
        Expression::CallExpression(func, arg) => {
            if let Value::Function(body, argn, mut inner) = eval_expression(func, scope.clone()) {
                let argv = eval_expression(arg, scope.clone());
                inner.insert(argn, argv);
                eval_expression(&body, inner)
            } else {
                Value::Error("Call to non-function".to_owned())
            }
        }
        Expression::FunctionExpression(argn, body) => {
            Value::Function((**body).clone(), (*argn).to_owned(), scope)
        }
        Expression::IdentifierExpression(id) => {
            if let Some(val) = scope.get(&(*id).to_owned()) {
                val.clone()
            } else {
                Value::Error(format!("{} is not defined", id))
            }
        }
        Expression::NumberExpression(n) => Value::Number(n.parse().unwrap()),
        Expression::StringExpression(s) => Value::String((*s).to_owned()),
        Expression::UnaryExpression(op, v) => {
            if let Value::Number(n) = eval_expression(v, scope.clone()) {
                match op {
                    Operator::UnaryMinus => Value::Number(-n),
                    Operator::UnaryPlus => Value::Number(n),
                    _ => Value::Error("no".to_owned()),
                }
            } else {
                Value::Error("Operation on non-number".to_owned())
            }
        }
    }
}