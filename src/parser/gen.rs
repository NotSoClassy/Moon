use std::convert::TryInto;

use crate::common::{ Closure, Opcode, Value };
use crate::parser::ast::{
  Stmt, Expr, UnOp, BinOp
};

#[derive(Debug)]
pub enum CompileError {
  ConstantOverflow,
}

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

pub struct FuncState {
  pub nvars: u8,
  pub freereg: u8,
  pub vars: Vec<VarInfo>,
  pub closure: Closure
}

impl FuncState {
  pub fn new() -> Self {
    FuncState {
      nvars: 0,
      freereg: 0,
      vars: Vec::new(),
      closure: Closure::new()
    }
  }
}

pub struct Compiler {
  pub fs: FuncState,
}

impl Compiler {
  pub fn new() -> Self {
    Compiler {
      fs: FuncState::new()
    }
  }

  pub fn compile(&mut self, ast: Vec<Stmt>) -> Result<(), CompileError> {
    for node in ast {
      self.walk(node)?;
      self.fs.freereg = self.fs.nvars;
    }
    Ok(())
  }

  pub fn walk(&mut self, stmt: Stmt) -> Result<(), CompileError> {
    match stmt {
      Stmt::Let(name, value) => self.let_stmt(name, value),
      Stmt::If(cond, stmts) => self.if_stmt(cond, *stmts),
      Stmt::While(cond, body) => self.while_stmt(cond, *body),
      Stmt::Block(code) => self.block_stmt(code),
      Stmt::Fn(name, params, body) => self.fn_stmt(name, params, *body),

      Stmt::Expr(exp) => {
        self.exp2freereg(exp)?;
        Ok(())
      }
    }
  }

  fn walk_len(&mut self, stmt: Stmt) -> Result<usize, CompileError> {
    let start = self.fs.closure.code.len();
    self.walk(stmt)?;
    Ok((self.fs.closure.code.len() - start) / 4)
  }

  fn block_stmt(&mut self, code: Vec<Stmt>) -> Result<(), CompileError> {
    let nvars = self.fs.nvars;
    self.compile(code)?;

    let close = self.fs.nvars - nvars;

    if close != 0 {
      self.fs.nvars -= close;
      self.emit_abc(Opcode::Close, close, 0, 0);
    }

    Ok(())
  }

  fn fn_stmt(&mut self, name: String, params: Vec<String>, body: Stmt) -> Result<(), CompileError> {
    let mut compiler = Compiler::new();
    compiler.fs.closure.name = name.clone();
    compiler.fs.closure.nparams = params.len() as u8;

    for param in params {
      compiler.new_var(param);
    }

    compiler.walk(body)?;

    let closure = Value::Closure(compiler.fs.closure);

    let var = self.new_var(name);
    self.load_const(closure, var)?;

    Ok(())
  }

  fn let_stmt(&mut self, name: String, value: Expr) -> Result<(), CompileError> {
    let reg = self.new_var(name);
    self.expr(value, reg)
  }

  fn while_stmt(&mut self, cond: Expr, block: Stmt) -> Result<(), CompileError> {
    let start = self.fs.closure.code.len();
    let reg = self.exp2freereg(cond)?;

    self.emit_abc(Opcode::Test, reg, 0, 0);
    self.emit_abc(Opcode::Jmp, 0, 0, 0);

    let jmp_pos = self.fs.closure.code.len();
    let pos = self.walk_len(block)? + 2;

    self.patch_jmp(jmp_pos, pos.try_into().unwrap());

    self.emit_abx(Opcode::Jmp, 1, (((self.fs.closure.code.len() - start) / 4) + 1) as u16);

    Ok(())
  }

  fn if_stmt(&mut self, cond: Expr, blocks: (Stmt, Option<Stmt>)) -> Result<(), CompileError> {
    let (truthy, falsey) = blocks;
    let reg = self.exp2freereg(cond)?;

    self.emit_abc(Opcode::Test, reg, 0, 0);
    self.emit_abc(Opcode::Jmp, 0, 0, 0);

    let jmp_pos = self.fs.closure.code.len();
    let mut len = self.walk_len(truthy)?;

    if falsey.is_some() { len += 1 }

    self.patch_jmp(jmp_pos, len.try_into().unwrap());

    if let Some(stmt) = falsey {
      self.emit_abc(Opcode::Jmp, 0, 0, 0);

      let jmp_pos = self.fs.closure.code.len();
      let len = self.walk_len(stmt)?;

      self.patch_jmp(jmp_pos, len.try_into().unwrap());
    }

    Ok(())
  }

  fn expr(&mut self, exp: Expr, reg: u8) -> Result<(), CompileError> {
    match exp {
      Expr::String(str) => self.load_const(Value::String(str), reg),
      Expr::Number(n) => self.load_const(Value::Number(n), reg),
      Expr::Bool(b) => Ok(self.load_bool(b, reg)),
      Expr::Name(str) => self.load_var(str, reg),
      Expr::Nil => Ok(self.load_nil(reg)),

      Expr::Binary(lhs, op, rhs) => self.bin_expr(*lhs, op, *rhs, reg),
      Expr::Unary(op, exp) => self.un_expr(op, *exp, reg),
      Expr::Call(func, args) => self.call(*func, args, reg),
    }
  }

