use std::time::{ SystemTime, UNIX_EPOCH };
use std::io::Write;
use std::rc::Rc;
use std::io;

use crate::common::{ Value, BuiltIn, RustFunc, type_value };
use crate::vm::VM;

fn new_func(vm: &mut VM, name: &str, func: &'static BuiltIn) {
  let rf = RustFunc {
    name: name.clone().into(),
    func
  };

  let nf = Value::NativeFunc(Rc::new(rf));

  vm.globals.insert(name.into(), nf);
}

pub fn load(vm: &mut VM) {
  new_func(vm, "print", &print);
  new_func(vm, "write", &write);
  new_func(vm, "clock", &clock);
  new_func(vm, "error", &error);
  new_func(vm, "read", &read);
  new_func(vm, "len", &len);
}

fn get(vm: &mut VM, i: u8) -> Result<Value, String> {
  if i > vm.nci.top {
    Err("expected value".into())
  } else {
    vm.nci.base += 1; // so this never gets read again
    Ok(vm.regs.get(i as usize).unwrap_or(&Value::Nil).clone())
  }
}

fn expect_string(vm: &mut VM) -> Result<String, String> {
  let v = get(vm, vm.nci.base)?;

  if let Value::String(s) = v {
    Ok(s.clone())
  } else {
    Err(format!("expected string got {}", type_value(v.clone())))
  }
}

#[inline(always)]
fn expect_any(vm: &mut VM) -> Result<Value, String> {
  get(vm, vm.nci.base)
}

fn get_all(vm: &mut VM) -> Vec<Value> {
  let mut vals = Vec::new();

  for i in vm.nci.base .. vm.nci.top {
    vals.push(vm.regs[i as usize].clone());
  }

  vals
}

fn len(vm: &mut VM) -> Result<Value, String> {
  let val = expect_any(vm)?;

  let n = match val.clone() {
    Value::Array(a) => Some(a.borrow().len()),
    Value::String(s) => Some(s.len()),

    _ => None
  };

  if let Some(n) = n {
    Ok(Value::Number(n as f64))
  } else {
    Err(format!("cannot get len on a {} value", type_value(val)))
  }
}

fn write(vm: &mut VM) -> Result<Value, String> {
  let vals = get_all(vm);
  let len = vals.len();

  for i in 0 .. len {
    print!("{:?}", vals[i]);
    if i < len - 1 { print!("\t") }
  }

  Ok(Value::Nil)
}

fn print(vm: &mut VM) -> Result<Value, String> {
  write(vm)?;
  print!("\n");
  Ok(Value::Nil)
}

fn clock(_vm: &mut VM) -> Result<Value, String> {
  let now = SystemTime::now();
  let time = now
    .duration_since(UNIX_EPOCH)
    .expect("Time went backwards")
    .as_secs_f64();
  Ok(Value::Number(time))
}

fn error(vm: &mut VM) -> Result<Value, String> {
  let s = expect_string(vm)?;
  Err(s)
}

fn read(_vm: &mut VM) -> Result<Value, String> {
  let res = io::stdout().flush();

  if res.is_err() {
    return Err("unable to write to stdout".into())
  }

  let mut str = String::new();
  let res = io::stdin().read_line(&mut str);

  if res.is_err() {
    Err("unable to read from stdin".into())
  } else {
    Ok(Value::String(str.trim().to_string()))
  }
}