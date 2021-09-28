use std::time::{ SystemTime, UNIX_EPOCH };
use std::io::Write;
use std::io;

use crate::common::{ Value, Type };
use crate::vm::{ VM, env::{ Env, aux::try_get }, RuntimeError };
use crate::{ expect, expect_any, get_all, optional };

pub fn load(env: &mut Env) {
  env.builtin("tonumber", &tonumber);
  env.builtin("argcheck", &argcheck);
  env.builtin("print", &print);
  env.builtin("write", &write);
  env.builtin("clock", &clock);
  env.builtin("error", &error);
  env.builtin("read", &read);
  env.builtin("next", &next);
  env.builtin("type", &gettype);
  env.builtin("len", &len);
}

fn gettype(vm: &mut VM) -> Result<Value, RuntimeError> {
  Ok(Value::String(Type::from(&expect_any!(vm)).to_string()))
}

fn argcheck(vm: &mut VM) -> Result<Value, RuntimeError> {
  let call = vm.call();
  let nparams = call.closure.nparams.wrapping_sub(1).into();

  let mut args = Vec::with_capacity(nparams);
  let mut n = 1;

  for i in call.base ..= call.base + nparams {
    args.push(vm.regs.get(i).unwrap_or(&Value::Nil).clone())
  }

  while let Some(val) = try_get(vm) {
    let t = Type::from(&val);
    let expected_type;

    if t != Type::String {
      return Err(format!("expected string got {:?}", t).into())
    } else {
      if let Value::String(str) = val {
        expected_type = str;
      } else {
        panic!("this is impossible");
      }
    }

    let arg = args.remove(0);
    let arg_type = Type::from(&arg).to_string();

    if arg_type != expected_type {
      return Err(format!("bad argument #{}: expected {} got {}", n, expected_type, arg_type).into())
    }

    n += 1;
  }

  Ok(Value::Nil)
}

fn tonumber(vm: &mut VM) -> Result<Value, RuntimeError> {
  let val = expect_any!(vm);
  let base = (optional!(Number, 10.0, vm) as u32).clamp(2, 32);

  match val {
    Value::String(s) => {
      let n = u64::from_str_radix(s.as_str(), base);

      Ok(if let Ok(n) = n {
        Value::Number(n as f64)
      } else {
        Value::Nil
      })
    }

    _ => Ok(Value::Nil)
  }
}

fn next(vm: &mut VM) -> Result<Value, RuntimeError> {
  let val = expect_any!(vm);
  let idx = optional!(Number, 0.0, vm) as usize;

  match val {
    Value::Table(t) => {
      let tbl = t.tbl.borrow();
      let (_, val) = tbl.iter().nth(idx).unwrap_or((&Value::Nil, &Value::Nil));
      Ok(val.clone())
    }

    Value::Array(a) => {
      let v = a.get(&Value::Number(idx as f64))?;

      Ok(v.clone())
    }

    _ => Err("bad argument #1".into())
  }
}

fn len(vm: &mut VM) -> Result<Value, RuntimeError> {
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
    Err(RuntimeError::TypeError("get len".into(), Type::from(&val), None))
  }
}

fn write(vm: &mut VM) -> Result<Value, RuntimeError> {
  let vals = get_all!(vm);
  let len = vals.len();

  for i in 0 .. len {
    print!("{:?}", vals[i]);
    if i < len - 1 { print!("\t") }
  }

  Ok(Value::Nil)
}

fn print(vm: &mut VM) -> Result<Value, RuntimeError> {
  write(vm)?;
  print!("\n");
  Ok(Value::Nil)
}

fn clock(_vm: &mut VM) -> Result<Value, RuntimeError> {
  let now = SystemTime::now();
  let time = now
    .duration_since(UNIX_EPOCH)
    .expect("Time went backwards")
    .as_secs_f64();
  Ok(Value::Number(time))
}

fn error(vm: &mut VM) -> Result<Value, RuntimeError> {
  let s = expect!(String, vm)?;
  Err(s.into())
}

fn read(_vm: &mut VM) -> Result<Value, RuntimeError> {
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