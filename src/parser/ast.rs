#[derive(Debug, PartialEq)]
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
  Div
}

#[derive(Debug)]
pub enum UnOp {
  Neg,
  Not
}

#[derive(Debug)]
pub enum Expr {
  String(String),
  Number(f64),
  Name(String),
  Bool(bool),
  Nil,

  Call(Box<Expr>, Vec<Expr>),
  Binary(Box<Expr>, BinOp, Box<Expr>),
  Unary(UnOp, Box<Expr>),
}

#[derive(Debug)]
pub enum Stmt {
  Let(String, Expr),
  If(Expr, Box<(Stmt, Option<Stmt>)>),
  /// fn body must be `Stmt::Block`
  Fn(String, Vec<String>, Box<Stmt>),
  While(Expr, Box<Stmt>),
  Block(Vec<Stmt>),
  Expr(Expr)
}

/*impl Expr {
  pub fn optimize(&mut self) {
    match self {
      Expr::Binary(lhs, op, rhs) => {
        if let Expr::Number(lhn) = lhs.as_ref() {
          if let Expr::Number(rhn) = rhs.as_ref() {
            let n = match op {
              BinOp::Add => lhn + rhn,
              BinOp::Sub => lhn - rhn,
              BinOp::Mult => lhn * rhn,
              BinOp::Div => lhn / rhn,

              _ => return
            };

            *self = Expr::Number(n);
          }
        }
      }
      _ => {}
    }
  }
}*/