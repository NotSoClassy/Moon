use std::fmt::{ Debug, Formatter, Result as FmtResult };
use std::cmp::{ PartialEq, Ordering };
use std::hash::{ Hash, Hasher };
use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

use crate::vm::error::RuntimeError;
use crate::common::Value;

#[derive(Clone, PartialEq)]
pub struct Table {
  pub tbl: Rc<RefCell<HashMap<Value, Value>>>
}

impl Table {
  pub fn new() -> Self {
    Table {
      tbl: Rc::new(RefCell::new(HashMap::new()))
    }
  }

  pub fn validate_index(&self, idx: &Value) -> Result<(), RuntimeError> {
    if let Value::Nil = idx {
      return Err(RuntimeError::TableIdxNil)
    }

    if let Value::Number(n) = idx {
      if n.is_nan() {
        return Err(RuntimeError::TableIdxNaN)
      }
    }

    Ok(())
  }

  #[inline]
  pub fn get(&self, idx: &Value) -> Result<Value, RuntimeError> {
    self.validate_index(idx)?;

    Ok(self.tbl.borrow().get(idx).unwrap_or(&Value::Nil).clone())
  }

  #[inline]
  pub fn insert(&self, idx: Value, val: Value) -> Result<(), RuntimeError> {
    self.validate_index(&idx)?;

    self.tbl.borrow_mut().insert(idx, val);
    Ok(())
  }

  #[inline]
  pub fn len(&self) -> usize {
    self.tbl.borrow().len()
  }
}

impl Debug for Table {
  fn fmt(&self, fmt: &mut Formatter<'_>) -> FmtResult {
    write!(fmt, "{:?}", self.tbl.borrow())
  }
}

impl Hash for Table {
  fn hash<H>(&self, state: &mut H) where H: Hasher {
    Rc::as_ptr(&self.tbl).hash(state)
  }
}

impl PartialOrd for Table {
  fn partial_cmp(&self, _other: &Table) -> Option<Ordering> {
    None
  }
}