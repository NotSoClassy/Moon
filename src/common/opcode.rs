// mostly copied from Lua
#[derive(Debug, Clone, PartialEq)]
pub enum Opcode {

  /// A B | Reg[A] = Reg[B]
  Move,

  /// A Bx | `Reg[A] = Const[Bx]`
  LoadConst,

  /// A B | `Reg[A] = B as bool`
  LoadBool,

  /// A | `Reg[A] = nil`
  LoadNil,

  /// A B C | `Reg[A] = UpValues[B]`
  GetUpVal,

  /// A B | `UpValue[A] = Reg[B]`
  SetUpVal,

  /// A Bx | `Reg[A] = Global[Const[Bx]]`
  GetGlobal,

  /// A Bx | `Global[Bx] = Reg[A]`
  SetGlobal,

  /// A B | `Reg[A] = { R[A]: R[A+1], .. Reg[N]: Reg[B-1] }
  NewTable,

  /// A B | `Reg[A] = [ Reg[A+1] .. Reg[B] ]`
  NewArray,

  /// A B | `Reg[A] = Reg[A][RC(B)]
  GetObj,

  /// A B C | `Reg[C][RC(A)] = RC(B)`
  SetObj,

  /// A B C | `Reg[C] = RC[A] + RC[B]`
  Add,
  /// A B C | `Reg[C] = RC[A] - RC[B]`
  Sub,
  /// A B C | `Reg[C] = RC[A] * RC[B]`
  Mul,
  /// A B C | `Reg[C] = RC[A] / RC[B]`
  Div,
  /// A B C | `Reg[C] = RC[A] % RC[B]`
  Mod,


  /// A B C | `Reg[C] = RC[A] == RC[B]`
  Eq,
  /// A B C | `Reg[C] = RC[A] != RC[B]`
  Neq,
  /// A B C | `Reg[C] = RC[A] < RC[B]`
  Gt,
  /// A B C | `Reg[C] = RC[A] <= RC[B]`
  Ge,
  /// A B C | `Reg[C] = RC[A] > RC[B]`
  Lt,
  /// A B C | `Reg[C] = RC[A] >= RC[B]`
  Le,

  /// A B | `Reg[A] = -RC[B]`
  Neg,
  /// A B | `Reg[A] = !RC[B]`
  Not,

  /// A Bx | `if A then pc -= Bx else pc += Bx`
  Jmp,

  /// A B | `if Reg[A] == (B == 0) then pc += 2`
  Test,

  /// A B C | `Reg[C] = Reg[A](Reg(A+1) .. Reg(B))`
  Call,

  /// A Bx | `Reg[A] = Consts[Bx]`
  Closure,

  /// A B | `return B && nil || Reg[A]`
  Return,

  /// A | `Reg[A..] = nil`
  Close
}

impl From<u8> for Opcode {
  fn from(n: u8) -> Opcode {
    unsafe { std::mem::transmute(n) }
  }
}

pub static OPMODES: &[(&'static str, Opmode)] = &[
  ("Move      ", Opmode::Abc),
  ("LoadConst ", Opmode::Abx),
  ("LoadBool  ", Opmode::Abc),
  ("LoadNil   ", Opmode::Abc),
  ("GetUpVal  ", Opmode::Abc),
  ("SetUpVal  ", Opmode::Abc),
  ("GetGlobal ", Opmode::Abx),
  ("SetGlobal ", Opmode::Abx),
  ("NewTable  ", Opmode::Abc),
  ("NewArray  ", Opmode::Abc),
  ("GetObj    ", Opmode::Abc),
  ("SetObj    ", Opmode::Abc),
  ("Add       ", Opmode::Abc),
  ("Sub       ", Opmode::Abc),
  ("Mul       ", Opmode::Abc),
  ("Div       ", Opmode::Abc),
  ("Mod       ", Opmode::Abc),
  ("Eq        ", Opmode::Abc),
  ("Neq       ", Opmode::Abc),
  ("Gt        ", Opmode::Abc),
  ("Ge        ", Opmode::Abc),
  ("Lt        ", Opmode::Abc),
  ("Le        ", Opmode::Abc),
  ("Neg       ", Opmode::Abc),
  ("Not       ", Opmode::Abc),
  ("Jmp       ", Opmode::Abx),
  ("Test      ", Opmode::Abc),
  ("Call      ", Opmode::Abc),
  ("Closure   ", Opmode::Abx),
  ("Return    ", Opmode::Abc),
  ("Close     ", Opmode::Abc)
];

#[derive(Clone, Copy)]
pub enum Opmode {
  Abc,
  Abx
}