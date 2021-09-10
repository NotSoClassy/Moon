use crate::common::{ Value, Table };
use crate::vm::{ VM, env::{ Env, aux::* } };

pub fn load(env: &mut Env) {
  let tbl = Table::new();

  tbl_builtin(&tbl, "upper", &str_upper);

  env.set_global(Value::String("string".into()), Value::Table(tbl))
}

fn str_upper(vm: &mut VM) -> Result<Value, String> {
  let str = expect_string(vm)?;

  Ok(Value::String(str.to_uppercase()))
}