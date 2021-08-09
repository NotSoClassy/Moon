use std::rc::Rc;

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
}

fn print(vals: Vec<&Value>) -> Value {
  for val in vals {
    print!("{:?}\t", val);
  }
  println!("");
  Value::Nil
}