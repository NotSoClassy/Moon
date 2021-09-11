use crate::common::{ Opcode, Opmode, OPMODES, Closure, Value };

pub fn get_op(i: u32) -> Opcode {
  Opcode::from((i >> 26 & 0x3F) as u8)
}

pub fn get_a_mode(i: u32) -> u8 {
  (i >> 25 & 0x1) as u8
}

pub fn get_a(i: u32) -> u8 {
  (i >> 17 & 0xFF) as u8
}

pub fn get_b_mode(i: u32) -> u8 {
  (i >> 16 & 0x1) as u8
}

pub fn get_b(i: u32) -> u8 {
  (i >> 8 & 0xFF) as u8
}

pub fn get_c(i: u32) -> u8 {
  (i & 0xFF) as u8
}

pub fn get_bx(i: u32) -> u16 {
  ((get_b(i) as u16) << 8)
  | (get_c(i) as u16)
}

pub fn format_instruction(i: u32) -> String {
  let mode = OPMODES[get_op(i) as usize];
  let am = if get_a_mode(i) == 1 { "-" } else { "" };

  match mode {
    Opmode::Abc => {
      let bm = if get_b_mode(i) == 1 { "-" } else { "" };
      format!("{:?} {}{} {}{} {}", get_op(i), am, get_a(i), bm, get_b(i), get_c(i))
    }

    Opmode::Abx => {
      format!("{:?} {}{} {}", get_op(i), am, get_a(i), get_bx(i))
    }
  }
}

fn get_fn_info(closure: Closure) -> String {
  let mut nfn = 0;

  for val in &closure.consts {
    if let Value::Closure(..) = val { nfn += 1 }
  }

  format!("{} <{}> ({} instructions)\n{} params, {} constants, {} functions", closure.name, closure.file_name, closure.code.len(), closure.nparams, closure.consts.len() - nfn, nfn)
}

pub fn pretty_print_closure(closure: Closure, recursive: bool) {
  println!("{}", get_fn_info(closure.clone()));

  for (idx, instruction) in closure.code.iter().enumerate() {
    let s = format_instruction(*instruction);

    println!("\t{}\t[{}]\t{}", idx + 1, closure.lines[idx], s);
  }

  let mut funcs = Vec::new();

  let consts = closure.consts.iter().filter(| v | {
    if let Value::Closure(c) = v { funcs.push(c); false } else { true }
  }).collect::<Vec<&Value>>();

  println!("constants ({})", consts.len());

  for (idx, konst) in consts.iter().enumerate() {
    println!("\t{}\t{:?}", idx + 1, konst)
  }

  if recursive {
    for func in funcs {
      println!();
      pretty_print_closure(func.clone(), true)
    }
  }
}