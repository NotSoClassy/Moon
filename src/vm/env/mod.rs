use crate::common::{ Value, BuiltIn, Table };
use aux::tbl_builtin;

mod globals;
mod strlib;
mod aux;

pub struct Env {
  pub globals: Table
}

impl Env {
  pub fn new() -> Self {
    Env {
      globals: Table::new()
    }
  }

  pub fn set_global(&mut self, idx: Value, val: Value) {
    self.globals.insert(idx, val).unwrap();
  }

  pub fn load(&mut self) {
    globals::load(self);
    strlib::load(self);

    self.set_global(Value::String("_G".into()), Value::Table(self.globals.clone()))
  }

  #[inline]
  pub(super) fn builtin(&mut self, name: &str, func: &'static BuiltIn) {
    tbl_builtin(&self.globals, name, func)
  }
}