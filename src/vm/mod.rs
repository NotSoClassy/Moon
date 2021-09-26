use crate::common::{ Closure, Value, Opcode, Type, Array, Table };
use crate::vm::env::Env;
use code::*;

pub mod code;
pub mod env;
pub mod error;

pub use code::pretty_print_closure;
pub use error::RuntimeError;

pub struct NativeCallInfo {
  base: usize,
  top: usize
}

impl NativeCallInfo {
  pub fn new() -> Self {
    NativeCallInfo {
      base: 0,
      top: 0
    }
  }
}

#[derive(Debug)]
pub struct CallInfo {
  closure: Closure,
  is_builtin: bool,
  base: usize,
  pc: usize
}

impl CallInfo {
  pub fn new(closure: Closure, base: usize) -> Self {
    CallInfo {
      closure,
      is_builtin: false,
      base,
      pc: 0
    }
  }
}

pub struct VM {
  pub env: Env,
  call_stack: Vec<CallInfo>,
  nci: NativeCallInfo,
  regs: Vec<Value>,
  ncalls: usize
}

impl VM {
  pub fn new(closure: Closure) -> Self {
    VM {
      call_stack: vec![ CallInfo::new(closure, 0) ],
      env: Env::new(),
      nci: NativeCallInfo::new(),
      regs: Vec::with_capacity(20),
      ncalls: 0
    }
  }

  pub fn run(&mut self) -> Result<(), String> {
    self.env.load();

    while self.is_end_of_code() {
      self.run_once()?;
    }

    Ok(())
  }

  pub fn run_closure(&mut self, closure: Closure) -> Result<(), String> {
    let base = self.call_stack.last().unwrap().base;

    self.call_stack.push(CallInfo::new(closure, base));

    while self.is_end_of_code() {
      self.run_once()?;
    }

    Ok(())
  }

  fn run_once(&mut self) -> Result<(), String> {
    let res = self.exec();

    if let Err(e) = res {
      let err = e.to_error(&self.call_stack);

      if let Some(v) = self.call_stack.last() {
        if v.is_builtin { self.call_stack.pop(); }
      }

      return Err(self.error(err))
    }

    Ok(())
  }

