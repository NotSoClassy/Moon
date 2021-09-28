use libc::{ RAND_MAX, rand, srand, c_uint };

use std::time::UNIX_EPOCH;
use std::f64::consts::PI;

use crate::vm::{ VM, env::{ Env, aux::* }, RuntimeError };
use crate::common::{ Value, Table };
use crate::{ expect, arg_check };

macro_rules! max_min {
  ($op:tt, $vm:ident) => {{
    let vm = $vm;

    let mut final_max = expect!(Number, vm)?;
    let narg = vm.nci.top - vm.nci.base;

    for _ in 0..narg {
      let n = expect!(Number, vm)?;

      if n $op final_max {
        final_max = n
      }
    }

    Ok(Value::Number(final_max))
  }};
}

macro_rules! method {
  ($id:ident, $vm:ident) => {{
    let n = expect!(Number, $vm)?;

    Ok(Value::Number(n.$id()))
  }};
}

macro_rules! method2 {
  ($id:ident, $vm:ident) => {{
    let n = expect!(Number, $vm)?;
    let n2 = expect!(Number, $vm)?;

    Ok(Value::Number(n.$id(n2)))
  }};
}

pub fn load(env: &mut Env) {
  let tbl = Table::new();

  unsafe { srand(UNIX_EPOCH.elapsed().unwrap_or_default().as_nanos() as c_uint) }

  tbl_builtin(&tbl, "randomseed", &math_randomseed);
  tbl_builtin(&tbl, "random", &math_random);
  tbl_builtin(&tbl, "floor", &math_floor);
  tbl_builtin(&tbl, "log10", &math_log10);
  tbl_builtin(&tbl, "sqrt", &math_sqrt);
  tbl_builtin(&tbl, "ceil", &math_ceil);
  tbl_builtin(&tbl, "log", &math_log);
  tbl_builtin(&tbl, "pow", &math_pow);
  tbl_builtin(&tbl, "max", &math_max);
  tbl_builtin(&tbl, "min", &math_min);
  tbl_builtin(&tbl, "abs", &math_abs);

  tbl.insert(Value::String("pi".into()), Value::Number(PI)).unwrap();
  tbl.insert(Value::String("huge".into()), Value::Number(f64::INFINITY)).unwrap();

  env.set_global(Value::String("math".into()), Value::Table(tbl))
}

fn math_floor(vm: &mut VM) -> Result<Value, RuntimeError> {
  method!(floor, vm)
}

fn math_sqrt(vm: &mut VM) -> Result<Value, RuntimeError> {
  method!(sqrt, vm)
}

fn math_abs(vm: &mut VM) -> Result<Value, RuntimeError> {
  method!(abs, vm)
}

fn math_ceil(vm: &mut VM) -> Result<Value, RuntimeError> {
  method!(ceil, vm)
}

fn math_log10(vm: &mut VM) -> Result<Value, RuntimeError> {
  method!(log10, vm)
}

fn math_log(vm: &mut VM) -> Result<Value, RuntimeError> {
  method2!(log, vm)
}

fn math_pow(vm: &mut VM) -> Result<Value, RuntimeError> {
  method2!(powf, vm)
}

fn math_max(vm: &mut VM) -> Result<Value, RuntimeError> {
  max_min!(>, vm)
}

fn math_min(vm: &mut VM) -> Result<Value, RuntimeError> {
  max_min!(<, vm)
}

fn math_random(vm: &mut VM) -> Result<Value, RuntimeError> {
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

fn math_randomseed(vm: &mut VM) -> Result<Value, RuntimeError> {
  let seed = expect!(Number, vm)?;

  unsafe { srand(seed as c_uint) }

  Ok(Value::Nil)
}