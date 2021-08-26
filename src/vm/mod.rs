use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

use crate::common::{ Closure, Value, Opcode, type_value };
use code::*;

mod builtin;
mod code;

pub use code::pretty_print_code;

pub struct NativeCallInfo {
  base: u8,
  top: u8
}

impl NativeCallInfo {
  pub fn new() -> Self {
    NativeCallInfo {
      base: 0,
      top: 0
    }
  }
}

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
  nci: NativeCallInfo,
  regs: Vec<Value>
}

impl VM {
  pub fn new(closure: Closure) -> Self {
    VM {
      call_stack: vec![ CallInfo::new(closure, 0) ],
      globals: HashMap::new(),
      nci: NativeCallInfo::new(),
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
    let i = call.closure.code[call.pc];

    macro_rules! get_mut {
      ($pos:expr) => {{
        let pos = $pos;
        let v = self.regs.get_mut(pos);
        if v.is_none() {
          self.regs.insert(pos, Value::Nil);
          self.regs.get_mut(pos).unwrap()
        } else {
          v.unwrap()
        }}
      };
    }

    macro_rules! konst {
      ($v:expr) => {
        call.closure.consts[($v) as usize].clone()
      };
    }

    macro_rules! RA_mut {
      () => {{
        let pos = (base + get_a(i)) as usize;
        get_mut!(pos)
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
        get_mut!(pos)
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

      Opcode::NewArray => {
        let (a, b) = (get_a(i), get_b(i));
        let size = b - a;

        let mut array = Vec::with_capacity(size as usize);

        for i in a .. b {
          array.push(self.regs[i as usize].clone())
        }

        *RA_mut!() = self.new_array(array);
      }

      Opcode::GetArray => {
        let a = get_a(i);
        let b = RCB!();

        *RA_mut!() = self.index_array(self.regs[a as usize].clone(), b)?
      }

      Opcode::SetArray => {
        let idx = RCA!();
        let val = RCB!();
        let val_array = RC!();

        let (ref_array, idx) = self.validate_index(val_array, idx)?;
        let mut array = ref_array.borrow_mut();
        let len = array.len();

        if idx == len {
          array.push(val);
        } else {
          if let Some(v) = array.get_mut(idx) {
            *v = val;
          } else {
            return Err("index out of bounds".into())
          }
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
            self.nci = NativeCallInfo {
              base,
              top: get_b(i)
            };

            let ret = (nf.func)(self)?;
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

  fn index_array(&self, array: Value, pos: Value) -> Result<Value, String> {
    let (array, pos) = self.validate_index(array, pos)?;
    let array = array.borrow();

    Ok(array.get(pos).unwrap_or(&Value::Nil).clone())
  }

  fn validate_index(&self, array: Value, idx: Value) -> Result<(Rc<RefCell<Vec<Value>>>, usize), String> {
    if let Value::Array(arr) = array {
      if let Value::Number(pos) = idx {
        if pos.floor() == pos {
          if pos.is_sign_positive() {
            Ok((arr, pos as usize))
          } else {
            Err("array index must be positive".into())
          }
        } else {
          Err("array index must be an integer".into())
        }
      } else {
        Err(format!("attempt to index array with a {}", type_value(idx)))
      }
    } else {
      Err(format!("attempt to index a {} value", type_value(array)))
    }
  }

  #[inline(always)]
  fn new_array(&self, vals: Vec<Value>) -> Value {
    Value::Array(Rc::new(RefCell::new(vals)))
  }

  #[inline(always)]
  fn is_end_of_code(&self) -> bool {
    let call = self.call_stack.last().unwrap();
    call.pc < call.closure.code.len()
  }

  #[inline(always)]
  fn call(&self) -> &CallInfo {
    self.call_stack.last().unwrap()
  }

  #[inline(always)]
  fn call_mut(&mut self) -> &mut CallInfo {
    self.call_stack.last_mut().unwrap()
  }

  #[inline(always)]
  fn pc_mut(&mut self) -> &mut usize {
    &mut self.call_mut().pc
  }
}