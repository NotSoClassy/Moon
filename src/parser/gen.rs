use std::convert::TryInto;

use crate::common::{ Closure, Opcode, Value };
use crate::parser::ast::{
  Node, Stmt, Expr, UnOp, BinOp
};

pub struct VarInfo {
  pub name: String,
  pub pos: u8
}

impl VarInfo {
  pub fn new(name: String, pos: u8) -> Self {
    VarInfo {
      name,
      pos
    }
  }
}

pub struct Compiler {
  pub closure: Closure,
  freereg: u8,
  nvars: u8,
  vars: Vec<VarInfo>,
  name: String,
  line: usize,
  ni: usize
}

#[inline(always)]
fn get_mode(val: u16) -> u8 {
  (val >> 8 & 0x1).try_into().unwrap()
}

#[inline(always)]
fn set_mode(mode: u8, val: u8) -> u16 {
  let mode = mode & 0x1;
  ((mode as u16) << 8)
      | (val as u16)
}

#[inline(always)]
fn make_abx(op: Opcode, a: u16, bx: u16) -> u32 {
  make_abc(op, a, (bx >> 8) & 0xFF, (bx & 0xFF) as u8)
}

fn make_abc(op: Opcode, a: u16, b: u16, c: u8) -> u32 {
  let op = op as u8 & 0x3F; // select first 6 bits (0x3F == 0b111111)
  let a = a & 0x1FF; // select first 9 bits (0x1FF == 0b111111111)
  let b = b & 0x1FF;

  ((op as u32) << 26)
      | ((a as u32) << 17)
      | ((b as u32) << 8)
      |  (c as u32)
}

impl Compiler {
  pub fn new(name: String) -> Self {
    let mut closure = Closure::new();
    closure.name = name.clone();

    Compiler {
      nvars: 0,
      freereg: 0,
      vars: Vec::new(),
      closure: closure,
      line: 1,
      name,
      ni: 0
    }
  }

  pub fn compile(&mut self, nodes: Vec<Node>) -> Result<(), String> {
    for node in nodes {
      let err = self.walk(node);
      self.error(err)?;

      self.freereg = self.nvars;
    }

    self.final_ret();

    Ok(())
  }

  fn final_ret(&mut self) {
    self.emit(make_abc(Opcode::Return, 0, 1, 0))
  }

  #[inline(always)]
  fn error(&self, err: Result<(), String>) -> Result<(), String> {
    if let Err(e) = err {
      Err(format!("{}:{}: {}", self.name, self.line, e))
    } else {
      Ok(())
    }
  }

  #[inline(always)]
  fn walk(&mut self, node: Node) -> Result<(), String> {
    self.stmt(node.stmt)?;
    self.line = node.line;
    Ok(())
  }

  fn stmt(&mut self, stmt: Stmt) -> Result<(), String> {
    match stmt {
      Stmt::Return(val) => self.return_stmt(val),
      Stmt::While(cond, block) => self.while_stmt(cond, *block),
      Stmt::Block(block) => self.block_stmt(block),
      Stmt::Let(name, value) => self.let_stmt(name, value),
      Stmt::If(cond, blocks) => self.if_stmt(cond, *blocks),
      Stmt::Fn(name, params, body) => self.fn_stmt(name, params, *body),

      Stmt::Expr(exp) => { self.exp2nextreg(exp)?; Ok(())},
    }
  }

  fn let_stmt(&mut self, name: String, value: Expr) -> Result<(), String> {
    self.reserve_regs(1)?;
    self.expr(value, self.freereg - 1)?;
    self.register_var(name)?;
    Ok(())
  }

  fn fn_stmt(&mut self, name: String, params: Vec<String>, body: Node) -> Result<(), String> {
    let mut compiler = Compiler::new(self.name.clone());
    compiler.closure.name = name.clone();
    compiler.closure.nparams = params.len() as u8;

    for param in params {
      compiler.register_var(param)?;
    }

    compiler.freereg = compiler.nvars;
    compiler.walk(body)?;

    let closure = Value::Closure(compiler.closure);

    let var = self.register_var(name)?;
    self.load_const(closure, var)?;

    Ok(())
  }

  fn return_stmt(&mut self, val: Expr) -> Result<(), String> {
    let val = self.exp2nextreg(val)?;

    self.emit(make_abc(Opcode::Return, val.into(), 0, 0));
    Ok(())
  }

  fn if_stmt(&mut self, cond: Expr, blocks: (Node, Option<Node>)) -> Result<(), String> {
    let (if_block, else_block) = blocks;
    let cond = self.exp2nextreg(cond)?;

    self.emit(make_abc(Opcode::Test, cond.into(), 0, 0));
    self.emit(make_abc(Opcode::Jmp, 0, 0, 0));

    let jmp_pos = self.ni - 1;
    let mut jmp = if else_block.is_some() { 1 } else { 0 };
    let start = self.ni;

    self.walk(if_block)?;

    jmp += self.ni - start;

    self.fix_jmp(jmp_pos, false, jmp)?;

    if let Some(block) = else_block {
      self.emit(make_abc(Opcode::Jmp, 0, 0, 0));
      let jmp_pos = self.ni - 1;
      let start = self.ni;

      self.walk(block)?;
      self.fix_jmp(jmp_pos, false, self.ni - start)?;
    }

    Ok(())
  }

