use std::os::raw::{ c_int, c_uint };
use std::time::UNIX_EPOCH;

use crate::common::{ Value, Table };
use crate::vm::{ VM, env::{ Env, aux::* } };
use crate::{ expect, arg_check };

const RAND_MAX: c_int = c_int::MAX;

macro_rules! method {
  ($id:ident, $vm:ident) => {{
    let n = expect!(Number, $vm)?;

    Ok(Value::Number(n.$id()))
  }};
}

extern "C" {
  fn srand(seed: c_uint);
  fn rand() -> c_int;
}

pub fn load(env: &mut Env) {
  let tbl = Table::new();

  unsafe { srand(UNIX_EPOCH.elapsed().unwrap().as_nanos() as c_uint) }

  tbl_builtin(&tbl, "randomseed", &randomseed);
  tbl_builtin(&tbl, "random", &random);
  tbl_builtin(&tbl, "floor", &floor);
  tbl_builtin(&tbl, "sqrt", &sqrt);
  tbl_builtin(&tbl, "abs", &abs);

  env.set_global(Value::String("math".into()), Value::Table(tbl))
}

fn floor(vm: &mut VM) -> Result<Value, String> {
  method!(floor, vm)
}

fn sqrt(vm: &mut VM) -> Result<Value, String> {
  method!(sqrt, vm)
}

fn abs(vm: &mut VM) -> Result<Value, String> {
  method!(abs, vm)
}

fn random(vm: &mut VM) -> Result<Value, String> {
  let r = unsafe { (rand() % RAND_MAX ) as f64 / RAND_MAX as f64 };

  match vm.nci.top - vm.nci.base {

    0 => Ok(Value::Number(r)),

    1 => {
      let u = expect!(Number, vm)?;

      arg_check!(1.0 <= u, 1, "interval is empty")?;

      Ok(Value::Number((r * u).floor() + 1.0))
    }

    2 => {
      let l = expect!(Number, vm)?;
      let u = expect!(Number, vm)?;

      arg_check!(l <= u, 2, "interval is empty")?;

      Ok(Value::Number((r * (u - l + 1.0)).floor() + l))
    }

    _ => Err("wrong number of arguments".into())
  }
}

fn randomseed(vm: &mut VM) -> Result<Value, String> {
  let seed = expect!(Number, vm)?;

  unsafe { srand(seed as c_uint) }

  Ok(Value::Nil)
}