use crate::parser::*;

use std::io::prelude::*;
use std::fs::File;
use std::env;

mod parser;
mod common;
mod lexer;
mod vm;

fn main() {
  let arg = env::args().nth(1).expect("expected file name");
  let mut file = File::open(arg.clone()).expect("could not open file");
  let mut str = String::new();

  file.read_to_string(&mut str).expect("failure to read file");

  let mut parser = Parser::new(str.into(), arg);

  let res = parser.parse();

  if let Err(e) = res {
    eprintln!("{}", e);
    return
  }

  let mut compiler = gen::Compiler::new();
  let res = compiler.compile(parser.nodes);

  if let Err(e) = res {
    eprintln!("{:?}", e);
    return
  }


  let mut vm = crate::vm::VM::new(compiler.fs.closure);
  let res = vm.run();

  if let Err(e) = res {
    eprintln!("{}", e);
    return
  }
}