  fn while_stmt(&mut self, cond: Expr, block: Node) -> Result<(), String> {
    let start = self.ni;
    let cond = self.exp2nextreg(cond)?;

    self.emit(make_abc(Opcode::Test, cond.into(), 0, 0));
    self.emit(make_abc(Opcode::Jmp, 0, 0, 0));

    let jmp_pos = self.ni - 1;

    self.walk(block)?;
    let to = self.ni - start;
    self.closure.code[jmp_pos] = self.jmp(false, to - 2)?;

    let jmp = self.jmp(true, to)?;
    self.emit(jmp);

    Ok(())
  }

  fn block_stmt(&mut self, block: Vec<Node>) -> Result<(), String> {
    let nvars = self.nvars;
    self.compile(block)?;

    let close = self.nvars - nvars;

    if close != 0 {
      self.nvars -= close;
      self.emit(make_abc(Opcode::Close, close.into(), 0, 0));
    }

    Ok(())
  }

  fn exp2nextreg(&mut self, exp: Expr) -> Result<u8, String> {
    self.reserve_regs(1)?;
    let reg = self.freereg - 1;
    self.expr(exp, reg)?;
    Ok(reg)
  }

  fn expr(&mut self, exp: Expr, reg: u8) -> Result<(), String> {
    match exp {
      Expr::String(s) => self.load_const(Value::String(s), reg),
      Expr::Number(n) => self.load_const(Value::Number(n), reg),
      Expr::Name(s) => self.load_var(s, reg),
      Expr::Array(a) => self.load_array(a, reg),
      Expr::Index(obj, idx) => self.load_index(*obj, *idx, reg),

      Expr::Bool(b) => Ok(self.load_bool(b, reg)),
      Expr::Nil => Ok(self.load_nil(reg)),

      Expr::Binary(lhs, op, rhs) => self.binary(*lhs, op, *rhs, reg),
      Expr::Unary(op, exp) => self.unary(op, *exp, reg),
      Expr::Call(func, args) => self.call(*func, args, reg),
    }
  }

  fn load_array(&mut self, array: Vec<Expr>, reg: u8) -> Result<(), String> {
    let mut nelem = 0;
    let reg = reg.into();
    self.freeexp();

    for elem in array {
      self.exp2nextreg(elem)?;
      nelem += 1;
    }

    self.freereg -= nelem - 1;

    self.emit(make_abc(Opcode::NewArray, reg, reg + nelem as u16, 0));
    Ok(())
  }

  fn load_index(&mut self, obj: Expr, idx: Expr, reg: u8) -> Result<(), String> {
    self.expr(obj, reg)?;
    let idx = self.rc2nextreg(idx)?;
    let reg = reg.into();

    self.emit(make_abc(Opcode::GetArray, reg, idx, 0));
    Ok(())
  }

  fn call(&mut self, func: Expr, args: Vec<Expr>, reg: u8) -> Result<(), String> {
    self.expr(func, reg)?;

    let mut narg = 0;
    for arg in args {
      narg += 1;
      self.exp2nextreg(arg)?;
    }

    let func_reg = reg as u16;
    self.emit(make_abc(Opcode::Call, func_reg, func_reg + narg + 1, reg));
    Ok(())
  }

  fn binary(&mut self, lhs: Expr, op: BinOp, rhs: Expr, reg: u8) -> Result<(), String> {
    if op == BinOp::Assign { return self.assignment(lhs, rhs, reg) }

    let lhv = self.rc2reg(lhs, reg)?;
    let rhv = if get_mode(lhv) == 1 {
      self.rc2reg(rhs, reg)?
    } else {
      self.rc2nextreg(rhs)?
    };

    macro_rules! arith {
      ($i:ident) => {
        self.emit(make_abc(Opcode::$i, lhv, rhv, reg))
      };
    }

    macro_rules! cmp {
      ($i:ident) => {
        self.emit(make_abc(Opcode::$i, lhv, rhv, reg))
      };
    }

    match op {
      BinOp::Add => arith!(Add),
      BinOp::Sub => arith!(Sub),
      BinOp::Mul => arith!(Mul),
      BinOp::Div => arith!(Div),

      BinOp::Neq => cmp!(Neq),
      BinOp::Eq => cmp!(Eq),
      BinOp::Lt => cmp!(Lt),
      BinOp::Gt => cmp!(Gt),
      BinOp::Le => cmp!(Le),
      BinOp::Ge => cmp!(Ge),

      BinOp::Assign => {}
    }

    Ok(())
  }

  fn unary(&mut self, op: UnOp, exp: Expr, reg: u8) -> Result<(), String> {
    let exp = self.rc2nextreg(exp)?;

    match op {
      UnOp::Neg => {
        self.emit(make_abc(Opcode::Neg, reg.into(), exp, 0))
      }

      UnOp::Not => {
        self.emit(make_abc(Opcode::Not, reg.into(), exp, 0))
      }
    }

    Ok(())
  }

