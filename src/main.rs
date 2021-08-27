use parser::{ Parser, gen::Compiler };
use common::{ Closure, Value };
use vm::VM;

use std::io::prelude::*;
use std::process::exit;
use std::fs::File;
use std::env;

mod parser;
mod common;
mod lexer;
mod vm;

enum Exec {
  PrintBytecodeRecursive(String),
  PrintBytecode(String),
  DoFile(String),
  Exit
}

fn compile_file(name: String) -> Result<Closure, String> {
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

fn do_file(name: String) -> Result<(), String> {
  let closure = compile_file(name)?;

  let mut vm = VM::new(closure);
  vm.run()
}

fn bytecode_recursive(level: &mut usize, closure: Closure) {
  println!("{}{}:", "\t".repeat(*level - 1), closure.name);
  println!("{:?}",closure.consts);
  vm::pretty_print_code(&"\t".repeat(*level), closure.code);

  for konst in closure.consts {
    if let Value::Closure(c) = konst {
      *level += 1;
      bytecode_recursive(level, c)
    }
  }
}

fn get_name(name: Option<&String>, err: &str) -> Result<String, String> {
  if let Some(name) = name {
    Ok(name.to_string())
  } else {
    Err(err.to_string())
  }
}

fn parse_args(args: Vec<String>) -> Result<Exec, String> {
  match args.get(1) {
    Some(v) => match v.trim() {

      "-l" => {
        let name = get_name(args.get(2), "expected file name")?;
        Ok(Exec::PrintBytecode(name.trim().to_string()))
      },

      "-ll" => {
        let name = get_name(args.get(2), "expected file name")?;
        Ok(Exec::PrintBytecodeRecursive(name.trim().to_string()))
      }

      _ => {
        Ok(Exec::DoFile(v.to_string()))
      }

    },

    None => {
      println!("{} [options] filename
      Options:
          -l   print bytecode of main function
          -ll  print bytecode of main function and all sub functions
      ", args.get(0).unwrap_or(&"moon".to_string()));
      Ok(Exec::Exit)
    }
  }
}

fn run() -> Result<(), String> {
  let args = env::args().collect::<Vec<String>>();
  let exec = parse_args(args)?;

  match exec {
    Exec::DoFile(name) => {
      do_file(name)
    }

    Exec::PrintBytecode(name) => {
      let closure = compile_file(name)?;

      println!("{}:", closure.name);
      vm::pretty_print_code("\t", closure.code);
      Ok(())
    }

    Exec::PrintBytecodeRecursive(name) => {
      let closure = compile_file(name)?;
      bytecode_recursive(&mut 1, closure);

      Ok(())
    }

    Exec::Exit => Ok(())
  }
}

fn main() {
  let res = run();

  if let Err(e) = res {
    eprintln!("{}", e);
    exit(1)
  }
}