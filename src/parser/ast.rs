#[derive(Debug)]
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
  Div
}

#[derive(Debug, PartialEq, Clone)]
pub enum UnOp {
  Neg,
  Not
}

#[derive(Debug, Clone, PartialEq)]
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
  If(Expr, Box<(Node, Option<Node>)>),
  /// fn body must be `Stmt::Block`
  Fn(String, Vec<String>, Box<Node>),
  While(Expr, Box<Node>),
  Block(Vec<Node>),
  Expr(Expr)
}

impl Expr {
  #[inline(always)]
  pub fn boxed(&self) -> Box<Expr> {
    Box::new(self.clone())
  }
}

impl BinOp {
  pub fn priority(&self) -> u8 {
    match self {
      BinOp::Mul | BinOp::Div => 5,
      BinOp::Add | BinOp::Sub => 4,
      BinOp::Gt | BinOp::Ge |
        BinOp::Lt | BinOp::Le => 3,
      BinOp::Eq | BinOp::Neq => 2,
      BinOp::Assign => 1
    }
  }
}

/*
fn is_literal(exp: Expr) -> bool {
  matches!(exp, Expr::Bool(_) | Expr::Number(_) | Expr::String(_) | Expr::Nil)
}

// yucky
impl Expr {
  pub fn optimize(&mut self) {
    match self.clone() {
      Expr::Binary(lhs, op, rhs) => {
        if let Expr::Number(lhn) = lhs.as_ref() {
          if let Expr::Number(rhn) = rhs.as_ref() {
            let n = match op {
              BinOp::Add => lhn + rhn,
              BinOp::Sub => lhn - rhn,
              BinOp::Mul => lhn * rhn,
              BinOp::Div => lhn / rhn,

              _ => {
                let b = match op {
                  BinOp::Lt => lhn > rhn,
                  BinOp::Le => lhn >= rhn,
                  BinOp::Gt => lhn < rhn,
                  BinOp::Ge => lhn <= rhn,
                  BinOp::Eq => rhn == lhn,
                  BinOp::Neq => rhn != lhn,

                  _ => return
                };

                *self = Expr::Bool(b);
                return
              }
            };

            *self = Expr::Number(n);
            return
          }
        }

        if is_literal(self.clone()) {
          let b = match op {
            BinOp::Eq => *rhs == *lhs,
            BinOp::Neq => *rhs != *lhs,

            _ => return
          };

          *self = Expr::Bool(b);
          return
        }
      }
      _ => {}
    }
  }
}
*/