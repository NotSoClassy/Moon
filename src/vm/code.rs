use crate::common::{ Opcode, Opmode, OPMODES };

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

pub fn pretty_print_code(code: Vec<u32>) {
  for i in code {
    let mode = OPMODES[get_op(i) as usize];
    let am = if get_a_mode(i) == 1 { "-" } else { "" };

    match mode {
      Opmode::Abc => {
        let bm = if get_b_mode(i) == 1 { "-" } else { "" };
        println!("{:?} {}{} {}{} {}", get_op(i), am, get_a(i), bm, get_b(i), get_c(i))
      }

      Opmode::Abx => {
        println!("{:?} {}{} {}", get_op(i), am, get_a(i), get_bx(i))
      }
    }
  }
}