  fn assignment(&mut self, name: Expr, value: Expr, reg: u8) -> Result<(), String> {
    if let Expr::Name(var) = name {
      self.expr(value, reg)?;

      if let Some(var_reg) = self.get_var(var.clone()) {
        self.emit(make_abc(Opcode::Move, var_reg.into(), reg.into(), 0));
      } else {
        let c = self.resolve_const(Value::String(var))?;
        self.emit(make_abx(Opcode::SetGlobal, reg.into(), c))
      }

      Ok(())
    } else if let Expr::Index(obj, idx) = name {
      self.expr(*obj, reg)?;
      let a = self.rc2nextreg(*idx)?;
      let b = self.rc2nextreg(value)?;

      self.emit(make_abc(Opcode::SetArray, a, b, reg));
      Ok(())
    } else {
      panic!("This should be impossible!");
    }
  }

  fn rc2nextreg(&mut self, exp: Expr) -> Result<u16, String> {
    self.reserve_regs(1)?;
    let r = self.rc2reg(exp, self.freereg - 1)?;
    self.freereg -= 1;
    Ok(r)
  }

  fn rc2reg(&mut self, exp: Expr, reg: u8) -> Result<u16, String> {
    macro_rules! RC {
      ($i:ident, $v:ident) => {
        {
          let pos = self.resolve_const(Value::$i($v))?;

          if pos < u8::MAX.into() {
            return Ok(set_mode(1, pos.try_into().unwrap()))
          } else {
            self.expr(exp, reg)?;
            return Ok(reg.into());
          }
        }
      };
    }

    match exp.clone() {
      Expr::String(s) => RC!(String, s),
      Expr::Number(n) => RC!(Number, n),

      _ => {
        self.expr(exp, reg)?;
        return Ok(reg.into());
      }
    }
  }

  fn load_nil(&mut self, reg: u8) {
    self.emit(make_abc(Opcode::LoadNil, reg.into(), 0, 0))
  }

  fn load_bool(&mut self, b: bool, reg: u8) {
    self.emit(make_abc(Opcode::LoadBool, reg.into(), b.into(), 0))
  }

  fn load_var(&mut self, name: String, reg: u8) -> Result<(), String> {
    let locvar = self.get_var(name.clone());

    if let Some(pos) = locvar {
      self.emit(make_abc(Opcode::Move, reg.into(), pos.into(), 0));
    } else {
      let pos = self.resolve_const(Value::String(name))?;
      self.emit(make_abx(Opcode::GetGlobal, reg.into(), pos));
    }

    Ok(())
  }

  fn load_const(&mut self, val: Value, reg: u8) -> Result<(), String> {
    let pos = self.resolve_const(val)?;
    self.emit(make_abx(Opcode::LoadConst, reg.into(), pos));
    Ok(())
  }

  fn fix_jmp(&mut self, jmp_pos: usize, back: bool, jmp: usize) -> Result<(), String> {
    self.closure.code[jmp_pos] = self.jmp(back, jmp + 1)?;
    Ok(())
  }

  fn jmp(&mut self, back: bool, to: usize) -> Result<u32, String> {
    if to >= u16::MAX.into() {
      return Err("block is too long".into())
    }

    Ok(make_abx(Opcode::Jmp, back as u16, to.try_into().unwrap()))
  }

  fn resolve_const(&mut self, val: Value) -> Result<u16, String> {
    let mut pos: Option<u16> = None;

    for (i, val2) in self.closure.consts.iter().enumerate() {
      if val == *val2 {
        pos = Some(i.try_into().unwrap()) // this shouldn't panic
      }
    }

    if pos.is_none() {
      if self.closure.consts.len() >= u16::MAX.into() {
        return Err("constant overflow".into())
      }

      self.closure.consts.push(val);
      pos = Some((self.closure.consts.len() - 1).try_into().unwrap()); // this shouldn't panic either
    }

    Ok(pos.unwrap())
  }

  fn get_var(&mut self, name: String) -> Option<u8> {
    for var in &self.vars {
      if var.name == name {
        return Some(var.pos)
      }
    }
    None
  }

  fn freeexp(&mut self) {
    if self.freereg != 0 && self.nvars <= self.freereg {
      self.freereg -= 1;
    }
  }

  fn reserve_regs(&mut self, reg: u8) -> Result<(), String> {
    let (_, err) = self.freereg.overflowing_add(reg);

    if err {
      Err("function or expression too complex".into())
    } else {
      self.freereg += reg;
      Ok(())
    }
  }

  fn register_var(&mut self, name: String) -> Result<u8, String> {
    if self.nvars >= 255 {
      Err("too many local variables".into())
    } else {
      let pos = self.nvars;

      self.nvars += 1;
      self.vars.insert(0, VarInfo::new(name, pos));

      Ok(pos)
    }
  }

  #[inline(always)]
  fn emit(&mut self, code: u32) {
    self.ni += 1;
    self.closure.code.push(code);
    self.closure.lines.push(self.line);
  }
}