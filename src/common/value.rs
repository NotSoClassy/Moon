use std::fmt::{ Debug, Formatter, Result as FmtResult };
use std::cmp::{ PartialEq, Ordering };
use std::ops::{ Add, Sub, Div, Mul };
use std::rc::Rc;

use crate::common::Closure;

pub type BuiltIn = dyn Fn(Vec<&Value>) -> Result<Value, String>;

pub struct RustFunc {
  pub name: String,
  pub func: &'static BuiltIn
}

#[derive(Clone, PartialEq, PartialOrd)]
pub enum Value {
  String(String),
  Number(f64),
  Bool(bool),
  Closure(Closure),
  NativeFunc(Rc<RustFunc>),
  Nil
}

impl Debug for Value {
  fn fmt(&self, fmt: &mut Formatter<'_>) -> FmtResult {
    match self {
      Value::String(s) => write!(fmt, "{}", s),
      Value::Number(n) => write!(fmt, "{}", n),
      Value::Bool(b) => write!(fmt, "{}", b),
      Value::Closure(c) => write!(fmt, "function: {}", c.name),
      Value::NativeFunc(rf) => write!(fmt, "function: {}", rf.name),
      Value::Nil => write!(fmt, "nil")
    }
  }
}

pub fn type_value(val: Value) -> String {
  let s = match val {
    Value::NativeFunc(..) | Value::Closure(..) => "function",
    Value::Number(..) => "number",
    Value::String(..) => "string",
    Value::Bool(..) => "bool",
    Value::Nil => "nil"
  };

  s.to_string()
}

impl Add for Value {
  fn add(self, rhs: Value) -> Result<Value, String> {
    match (self.clone(), rhs.clone()) {
      (Value::Number(lhs), Value::Number(rhs)) => Ok(Value::Number(lhs + rhs)),
      (Value::String(lhs), Value::String(rhs)) => Ok(Value::String(lhs + &rhs)),

      _ => Err(format!("cannot add {} and {}", type_value(self), type_value(rhs)))
    }
  }

  type Output = Result<Value, String>;
}

impl Sub for Value {
  fn sub(self, rhs: Value) -> Result<Value, String> {
    match (self.clone(), rhs.clone()) {
      (Value::Number(lhs), Value::Number(rhs)) => Ok(Value::Number(lhs - rhs)),

      _ => Err(format!("cannot sub {} and {}", type_value(self), type_value(rhs)))
    }
  }

  type Output = Result<Value, String>;
}

impl Mul for Value {
  fn mul(self, rhs: Value) -> Result<Value, String> {
    match (self.clone(), rhs.clone()) {
      (Value::Number(lhs), Value::Number(rhs)) => Ok(Value::Number(lhs * rhs)),

      _ => Err(format!("cannot multiply {} and {}", type_value(self), type_value(rhs)))
    }
  }

  type Output = Result<Value, String>;
}

impl Div for Value {
  fn div(self, rhs: Value) -> Result<Value, String> {
    match (self.clone(), rhs.clone()) {
      (Value::Number(lhs), Value::Number(rhs)) => Ok(Value::Number(lhs / rhs)),

      _ => Err(format!("cannot divide {} and {}", type_value(self), type_value(rhs)))
    }
  }

  type Output = Result<Value, String>;
}

impl Debug for RustFunc {
  fn fmt(&self, fmt: &mut Formatter<'_>) -> FmtResult {
    write!(fmt, "function: {}", self.name)
  }
}

impl PartialEq for RustFunc {
  fn eq(&self, rhs: &RustFunc) -> bool {
    let self_ptr = self as *const RustFunc;
    let rhs_ptr = rhs as *const RustFunc;
    self_ptr == rhs_ptr
  }
}

impl PartialOrd for Closure {
  fn partial_cmp(&self, _: &Closure) -> Option<Ordering> {
    None
  }
}

impl PartialOrd for RustFunc {
  fn partial_cmp(&self, _: &RustFunc) -> Option<Ordering> {
    None
  }
}