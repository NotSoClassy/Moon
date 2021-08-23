use crate::common::Value;
//use std::cell::RefCell;
//use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct Closure {
  pub name: String,
  pub lines: Vec<usize>,
  //pub upvals: Vec<Rc<RefCell<Value>>>,
  pub code: Vec<u32>,
  pub consts: Vec<Value>,
  pub nparams: u8
}

impl Closure {
  pub fn new() -> Self {
    Closure {
      name: "0".into(),
      lines: Vec::new(),
      //upvals: Vec::new(),
      code: Vec::new(),
      consts: Vec::new(),
      nparams: 0
    }
  }
}