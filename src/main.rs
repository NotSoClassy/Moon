extern crate libc;

use std::process::exit;
use std::env;

use common::utils::{ compile_file, do_file };

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

      vm::pretty_print_closure(closure, false);
      Ok(())
    }

    Exec::PrintBytecodeRecursive(name) => {
      let closure = compile_file(name)?;
      vm::pretty_print_closure(closure, true);

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