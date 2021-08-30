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

  /*/// A B | `Reg[A] = UpValues[B]`
  GetUpVal,

  /// A B | `UpValue[A] = Reg[B]`
  SetUpVal,*/

  /// A Bx | `Reg[A] = Global[Const[Bx]]`
  GetGlobal,

  /// A Bx | `Global[Bx] = Reg[A]`
  SetGlobal,

  /// A B | `Reg[A] = [ R[A+1] .. B ]`
  NewArray,

  /// A B | `Reg[A] = Reg[A][RC(B)]
  GetArray,

  /// A B C | `Reg[C][RC(A)] = RC(B)`
  SetArray,

  /// A B C | `Reg[C] = RC[A] + RC[B]`
  Add,
  /// A B C | `Reg[C] = RC[A] - RC[B]`
  Sub,
  /// A B C | `Reg[C] = RC[A] * RC[B]`
  Mul,
  /// A B C | `Reg[C] = RC[A] / RC[B]`
  Div,
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

  /// A | `if Reg[A] then pc += 2`
  Test,

  /// A B C | `Reg[C] = Reg[A](Reg(A+1) .. Reg(B))`
  Call,

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

pub static OPMODES: &[Opmode] = &[
  Opmode::Abc,
  Opmode::Abx,
  Opmode::Abc,
  Opmode::Abc,
  Opmode::Abx,
  Opmode::Abx,
  Opmode::Abc,
  Opmode::Abc,
  Opmode::Abc,
  Opmode::Abc,
  Opmode::Abc,
  Opmode::Abc,
  Opmode::Abc,
  Opmode::Abc,
  Opmode::Abc,
  Opmode::Abc,
  Opmode::Abc,
  Opmode::Abc,
  Opmode::Abc,
  Opmode::Abc,
  Opmode::Abx,
  Opmode::Abc,
  Opmode::Abc,
  Opmode::Abc,
  Opmode::Abc,
  Opmode::Abc
];

#[derive(Clone, Copy)]
pub enum Opmode {
  Abc,
  Abx
}