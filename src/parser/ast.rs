#[derive(Debug, Clone)]
pub struct Node {
  pub line: usize,
  pub stmt: Stmt
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BinOp {
  Eq,
  Neq,
  Lt,
  Gt,
  Le,
  Ge,
  Assign,
  Add,
  Sub,
  Mul,
  Div,
  Mod,
  And,
  Or
}

#[derive(Debug, PartialEq, Clone)]
pub enum UnOp {
  Neg,
  Not
}

#[derive(Debug, Clone)]
pub enum Expr {
  String(String),
  Number(f64),
  Name(String),
  AnonFn(Vec<String>, Box<Node>),
  Index(Box<Expr>, Box<Expr>),
  Bool(bool),
  Array(Vec<Expr>),
  Table(Vec<(Expr, Expr)>),
  Nil,

  Call(Box<Expr>, Vec<Expr>),
  Binary(Box<Expr>, BinOp, Box<Expr>),
  Unary(UnOp, Box<Expr>),
}

#[derive(Debug, Clone)]
pub enum Stmt {
  Let(String, Expr),
  If(Expr, Box<(Node, Option<Node>)>),
  /// First statement should be Stmt::Let or Stmt::Expr
  For(Box<Node>, Expr, Expr, Box<Node>),
  Fn(String, Vec<String>, Box<Node>),
  Return(Expr),
  While(Expr, Box<Node>),
  Block(Vec<Node>),
  Expr(Expr)
}

impl Expr {
  #[inline]
  pub fn boxed(&self) -> Box<Expr> {
    Box::new(self.clone())
  }
}

impl BinOp {
  pub fn priority(&self) -> u8 {
    match self {
      BinOp::Mul | BinOp::Div | BinOp::Mod => 7,
      BinOp::Add | BinOp::Sub => 6,
      BinOp::Gt | BinOp::Ge |
        BinOp::Lt | BinOp::Le => 5,
      BinOp::Eq | BinOp::Neq => 4,
      BinOp::And => 3,
      BinOp::Or => 2,
      BinOp::Assign => 1
    }
  }
}