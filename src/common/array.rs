use std::fmt::{ Debug, Formatter, Result as FmtResult };
use std::hash::{ Hash, Hasher };
use std::cell::RefCell;
use std::rc::Rc;

use crate::vm::error::RuntimeError;
use crate::common::{ Value, Type };

#[derive(Clone, PartialEq, PartialOrd)]
pub struct Array {
  pub vec: Rc<RefCell<Vec<Value>>>
}

impl Array {
  pub fn new(vec: Vec<Value>) -> Self {
    Array {
      vec: Rc::new(RefCell::new(vec))
    }
  }

  pub fn validate_index(&self, idx: &Value) -> Result<usize, RuntimeError> {
    if let Value::Number(idx) = idx {
      let idx = *idx;

      if idx.floor() == idx {
        if idx.is_sign_positive() {
          Ok(idx as usize)
        } else {
          Err(RuntimeError::ArrayIdxNeg)
        }
      } else {
        Err(RuntimeError::ArrayIdxFloat)
      }
    } else {
      Err(RuntimeError::TypeError("index an array with".into(), Type::from(idx), None))
    }
  }

  pub fn insert(&self, idx: &Value, val: Value) -> Result<(), RuntimeError> {
    let idx = self.validate_index(idx)?;

    let mut vec = self.vec.borrow_mut();
    let len = vec.len();

    if idx == len {
      vec.push(val);
    } else {
      if let Some(v) = vec.get_mut(idx) {
        *v = val;
      } else {
        return Err(RuntimeError::ArrayIdxBound)
      }
    }

    Ok(())
  }

  #[inline]
  pub fn get(&self, idx: &Value) -> Result<Value, RuntimeError> {
    let idx = self.validate_index(idx)?;

    Ok(self.vec.borrow().get(idx).unwrap_or(&Value::Nil).clone())
  }

  #[inline]
  pub fn len(&self) -> usize {
    self.vec.borrow().len()
  }
}

impl Debug for Array {
  fn fmt(&self, fmt: &mut Formatter<'_>) -> FmtResult {
    write!(fmt, "{:?}", self.vec)
  }
}

impl Hash for Array {
  fn hash<H>(&self, state: &mut H) where H: Hasher {
    Rc::as_ptr(&self.vec).hash(state)
  }
}