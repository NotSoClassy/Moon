use crate::parser::{ Parser, gen::Compiler };
use crate::common::Closure;
use crate::vm::VM;

use std::io::prelude::*;
use std::fs::File;

pub fn compile_file(name: String) -> Result<Closure, String> {
  let mut file = File::open(name.clone()).expect("could not open file");
  let mut str = String::new();

  file.read_to_string(&mut str).expect("failure to read file");

  if str == "" {
    let mut c = Closure::new(name);
    c.name = "main".into();
    return Ok(c)
  }

  let mut parser = Parser::new(str.into(), name.clone());
  parser.parse()?;

  let mut compiler = Compiler::new(name.clone());
  compiler.compile(parser.nodes)?;
  compiler.closure.name = "main".into();

  Ok(compiler.closure)
}

pub fn do_file(name: String) -> Result<(), String> {
  let closure = compile_file(name)?;

  let mut vm = VM::new(closure);
  vm.run()
}

#[allow(unused)]
pub fn do_string(src: String) -> Result<(), String> {
  let parser = Parser::new(src, "buf".into());

  let mut compiler = Compiler::new("buf".into());
  compiler.compile(parser.nodes)?;

  let mut vm = VM::new(compiler.closure.clone());
  vm.run()
}