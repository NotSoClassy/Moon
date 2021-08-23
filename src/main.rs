use parser::{ Parser, gen::Compiler };
use common::Closure;
use vm::VM;

use std::io::prelude::*;
use std::fs::File;
use std::env;

mod parser;
mod common;
mod lexer;
mod vm;

fn do_file(name: String) -> Result<Closure, String> {
  let mut file = File::open(name.clone()).expect("could not open file");
  let mut str = String::new();

  file.read_to_string(&mut str).expect("failure to read file");

  if str == "" { return Ok(Closure::new()) }

  let mut parser = Parser::new(str.into(), name.clone());
  parser.parse()?;

  let mut compiler = Compiler::new(name.clone());
  compiler.compile(parser.nodes)?;

  Ok(compiler.closure)
}

fn main() {
  let arg = env::args().nth(1).expect("expected file name");

  let res = do_file(arg);

  if let Err(e) = res {
    eprintln!("{}", e);
    return
  }

  let closure = res.unwrap();

  #[cfg(debug_assertions)]
  vm::pretty_print_code(closure.clone().code);

  let mut vm = VM::new(closure);
  let res = vm.run();

  if let Err(e) = res {
    eprintln!("{}", e);
    return
  }
}