  fn exec(&mut self) -> Result<(), RuntimeError> {
    let call = self.call();
    let base = call.base;
    let i = call.closure.code[call.pc];

    macro_rules! get_mut {
      ($pos:expr) => {{
        let pos = $pos;
        let v = self.regs.get_mut(pos);

        if v.is_none() {
          while self.regs.get(pos).is_none() {
            self.regs.push(Value::Nil)
          }

          self.regs.get_mut(pos).unwrap()
        } else {
          v.unwrap()
        }}
      };
    }

    macro_rules! konst {
      ($v:expr) => {
        &call.closure.consts[($v) as usize]
      };
    }

    macro_rules! RA_mut {
      () => {{
        let pos = base + get_a(i) as usize;
        get_mut!(pos)
      }};
    }

    macro_rules! A {
      () => {
        base + get_a(i) as usize
      };
    }

    macro_rules! RA {
      () => {
        &self.regs[A!() as usize]
      };
    }

    macro_rules! B {
      () => {
        base + get_b(i) as usize
      };
    }

    macro_rules! RB {
      () => {
        &self.regs[B!() as usize]
      };
    }

    macro_rules! RC_mut {
      () => {{
        let pos = base + get_c(i) as usize;
        get_mut!(pos)
      }};
    }

    macro_rules! C {
      () => {
        base + get_c(i) as usize
      };
    }

    macro_rules! RC {
      () => {
        &self.regs[C!() as usize]
      };
    }

    macro_rules! RCA {
      () => {
        if get_a_mode(i) == 1 {
          konst!(get_a(i))
        } else {
          RA!()
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
        let t1 = Type::from(rca);
        let t2 = Type::from(rcb);

        let res = (rca.clone() $op rcb.clone());

        if let Err(()) = res {
          return Err(RuntimeError::TypeError("perform an arithmetic on".into(), t1, Some(t2)))
        } else {
          *RC_mut!() = res.unwrap()
        }
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
        *RA_mut!() = RB!().clone()
      }

      Opcode::LoadConst => {
        *RA_mut!() = konst!(get_bx(i)).clone()
      }

      Opcode::LoadBool => {
        *RA_mut!() = Value::Bool(get_b(i) == 1)
      }

      Opcode::LoadNil => {
        *RA_mut!() = Value::Nil
      }

      Opcode::SetUpVal => {
        let mut upvals = call.closure.upvals.borrow_mut();
        let (_, upval) = upvals.get_mut(get_a(i) as usize).unwrap();
        let val = RB!().clone();

        *upval = val
      }

      Opcode::GetUpVal => {
        let val = {
          let upvals = call.closure.upvals.borrow();
          let (_, upval) = upvals
            .get(get_b(i) as usize)
            .unwrap();

          upval.clone()
        };

        *RA_mut!() = val;
      }

      Opcode::GetGlobal => {
        let k = konst!(get_bx(i));

        *RA_mut!() = self.env.globals.tbl.borrow().get(&k).unwrap_or(&Value::Nil).clone();
      }

      Opcode::SetGlobal => {
        let k = konst!(get_bx(i)).clone();
        let ra = RA!().clone();

        self.env.set_global(k, ra);
      }

      Opcode::NewTable => {
        let (a, b) = (A!(), B!());
        let tbl = Table::new();
        let mut i = a;

        while i < b {
          let key = self.regs[i as usize].clone();
          let val = self.regs[i + 1 as usize].clone();

          tbl.insert(key, val)?;

          i += 2;
        }

        *RA_mut!() = Value::Table(tbl);
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

      Opcode::GetObj => {
        let a = RA!().clone();
        let b = RCB!();

        match a {
          Value::Array(array) => {
            *RA_mut!() = array.get(b)?
          }

          Value::Table(tbl) => {
            *RA_mut!() = tbl.get(b)?
          }

          _ => return Err(self.index_error(&a))
        }
      }

      Opcode::SetObj => {
        let idx = RCA!();
        let val = RCB!();
        let obj = RC!();

        match obj {
          Value::Array(array) => {
            array.insert(&idx, val.clone())?;
          }

          Value::Table(tbl) => {
            tbl.insert(idx.clone(), val.clone())?;
          }

          _ => return Err(self.index_error(&obj))
        }
      }

      Opcode::Add => arith!(+),
      Opcode::Sub => arith!(-),
      Opcode::Mul => arith!(*),
      Opcode::Div => arith!(/),
      Opcode::Mod => arith!(%),

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
          return Err(RuntimeError::TypeError("perform an arithmetic on".into(), Type::from(rcb), None))
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
        if self.bool(RA!()) == (get_b(i) == 0) {
          *self.pc_mut() += 2;
          return Ok(())
        }
      }

      Opcode::Call => {
        let func = RA!().clone();
        let base = A!() + 1;
        let b = B!();

        match func {
          Value::NativeFunc(nf) => {
            self.nci = NativeCallInfo {
              base,
              top: b
            };

            let ret = (nf.func)(self);

            if let Err(e) = ret {
              let mut c = Closure::new("rust".into());
              c.name = nf.name.clone();

              let mut info = CallInfo::new(c, 0); // for trace
              info.is_builtin = true;

              self.call_stack.push(info);

              return Err(e)
            } else {
              *self.pc_mut() += 1;
              *RC_mut!() = ret.unwrap();
            }

            return Ok(())
          }

          Value::Closure(c) => {
            for i in base ..= b {
              if self.regs.get(i as usize).is_none() {
                self.regs.push(Value::Nil);
              }
            }

            let mut call = CallInfo::new(c.clone(), base);

            for i in &call.closure.code {
              let i = *i;
              if get_op(i) == Opcode::GetUpVal && get_c(i) == 1 {
                let b = get_b(i);
                let val = self.regs[b as usize].clone();

                call.closure.upvals.borrow_mut().insert(get_a(i) as usize, (b, val));
                call.pc += 1;
              } else { break }
            }

            self.call_stack.push(call);
            self.ncalls += 1;

            if self.ncalls >= 20000 {
              return Err(RuntimeError::StackOverflow)
            }

            return Ok(()) // don't skip first instruction of new function
          }

          _ => return Err(RuntimeError::TypeError("call".into(), Type::from(&func), None))
        }
      }

      Opcode::Return => {
        let v = if get_b(i) == 1 {
          Value::Nil
        } else {
          RCA!().clone()
        };

        let base = if base == 0 { base } else { base - 1 };
        self.regs[base as usize] = v;

        let call = self.call_stack.pop().unwrap();

        for (pos, upval) in &*call.closure.upvals.borrow() {
          let val = upval.clone();
          self.regs[*pos as usize] = val;
        }

        if self.is_end_of_code() {
          *self.pc_mut() += 1;
        }

        return Ok(())
      }

      Opcode::Close => {
        for i in A!() as usize .. self.regs.len() {
          self.regs[i] = Value::Nil
        }
      }
    }

    *self.pc_mut() += 1;

    Ok(())
  }

  fn error(&self, err: String) -> String {
    let call = self.call();

    format!("{}:{}: {}", call.closure.file_name, call.closure.lines[call.pc], err)
  }

  fn bool(&self, val: &Value) -> bool {
    match val {
      Value::Bool(b) => *b,
      Value::Nil => false,
      _ => true
    }
  }

  #[inline]
  fn index_error(&self, val: &Value) -> RuntimeError {
    RuntimeError::TypeError("index".into(), Type::from(val), None)
  }

  #[inline]
  fn new_array(&self, vals: Vec<Value>) -> Value {
    Value::Array(Array::new(vals))
  }

  #[inline]
  fn is_end_of_code(&self) -> bool {
    let call = self.call_stack.last();
    if let Some(call) = call {
      call.pc < call.closure.code.len()
    } else {
      false
    }
  }

  #[inline]
  fn call(&self) -> &CallInfo {
    self.call_stack.last().unwrap()
  }

  #[inline]
  fn call_mut(&mut self) -> &mut CallInfo {
    self.call_stack.last_mut().unwrap()
  }

  #[inline]
  fn pc_mut(&mut self) -> &mut usize {
    &mut self.call_mut().pc
  }
}