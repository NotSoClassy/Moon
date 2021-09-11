use std::fmt::{ Debug, Formatter, Result as FmtResult };
use std::cmp::{ PartialEq, Ordering };
use std::ops::{ Add, Sub, Div, Mul };
use std::hash::{ Hash, Hasher };
use std::rc::Rc;

use crate::common::{ Closure, Array, Table };
use crate::vm::VM;

pub type BuiltIn = dyn Fn(&mut VM) -> Result<Value, String>;

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
  Array(Array),
  Table(Table),
  Nil
}

#[derive(PartialEq)]
pub enum Type {
  String,
  Number,
  Bool,
  Function,
  Array,
  Table,
  Nil
}

impl Eq for Value {}

impl Hash for Value {
  fn hash<H>(&self, state: &mut H) where H: Hasher {
    match self {
      Value::String(s) => s.hash(state),
      Value::Number(n) => n.to_bits().hash(state), // make sure `n != NaN`
      Value::Bool(b) => b.hash(state),

      Value::Array(a) => a.hash(state),
      Value::Table(t) => t.hash(state),
      Value::Closure(c) => (c as *const Closure).hash(state),
      Value::NativeFunc(nf) => Rc::as_ptr(nf).hash(state),

      Value::Nil => panic!("cannot hash nil")
    }
  }
}

impl Debug for Value {
  fn fmt(&self, fmt: &mut Formatter<'_>) -> FmtResult {
    match self {
      Value::String(s) => write!(fmt, "{}", s),
      Value::Number(n) => write!(fmt, "{}", n),
      Value::Bool(b) => write!(fmt, "{}", b),
      Value::Closure(c) => write!(fmt, "function: {}", c.name),
      Value::NativeFunc(rf) => write!(fmt, "function: {}", rf.name),
      Value::Array(array) => write!(fmt, "{:?}", array),
      Value::Table(t) => write!(fmt, "{:?}", t),
      Value::Nil => write!(fmt, "nil")
    }
  }
}

impl Debug for Type {
  fn fmt(&self, fmt: &mut Formatter<'_>) -> FmtResult {
    let str = match self {
      Type::String => "string",
      Type::Number => "number",
      Type::Bool => "bool",
      Type::Function => "function",
      Type::Array => "array",
      Type::Table => "table",
      Type::Nil => "nil"
    };

    write!(fmt, "{}", str)
  }
}

impl From<&Value> for Type {
  fn from(val: &Value) -> Self {
    match val {
      Value::NativeFunc(..) | Value::Closure(..) => Type::Function,
      Value::String(..) => Type::String,
      Value::Number(..) => Type::Number,
      Value::Bool(..) => Type::Bool,
      Value::Array(..) => Type::Array,
      Value::Table(..) => Type::Table,
      Value::Nil => Type::Nil
    }
  }
}

impl Add for Value {
  fn add(self, rhs: Value) -> Result<Value, ()> {
    match (self.clone(), rhs.clone()) {
      (Value::Number(lhs), Value::Number(rhs)) => Ok(Value::Number(lhs + rhs)),
      (Value::String(lhs), Value::String(rhs)) => Ok(Value::String(lhs + &rhs)),

      _ => Err(())
    }
  }

  type Output = Result<Value, ()>;
}

impl Sub for Value {
  fn sub(self, rhs: Value) -> Result<Value, ()> {
    match (self.clone(), rhs.clone()) {
      (Value::Number(lhs), Value::Number(rhs)) => Ok(Value::Number(lhs - rhs)),

      _ => Err(())
    }
  }

  type Output = Result<Value, ()>;
}

impl Mul for Value {
  fn mul(self, rhs: Value) -> Result<Value, ()> {
    match (self.clone(), rhs.clone()) {
      (Value::Number(lhs), Value::Number(rhs)) => Ok(Value::Number(lhs * rhs)),

      _ => Err(())
    }
  }

  type Output = Result<Value, ()>;
}

impl Div for Value {
  fn div(self, rhs: Value) -> Result<Value, ()> {
    match (self.clone(), rhs.clone()) {
      (Value::Number(lhs), Value::Number(rhs)) => Ok(Value::Number(lhs / rhs)),

      _ => Err(())
    }
  }

  type Output = Result<Value, ()>;
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