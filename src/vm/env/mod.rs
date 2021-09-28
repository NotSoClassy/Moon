use crate::common::{ Value, BuiltIn, Table };
use aux::tbl_builtin;

pub mod aux;
mod globals;
mod mathlib;
mod require;
mod strlib;

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
    mathlib::load(self);

    self.builtin("require", &require::require);
    self.set_global(Value::String("_G".into()), Value::Table(self.globals.clone()))
  }

  #[inline]
  pub fn builtin(&mut self, name: &str, func: &'static BuiltIn) {
    tbl_builtin(&self.globals, name, func)
  }
}