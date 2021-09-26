use std::fmt::{ Debug, Formatter, Result as FmtResult };
use std::ops::{ Add, Sub, Div, Mul, Rem };
use std::cmp::{ PartialEq, Ordering };
use std::hash::{ Hash, Hasher };
use std::rc::Rc;

use crate::common::{ Closure, Array, Table };
use crate::vm::{ VM, RuntimeError };

pub type BuiltIn = dyn Fn(&mut VM) -> Result<Value, RuntimeError>;

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

impl Value {
  pub fn to_string(&self) -> String {
    match self {
      Value::String(s) => s.to_string(),
      Value::Number(n) => n.to_string(),
      Value::Bool(b) => b.to_string(),
      Value::Closure(c) => format!("function: {}", c.name),
      Value::NativeFunc(rf) => format!("function: {}", rf.name),
      Value::Array(array) => format!("{:?}", array.vec.borrow()),
      Value::Table(t) => format!("{:?}", t.tbl.borrow()),
      Value::Nil => "nil".into()
    }
  }
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
    write!(fmt, "{}", self.to_string())
  }
}

impl Type {
  pub fn to_string(&self) -> String {
    let str = match self {
      Type::String => "string",
      Type::Number => "number",
      Type::Bool => "bool",
      Type::Function => "function",
      Type::Array => "array",
      Type::Table => "table",
      Type::Nil => "nil"
    };

    str.into()
  }
}

impl Debug for Type {
  fn fmt(&self, fmt: &mut Formatter<'_>) -> FmtResult {
    write!(fmt, "{}", self.to_string())
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
      (Value::String(lhs), rhs @ _) => Ok(Value::String(lhs + rhs.to_string().as_str())),
      (lhs @ _, Value::String(rhs)) => Ok(Value::String(lhs.to_string().as_str().to_owned() + &rhs)),

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

impl Rem for Value {
  fn rem(self, rhs: Value) -> Result<Value, ()> {
    match (self.clone(), rhs.clone()) {
      (Value::Number(lhs), Value::Number(rhs)) => Ok(Value::Number(lhs % rhs)),

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