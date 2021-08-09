
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
  GetUpVal,*/

  /// A Bx | `Reg[A] = Global[Const[Bx]]`
  GetGlobal,

  /// A Bx | `Global[Bx] = Reg[A]`
  SetGlobal,

  /*/// A B | `UpValue[B] = Reg[A]`
  SetUpVal,*/

  /// A B C | `Reg[A] = Reg[B] + Reg[C]`
  Add,
  /// A B C | `Reg[A] = Reg[B] - Reg[C]`
  Sub,
  /// A B C | `Reg[A] = Reg[B] * Reg[C]`
  Mul,
  /// A B C | `Reg[A] = Reg[B] / Reg[C]`
  Div,
  /// A B C | `Reg[A] = Reg[B] == Reg[C]`
  Eq,
  /// A B C | `Reg[A] = Reg[B] != Reg[C]`
  Neq,
  /// A B C | `Reg[A] = Reg[B] < Reg[C]`
  Gt,
  /// A B C | `Reg[A] = Reg[B] <= Reg[C]`
  Ge,
  /// A B C | `Reg[A] = Reg[B] > Reg[C]`
  Lt,
  /// A B C | `Reg[A] = Reg[B] >= Reg[C]`
  Le,

  /// A B | `Reg[A] = -Reg[B]`
  Neg,
  /// A B | `Reg[A] = !Reg[B]`
  Not,

  /// A Bx | `if A then pc -= Bx else pc += Bx`
  Jmp,

  /// A | `if Reg[A] then pc++`
  Test,

  /// A B C | `Reg[C] = Reg[A](Reg(A+1) .. Reg(B))`
  Call,

  /// A | `Reg[A..] = nil`
  Close
}

impl From<u8> for Opcode {
  fn from(n: u8) -> Opcode {
    unsafe { std::mem::transmute(n) }
  }
}