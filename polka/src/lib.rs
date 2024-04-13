#![forbid(unsafe_code)]

use std::{collections::HashMap, fmt::Display};

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Number(f64),
    Symbol(String),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Number(num) => write!(f, "{}", num),
            Self::Symbol(sym) => write!(f, "'{}", sym),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct Interpreter {
    stack: Vec<Value>,
    variables: HashMap<String, Value>,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            variables: HashMap::new(),
        }
    }

    pub fn eval(&mut self, expr: &str) {
        let tokens: Vec<&str> = expr.split_whitespace().collect();
        let mut op1;
        let mut op2;
        for s in tokens {
            if let Ok(x) = s.parse::<f64>() {
                self.stack.push(Value::Number(x));
                continue;
            }
            if s == "+" {
                let v1 = self.stack.pop();
                let v2 = self.stack.pop();
                op1 = self.check_operand(v1);
                op2 = self.check_operand(v2);
                self.stack.push(Value::Number(op1 + op2));
                continue;
            }
            if s == "-" {
                let v1 = self.stack.pop();
                let v2 = self.stack.pop();
                op1 = self.check_operand(v1);
                op2 = self.check_operand(v2);
                self.stack.push(Value::Number(op1 - op2));
                continue;
            }
            if s == "*" {
                let v1 = self.stack.pop();
                let v2 = self.stack.pop();
                op1 = self.check_operand(v1);
                op2 = self.check_operand(v2);
                self.stack.push(Value::Number(op1 * op2));
                continue;
            }
            if s == "/" {
                let v1 = self.stack.pop();
                let v2 = self.stack.pop();
                op1 = self.check_operand(v1);
                op2 = self.check_operand(v2);
                self.stack.push(Value::Number(op1 / op2));
                continue;
            }
            if let Some(y) = s.strip_prefix('\'') {
                self.stack.push(Value::Symbol(y.to_string()));
                continue;
            }
            if s == "set" {
                if let Some(Value::Symbol(x)) = self.stack.pop() {
                    if let Some(v) = self.stack.pop() {
                        self.variables.insert(x, v);
                        continue;
                    }
                    panic!("AAA");
                }
                panic!("AAA");
            }
            if let Some(y) = s.strip_prefix('$') {
                if let Some(x) = self.variables.get(&y.to_string()) {
                    self.stack.push(x.clone());
                    continue;
                }
                panic!("AAA");
            }
            panic!("Incorrect operation");
        }
    }

    fn check_operand(&self, operand: Option<Value>) -> f64 {
        match operand {
            Some(Value::Number(x)) => x,
            Some(Value::Symbol(x)) => match self.variables.get(&x) {
                Some(Value::Number(y)) => *y,
                _ => panic!("Unknown variable"),
            },
            _ => panic!("Incorrect operand"),
        }
    }

    pub fn stack(&self) -> &[Value] {
        &self.stack[..]
    }
}
