use crate::common::{
  Closure, Value, Opcode
};

use std::collections::HashMap;

mod builtin;

pub struct CallInfo {
  pub closure: Closure,
  pub base: usize,
  pub pc: usize
}

impl CallInfo {
  pub fn new(closure: Closure) -> Self {
    CallInfo {
      closure,
      base: 0,
      pc: 0
    }
  }
}

pub struct VM {
  pub call_stack: Vec<CallInfo>,
  pub globals: HashMap<String, Value>,
  pub regs: Vec<Value>
}

impl VM {
  pub fn new(closure: Closure) -> Self {
    let mut vm = VM {
      call_stack: vec![ CallInfo::new(closure) ],
      globals: HashMap::new(),
      regs: Vec::new()
    };

    vm.load_globals();

    vm
  }

  pub fn load_globals(&mut self) {
    builtin::load(self);
  }

  pub fn run(&mut self) -> Result<(), String> {
    while self.is_end_of_code() {
      self.execute()?
    }
    Ok(())
  }

  fn next(&mut self) {
    *self.pc_mut() += 1
  }

  fn next_byte(&mut self) -> u8 {
    let b = self.closure().code[self.pc()]; self.next(); b
  }

  fn operands(&mut self) -> (Opcode, u8, u8, u8, u16) {
    let op = self.next_byte();
    let a = self.next_byte();
    let b = self.next_byte();
    let c = self.next_byte();
    let bx = ((b as u16) << 8) | (c as u16);

    (Opcode::from(op), a, b, c, bx)
  }

  fn execute(&mut self) -> Result<(), String> {
    let (op, a, b, c, bx) = self.operands();
    let base = self.call_info().base;
    // println!("{:?} {} {} {} {}", op, a, b, c, bx);

    macro_rules! arithmetic {
      ($op:tt) => { {
        let lhs = get!(b);
        let rhs = get!(c);

        if let Value::Number(lhn) = lhs {
          if let Value::Number(rhn) = rhs {
            set!(a, Value::Number(lhn $op rhn))
          } else {
            return Err("expected number".into())
          }
        } else {
          return Err("expected number".into())
        } }
      };
    }

    macro_rules! cmp {
      ($op:tt) => { {
        let lhs = get!(b);
        let rhs = get!(c);

        set!(a, Value::Bool(lhs $op rhs))
      } };
    }

    macro_rules! set {
      ($pos:expr, $val:expr) => { {
        let idx = base + $pos as usize;
        if self.regs.get(idx).is_some() {
          self.regs[idx] = $val
        } else {
          let pos = $pos as usize;
          for _ in self.regs.len()..pos {
            self.regs.push(Value::Nil)
          }
          self.regs.push($val)
        } }
      };
    }

    macro_rules! get {
      ($pos:expr) => {
        self.regs[base + $pos as usize].clone()
      };
    }

    match Opcode::from(op) {
      Opcode::Move => set!(a, get!(b)),
      Opcode::LoadConst => set!(a, self.get_const(bx)),
      Opcode::LoadBool => set!(a, Value::Bool(b == 1)),
      Opcode::LoadNil => set!(a, Value::Nil),

      Opcode::GetGlobal => {
        if let Value::String(name) = self.get_const(bx) {
          let value = self.globals.get(&name).unwrap_or(&Value::Nil);
          set!(a, value.clone())
        } else {
          return Err("global index is not a string".into())
        }
      }

      Opcode::SetGlobal => {
        if let Value::String(name) = self.get_const(bx) {
          self.globals.insert(name, get!(a));
        } else {
          return Err("global index is not a string".into())
        }
      }

      Opcode::Add => arithmetic!(+),
      Opcode::Sub => arithmetic!(-),
      Opcode::Mul => arithmetic!(*),
      Opcode::Div => arithmetic!(/),
      Opcode::Neq => cmp!(!=),
      Opcode::Eq => cmp!(==),
      Opcode::Gt => cmp!(<),
      Opcode::Ge => cmp!(<=),
      Opcode::Lt => cmp!(>),
      Opcode::Le => cmp!(>=),

      Opcode::Not => set!(a, Value::Bool(self.not(get!(b)))),
      Opcode::Neg => {
        let n = self.number(get!(b))?;
        set!(a, Value::Number(-n))
      }

      Opcode::Jmp => self.jmp(a == 1, bx as usize),

      Opcode::Test => {
        if self.truthy(get!(a)) {
          *self.pc_mut() += 4;
        }
      }

      Opcode::Call => {
        let func = get!(a);

        let base = a as usize + 1;

        if let Value::Closure(closure) = func {
          let nparams = closure.nparams;
          let ci = CallInfo {
            closure,
            base: base + nparams as usize - 1,
            pc: 0
          };

          self.call_stack.push(ci);
        } else if let Value::NativeFunc(nf) = func {
          let mut vals = Vec::new();

          for i in base..b as usize {
            vals.push(self.regs.get(i).unwrap_or(&Value::Nil))
          }

          let ret = (nf.func)(vals);
          set!(c, ret)
        } else {
          return Err("only functions can be called".into())
        }
      }

      Opcode::Close => {
        for i in a..self.regs.len() as u8 {
          self.regs[i as usize] = Value::Nil
        }
      }
    }

    Ok(())
  }

  fn jmp(&mut self, neg: bool, i: usize) {
    if neg {
      *self.pc_mut() -= i * 4
    } else {
      *self.pc_mut() += i * 4
    }
  }

  fn not(&self, val: Value) -> bool {
    !self.truthy(val)
  }

  fn truthy(&self, val: Value) -> bool {
    match val {
      Value::Bool(b) => b,
      Value::Nil => false,
      _ => true,
    }
  }

  fn get_const(&self, pos: u16) -> Value {
    self.closure().consts[pos as usize].clone()
  }

  fn number(&self, v: Value) -> Result<f64, String> {
    if let Value::Number(n) = v {
      Ok(n)
    } else {
      Err("expected number".into())
    }
  }

  #[inline(always)]
  fn is_end_of_code(&self) -> bool {
    let cf = self.call_stack.last().unwrap();
    cf.pc < cf.closure.code.len()
  }

  #[inline(always)]
  fn pc_mut(&mut self) -> &mut usize {
    &mut self.call_stack.last_mut().unwrap().pc
  }

  #[inline(always)]
  fn pc(&self) -> usize {
    self.call_info().pc
  }

  #[inline(always)]
  fn call_info(&self) -> &CallInfo {
    &self.call_stack.last().unwrap()
  }

  #[inline(always)]
  fn closure(&self) -> &Closure {
    &self.call_info().closure
  }
}