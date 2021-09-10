use std::rc::Rc;

use crate::common::{ Value, Closure, Type, Array, Table, BuiltIn, RustFunc };
use crate::vm::VM;

macro_rules! expect {
  ($id:ident, $val:ident) => {
    if let Value::$id(x) = $val {
      Ok(x.clone())
    } else {
      Err(format!("expected {} got {:?}", stringify!($id).to_lowercase(), Type::from(&$val)))
    }
  };
}

pub fn tbl_builtin(tbl: &Table, name: &str, func: &'static BuiltIn) {
  let rf = RustFunc {
    name: name.clone().into(),
    func: func
  };

  tbl.insert(Value::String(name.into()), Value::NativeFunc(Rc::new(rf)))
}

fn get(vm: &mut VM, i: usize) -> Result<Value, String> {
  if i > vm.nci.top {
    Err("expected value".into())
  } else {
    vm.nci.base += 1; // so this never gets read again
    Ok(vm.regs.get(i as usize).unwrap_or(&Value::Nil).clone())
  }
}

#[allow(dead_code)]
pub fn expect_string(vm: &mut VM) -> Result<String, String> {
  let v = get(vm, vm.nci.base)?;

  expect!(String, v)
}

#[allow(dead_code)]
pub fn expect_number(vm: &mut VM) -> Result<f64, String> {
  let v = get(vm, vm.nci.base)?;

  expect!(Number, v)
}

#[allow(dead_code)]
pub fn expect_bool(vm: &mut VM) -> Result<bool, String> {
  let v = get(vm, vm.nci.base)?;

  expect!(Bool, v)
}

#[allow(dead_code)]
pub fn expect_closure(vm: &mut VM) -> Result<Closure, String> {
  let v = get(vm, vm.nci.base)?;

  expect!(Closure, v)
}

#[allow(dead_code)]
pub fn expect_array(vm: &mut VM) -> Result<Array, String> {
  let v = get(vm, vm.nci.base)?;

  expect!(Array, v)
}

#[allow(dead_code)]
pub fn expect_table(vm: &mut VM) -> Result<Table, String> {
  let v = get(vm, vm.nci.base)?;

  expect!(Table, v)
}

#[inline]
pub fn expect_any(vm: &mut VM) -> Result<Value, String> {
  get(vm, vm.nci.base)
}

pub fn get_all(vm: &mut VM) -> Vec<Value> {
  let mut vals = Vec::new();

  for i in vm.nci.base .. vm.nci.top {
    vals.push(vm.regs[i as usize].clone());
  }

  vals
}
