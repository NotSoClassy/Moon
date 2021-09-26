#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Token {
  String,
  Number,
  Name,
  /// default token
  None,
  Eof,
  /// used for errors
  SC(char),

  LeftParen,
  RightParen,
  LeftSquare,
  RightSquare,
  LeftBrace,
  RightBrace,
  Colon,
  Line,
  Dot,

  And,
  Or,
  Equal,
  Eq,
  Neq,
  Gt,
  Lt,
  Ge,
  Le,

  Plus,
  Dash,
  Star,
  Slash,
  Percent,
  Bang,
  Semi,
  Comma,

  Let,
  If,
  Else,
  Fn,
  For,
  Return,
  While,
  True,
  False,
  Nil
}