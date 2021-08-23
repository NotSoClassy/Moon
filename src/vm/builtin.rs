use std::rc::Rc;
use std::io;
use std::io::Write;

use crate::common::{ Value, BuiltIn, RustFunc };
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
  new_func(vm, "read", &read)
}

fn read(_vals: Vec<&Value>) -> Result<Value, String> {
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

fn write(vals: Vec<&Value>) -> Result<Value, String> {
  let len = vals.len() - 1;

  for i in 0 ..= len {
    print!("{:?}", vals[i]);
    if i < len { print!("\t") }
  }

  Ok(Value::Nil)
}

fn print(vals: Vec<&Value>) -> Result<Value, String> {
  write(vals)?;
  print!("\n");

  Ok(Value::Nil)
}