  fn bin_expr(&mut self, lhs: Expr, op: BinOp, rhs: Expr, res_reg: u8) -> Result<(), CompileError> {
    if op == BinOp::Assign {
      if let Expr::Name(name) = lhs {
        self.expr(rhs, res_reg)?;
        self.assign(name, res_reg)?;
        return Ok(())
      } else {
        panic!("this should be impossible")
      }
    }

    let lh_reg = self.exp2freereg(lhs)?;
    let rh_reg = self.exp2freereg(rhs)?;

    macro_rules! bin {
      ($n:ident) => {
        Ok(self.emit_abc(Opcode::$n, res_reg, lh_reg, rh_reg))
      };
    }

    match op {
      BinOp::Add => bin!(Add),
      BinOp::Sub => bin!(Sub),
      BinOp::Mul => bin!(Mul),
      BinOp::Div => bin!(Div),

      BinOp::Neq => bin!(Neq),
      BinOp::Eq => bin!(Eq),
      BinOp::Gt => bin!(Gt),
      BinOp::Ge => bin!(Ge),
      BinOp::Lt => bin!(Lt),
      BinOp::Le => bin!(Le),

      BinOp::Assign => Ok(())
    }
  }

  fn un_expr(&mut self, op: UnOp, exp: Expr, reg: u8) -> Result<(), CompileError> {
    self.expr(exp, reg)?;
    let op = match op {
      UnOp::Neg => Opcode::Neg,
      UnOp::Not => Opcode::Not
    };

    self.emit_abc(op, reg, reg, 0);
    Ok(())
  }

  fn call(&mut self, func: Expr, args: Vec<Expr>, reg_reg: u8) -> Result<(), CompileError> {
    let base = self.exp2freereg(func)?;

    for arg in args {
      self.exp2freereg(arg)?;
    }

    self.emit_abc(Opcode::Call, base, self.fs.freereg, reg_reg);
    Ok(())
  }

  fn load_bool(&mut self, bool: bool, reg: u8) {
    self.emit_abc(Opcode::LoadBool, reg, bool as u8, 0)
  }

  fn load_const(&mut self, val: Value, reg: u8) -> Result<(), CompileError> {
    let pos = self.resolve_const(val)?;
    self.emit_abx(Opcode::LoadConst, reg, pos);
    Ok(())
  }

  fn load_var(&mut self, name: String, reg: u8) -> Result<(), CompileError> {
    let locvar = self.get_var(name.clone());

    if let Some(pos) = locvar {
      self.emit_abc(Opcode::Move, reg, pos, 0);
    } else {
      let pos = self.resolve_const(Value::String(name))?;
      self.emit_abx(Opcode::GetGlobal, reg, pos);
    }

    Ok(())
  }

  fn load_nil(&mut self, reg: u8) {
    self.emit_abc(Opcode::LoadNil, reg, 0, 0);
  }

  fn assign(&mut self, var: String, val: u8) -> Result<(), CompileError> {
    if let Some(reg) = self.get_var(var.clone()) {
      self.emit_abc(Opcode::Move, reg, val, 0);
    } else {
      let cnst = self.resolve_const(Value::String(var))?;
      self.emit_abx(Opcode::SetGlobal, val, cnst)
    }

    Ok(())
  }

  fn resolve_const(&mut self, val: Value) -> Result<u16, CompileError> {
    let mut pos: Option<usize> = None;
    for (i, r#const) in self.fs.closure.consts.iter().enumerate() {
      if val == *r#const {
        pos = Some(i);
        break
      }
    }

    if pos.is_none() {
      self.fs.closure.consts.push(val);
      pos = Some(self.fs.closure.consts.len() - 1)
    }

    let pos = pos.unwrap();

    if pos > u16::MAX.into() {
      Err(CompileError::ConstantOverflow)
    } else {
      Ok(pos as u16)
    }
  }

  fn exp2freereg(&mut self, exp: Expr) -> Result<u8, CompileError> {
    let reg = self.fs.freereg;
    self.expr(exp, reg)?;
    self.fs.freereg += 1;
    Ok(reg)
  }

  fn patch_jmp(&mut self, jmp_pos: usize, jmp: isize) {
    let code = &mut self.fs.closure.code;

    let neg = jmp.is_negative() as u8;
    let pos = jmp.unsigned_abs() as u16;
    let b = (pos >> 8 & 0xFF) as u8;
    let c = (pos & 0xFF) as u8;

    code[jmp_pos - 3] = neg;
    code[jmp_pos - 2] = b;
    code[jmp_pos - 1] = c;
  }

  pub(super) fn new_var(&mut self, name: String) -> u8 {
    let pos = self.fs.nvars;

    self.fs.nvars += 1;
    self.fs.vars.insert(0, VarInfo::new(name, pos));
    pos
  }

  fn get_var(&self, name: String) -> Option<u8> {
    for var in &self.fs.vars {
      if var.name == name {
        return Some(var.pos)
      }
    }
    None
  }

  fn emit_abx(&mut self, op: Opcode, a: u8, bx: u16) {
    let b = (bx >> 8 & 0xFF) as u8;
    let c = (bx & 0xFF) as u8;
    self.emit_abc(op, a, b, c);
  }

  fn emit_abc(&mut self, op: Opcode, a: u8, b: u8, c: u8) {
    let code = &mut self.fs.closure.code;
    code.push(op as u8);
    code.push(a);
    code.push(b);
    code.push(c);
  }
}