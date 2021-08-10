use std::fmt::{ Debug, Formatter, Result };
use std::cmp::{ PartialEq, Ordering };
use std::rc::Rc;

use crate::common::Closure;

pub type BuiltIn = dyn Fn(Vec<&Value>) -> Value;

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
  fn fmt(&self, fmt: &mut Formatter<'_>) -> Result {
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

impl Debug for RustFunc {
  fn fmt(&self, fmt: &mut Formatter<'_>) -> Result {
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