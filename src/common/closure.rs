use crate::common::Value;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct Closure {
  pub name: String,
  pub file_name: String,
  pub lines: Vec<usize>,
  pub upvals: Vec<(u8, Rc<RefCell<Value>>)>,
  pub code: Vec<u32>,
  pub consts: Vec<Value>,
  pub nparams: u8
}

impl Closure {
  pub fn new(file_name: String) -> Self {
    let mut c = Closure {
      name: String::new(),
      file_name,
      lines: Vec::new(),
      upvals: Vec::new(),
      code: Vec::new(),
      consts: Vec::new(),
      nparams: 0
    };

    c.name = format!("{:?}", &c as *const Closure);

    c
  }
}