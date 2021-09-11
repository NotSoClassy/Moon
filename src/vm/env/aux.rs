use std::rc::Rc;

use crate::common::{ Value, Table, BuiltIn, RustFunc };
use crate::vm::VM;

pub fn tbl_builtin(tbl: &Table, name: &str, func: &'static BuiltIn) {
  let rf = RustFunc {
    name: name.clone().into(),
    func: func
  };

  tbl.insert(Value::String(name.into()), Value::NativeFunc(Rc::new(rf))).unwrap();
}

pub fn get(vm: &mut VM) -> Result<Value, String> {
  if vm.nci.base > vm.nci.top {
    Err("expected value".into())
  } else {
    let pos = vm.nci.base;
    vm.nci.base += 1; // so this never gets read again
    Ok(vm.regs.get(pos as usize).unwrap_or(&Value::Nil).clone())
  }
}

pub fn get_all(vm: &mut VM) -> Vec<Value> {
  let mut vals = Vec::new();

  for i in vm.nci.base .. vm.nci.top {
    vals.push(vm.regs[i as usize].clone());
  }

  vals
}

#[macro_export]
macro_rules! get_all {
  ($vm:ident) => {
    crate::vm::env::aux::get_all($vm)
  };
}

#[macro_export]
macro_rules! expect {
  ($id:ident, $vm:ident) => {{
    let val = crate::vm::env::aux::get($vm)?;

    if let Value::$id(x) = val {
      Ok(x.clone())
    } else {
      Err(format!("expected {} got {:?}", stringify!($id).to_lowercase(), crate::common::Type::from(&val)))
    }}
  };
}

#[macro_export]
macro_rules! expect_any {
  ($vm:ident) => {
    crate::vm::env::aux::get($vm)?
  };
}

#[macro_export]
macro_rules! optional {
  ($id:ident, $or:expr, $vm:ident) => {{
    let val = crate::vm::env::aux::get($vm)?;

    if let Value::$id(x) = val {
      x.clone()
    } else {
      $or
    }}
  };
}