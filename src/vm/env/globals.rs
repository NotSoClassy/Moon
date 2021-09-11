use std::time::{ SystemTime, UNIX_EPOCH };
use std::io::Write;
use std::io;

use crate::common::{ Value, Type, utils::compile_file };
use crate::vm::{ VM, env::Env };
use crate::{ expect, expect_any, get_all };

pub fn load(env: &mut Env) {
  env.builtin("require", &require);
  env.builtin("print", &print);
  env.builtin("write", &write);
  env.builtin("clock", &clock);
  env.builtin("error", &error);
  env.builtin("read", &read);
  env.builtin("len", &len);
}

fn require(vm: &mut VM) -> Result<Value, String> {
  let path = expect!(String, vm)?;
  let closure = compile_file(path)?;

  vm.run_closure(closure)?;

  Ok(Value::Nil)
}

fn len(vm: &mut VM) -> Result<Value, String> {
  let val = expect_any!(vm);

  let n = match val.clone() {
    Value::Array(a) => Some(a.len()),
    Value::Table(t) => Some(t.len()),
    Value::String(s) => Some(s.len()),

    _ => None
  };

  if let Some(n) = n {
    Ok(Value::Number(n as f64))
  } else {
    Err(format!("cannot get len on a {:?} value", Type::from(&val)))
  }
}

fn write(vm: &mut VM) -> Result<Value, String> {
  let vals = get_all!(vm);
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
  let s = expect!(String, vm)?;
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