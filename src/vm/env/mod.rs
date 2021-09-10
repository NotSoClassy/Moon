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
    self.globals.insert(idx, val);
  }

  pub fn load(&mut self) {
    globals::load(self);
    strlib::load(self);
  }

  #[inline]
  pub(super) fn builtin(&mut self, name: &str, func: &'static BuiltIn) {
    tbl_builtin(&self.globals, name, func)
  }
}