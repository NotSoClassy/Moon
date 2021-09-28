use libc::{ RTLD_LAZY, RTLD_LOCAL, dlopen, dlerror, dlsym, c_void};

use std::ffi::{ CStr, CString };
use std::mem::transmute;

use crate::common::{ Value, utils::compile_file };
use crate::vm::{ VM, RuntimeError };
use crate::expect;

const LOAD_FUNCTION: &str = "moon_load_lib";

unsafe fn load_from_dlib(vm: &mut VM, path: String) -> Result<Value, RuntimeError> {
  macro_rules! resolve {
    ($id:ident) => {{
      if ($id).is_null() {
        let err_str = CStr::from_ptr(dlerror()).to_str().unwrap();

        return Err(err_str.into())
      }
    }}
  }

  let lib_name = CString::new(path).unwrap();
  let handle: *mut c_void = dlopen(
    lib_name.as_ptr(),
    RTLD_LAZY | RTLD_LOCAL
  );

  resolve!(handle);

  let func_name = CString::new(LOAD_FUNCTION).unwrap();
  let func = dlsym(handle, func_name.as_ptr());

  resolve!(func);

  let func = transmute::<
    *mut c_void,
    Option<unsafe fn(vm: &mut VM) -> Value>
  >(func);

  if func.is_none() {
    let str = format!("\"{}\" is a null function pointer", LOAD_FUNCTION);
    return Err(str.into())
  }

  let ret = func.unwrap()(vm);

  Ok(ret)
}

pub fn require(vm: &mut VM) -> Result<Value, RuntimeError> {
  let path = expect!(String, vm)?;

  if path.starts_with('@') {
    // load dylib
    let path = path[1..].to_string();

    #[cfg(windows)]
    panic!("loading dynamic libraries is not supported for windows!");

    unsafe { load_from_dlib(vm, path) }
  } else {
    // normal file
    let closure = compile_file(path)?;

    vm.run_closure(closure)?;

    Ok(Value::Nil)
  }
}