use std::collections::HashMap;

use crate::common::{ Closure, Value, Opcode, type_value };
use code::*;

mod builtin;
mod code;

pub use code::pretty_print_code;

pub struct CallInfo {
  closure: Closure,
  base: u8,
  pc: usize
}

impl CallInfo {
  pub fn new(closure: Closure, base: u8) -> Self {
    CallInfo {
      closure,
      base,
      pc: 0
    }
  }
}

pub struct VM {
  call_stack: Vec<CallInfo>,
  globals: HashMap<String, Value>,
  regs: Vec<Value>
}

impl VM {
  pub fn new(closure: Closure) -> Self {
    VM {
      call_stack: vec![ CallInfo::new(closure, 0) ],
      globals: HashMap::new(),
      regs: Vec::with_capacity(20)
    }
  }

  pub fn run(&mut self) -> Result<(), String> {
    builtin::load(self);

    while self.is_end_of_code() {
      let res = self.exec();

      if let Err(e) = res {
        return Err(self.error(e))
      }
    }

    Ok(())
  }

  fn exec(&mut self) -> Result<(), String> {
    let call = self.call();
    let base = call.base;
    let i = self.get_i();

    macro_rules! konst {
      ($v:expr) => {
        call.closure.consts[($v) as usize].clone()
      };
    }

    macro_rules! RA_mut {
      () => {{
        let pos = (base + get_a(i)) as usize;
        let v = self.regs.get_mut(pos);
        if v.is_none() {
          self.regs.insert(pos, Value::Nil);
          self.regs.get_mut(pos).unwrap()
        } else {
          v.unwrap()
        }
      }};
    }

    macro_rules! RA {
      () => {
        self.regs[(base + get_a(i)) as usize].clone()
      };
    }

    macro_rules! RB {
      () => {
        self.regs[(base + get_b(i)) as usize].clone()
      };
    }

    macro_rules! RC_mut {
      () => {{
        let pos = (base + get_c(i)) as usize;
        let v = self.regs.get_mut(pos);
        if v.is_none() {
          self.regs.insert(pos, Value::Nil);
          self.regs.get_mut(pos).unwrap()
        } else {
          v.unwrap()
        }
      }};
    }

    macro_rules! RC {
      () => {
        self.regs[(base + get_c(i)) as usize].clone()
      };
    }

    macro_rules! RCA {
      () => {
        if get_a_mode(i) == 1 {
          konst!(get_a(i))
        } else {
          RC!()
        }
      };
    }

    macro_rules! RCB {
      () => {
        if get_b_mode(i) == 1 {
          konst!(get_b(i))
        } else {
          RB!()
        }
      };
    }

    macro_rules! arith {
      ($op:tt) => {{
        let rca = RCA!();
        let rcb = RCB!();

        *RC_mut!() = (rca $op rcb)?;
      }};
    }

    macro_rules! cmp {
      ($op:tt) => {{
        let rca = RCA!();
        let rcb = RCB!();

        *RC_mut!() = Value::Bool(rca $op rcb)
      }};
    }

    match get_op(i) {

      Opcode::Move => {
        *RA_mut!() = RB!()
      }

      Opcode::LoadConst => {
        *RA_mut!() = konst!(get_bx(i))
      }

      Opcode::LoadBool => {
        *RA_mut!() = Value::Bool(get_b(i) == 1)
      }

      Opcode::LoadNil => {
        *RA_mut!() = Value::Nil
      }

      Opcode::GetGlobal => {
        let k = konst!(get_bx(i));
        if let Value::String(str) = k {
          *RA_mut!() = self.globals.get(&str).unwrap_or(&Value::Nil).clone();
        } else {
          return Err("global index must be a string".into())
        }
      }

      Opcode::SetGlobal => {
        let k = konst!(get_bx(i));
        if let Value::String(str) = k {
          self.globals.insert(str, RA!());
        } else {
          return Err("global index must be a string".into())
        }
      }

      Opcode::Add => arith!(+),
      Opcode::Sub => arith!(-),
      Opcode::Mul => arith!(*),
      Opcode::Div => arith!(/),
      Opcode::Neq => cmp!(!=),
      Opcode::Eq => cmp!(==),
      Opcode::Lt => cmp!(>),
      Opcode::Gt => cmp!(<),
      Opcode::Le => cmp!(>=),
      Opcode::Ge => cmp!(<=),

      Opcode::Neg => {
        let rcb = RCB!();
        if let Value::Number(n) = rcb {
          *RA_mut!() = Value::Number(-n)
        } else {
          return Err(format!("cannot make a {} negative", type_value(rcb)))
        }
      }

      Opcode::Not => {
        *RA_mut!() = Value::Bool(!self.bool(RCB!()))
      }

      Opcode::Jmp => {
        if get_a(i) == 1 {
          *self.pc_mut() -= get_bx(i) as usize;
        } else {
          *self.pc_mut() += get_bx(i) as usize;
        }

        return Ok(())
      }

      Opcode::Test => {
        if self.bool(RA!()) {
          *self.pc_mut() += 2;
          return Ok(())
        }
      }

      Opcode::Call => {
        let func = RA!();
        let base = get_a(i) + 1;

        match func {
          Value::NativeFunc(nf) => {
            let mut vals = Vec::new();

            for i in base .. get_b(i) {
              vals.push(&self.regs[i as usize])
            }

            let ret = (nf.func)(vals)?;
            *RC_mut!() = ret;
          }

          Value::Closure(_c) => {
            todo!();
            /*let call = CallInfo::new(c, base);
            self.call_stack.push(call);
            return Ok(()) // don't skip first instruction of new function*/
          }

          _ => return Err(format!("cannot call a {} value", type_value(func)))
        }
      }

      Opcode::Close => {
        for i in get_a(i) as usize .. self.regs.len() {
          self.regs[i] = Value::Nil
        }
      }
    }

    *self.pc_mut() += 1;

    Ok(())
  }

  fn error(&self, err: String) -> String {
    let call = self.call();
    format!("{}:{}: {}", call.closure.name, call.closure.lines[call.pc], err)
  }

  fn bool(&self, val: Value) -> bool {
    match val {
      Value::Bool(b) => b,
      Value::Nil => false,
      _ => true
    }
  }

  #[inline(always)]
  fn is_end_of_code(&self) -> bool {
    let call = self.call_stack.last().unwrap();
    call.pc < call.closure.code.len()
  }

  #[inline(always)]
  fn get_i(&self) -> u32 {
    let call = self.call();
    call.closure.code[call.pc]
  }

  #[inline(always)]
  fn call(&self) -> &CallInfo {
    self.call_stack.last().unwrap()
  }

  #[inline(always)]
  fn call_mut(&mut self) -> &mut CallInfo {
    self.call_stack.last_mut().unwrap()
  }

  fn pc(self) -> usize {
    self.call().pc
  }

  #[inline(always)]
  fn pc_mut(&mut self) -> &mut usize {
    &mut self.call_mut().pc
  }
}