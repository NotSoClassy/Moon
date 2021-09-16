use crate::common::{ Value, Table };
use crate::vm::{ VM, env::{ Env, aux::* } };
use crate::{ expect, optional };

pub fn load(env: &mut Env) {
  let tbl = Table::new();

  tbl_builtin(&tbl, "upper", &str_upper);
  tbl_builtin(&tbl, "lower", &str_lower);
  tbl_builtin(&tbl, "sub", &str_sub);

  env.set_global(Value::String("string".into()), Value::Table(tbl))
}

fn str_upper(vm: &mut VM) -> Result<Value, String> {
  let str = expect!(String, vm)?;

  Ok(Value::String(str.to_uppercase()))
}

fn str_lower(vm: &mut VM) -> Result<Value, String> {
  let str = expect!(String, vm)?;

  Ok(Value::String(str.to_lowercase()))
}

fn str_sub(vm: &mut VM) -> Result<Value, String> {
  let str = expect!(String, vm)?;
  let len = str.len();

  let mut start = expect!(Number, vm)? as usize;
  let mut end = optional!(Number, len as f64, vm) as usize;

  start = start.clamp(0, len);
  end = end.clamp(0, len);
  start = start.clamp(0, end);

  Ok(Value::String(str[start..end].to_string()))
}