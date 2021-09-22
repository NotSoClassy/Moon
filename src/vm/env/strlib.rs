use crate::common::{ Value, Table, Array };
use crate::vm::{ VM, env::{ Env, aux::* }, RuntimeError };
use crate::{ expect, optional };

pub fn load(env: &mut Env) {
  let tbl = Table::new();

  tbl_builtin(&tbl, "upper", &str_upper);
  tbl_builtin(&tbl, "lower", &str_lower);
  tbl_builtin(&tbl, "split", &str_split);
  tbl_builtin(&tbl, "trim", &str_trim);
  tbl_builtin(&tbl, "byte", &str_byte);
  tbl_builtin(&tbl, "sub", &str_sub);

  env.set_global(Value::String("string".into()), Value::Table(tbl))
}

fn str_upper(vm: &mut VM) -> Result<Value, RuntimeError> {
  let str = expect!(String, vm)?;

  Ok(Value::String(str.to_uppercase()))
}

fn str_lower(vm: &mut VM) -> Result<Value, RuntimeError> {
  let str = expect!(String, vm)?;

  Ok(Value::String(str.to_lowercase()))
}

fn str_split(vm: &mut VM) -> Result<Value, RuntimeError> {
  let str = expect!(String, vm)?;
  let pat = optional!(String, " ".to_string(), vm);
  let mut vals = Vec::new();

  for found in str.split(&pat) {
    vals.push(Value::String(found.to_string()))
  }

  Ok(Value::Array(Array::new(vals)))
}

fn str_trim(vm: &mut VM) -> Result<Value, RuntimeError> {
  let str = expect!(String, vm)?;

  Ok(Value::String(str.trim().to_string()))
}

fn str_byte(vm: &mut VM) -> Result<Value, RuntimeError> {
  let str = expect!(String, vm)?;
  let pos = optional!(Number, 0.0, vm) as usize;

  Ok(Value::Number(str.bytes().nth(pos).unwrap_or(b'\0').into()))
}

fn str_sub(vm: &mut VM) -> Result<Value, RuntimeError> {
  let str = expect!(String, vm)?;
  let len = str.len();

  let mut start = expect!(Number, vm)? as usize;
  let mut end = optional!(Number, len as f64, vm) as usize;

  start = start.clamp(0, len);
  end = end.clamp(0, len);
  start = start.clamp(0, end);

  Ok(Value::String(str[start..end].to_string()))
}