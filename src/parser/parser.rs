use crate::parser::ast::{ Stmt, Expr, UnOp, BinOp, Node };
use crate::lexer::{ Lexer, Token };
use std::process::exit;

pub struct Parser {
  lex: Lexer,
  token: Token,
  pub nodes: Vec<Node>
}

impl Parser {
  pub fn new(code: String, name: String) -> Self {
    let mut lexer = Lexer::new(code, name);
    lexer.lex_next().unwrap();

    Parser {
      token: lexer.token,
      lex: lexer,
      nodes: Vec::new()
    }
  }

  pub fn parse(&mut self) -> Result<(), String> {
    if self.token == Token::Eof { return Ok(()) }

    while self.token != Token::Eof {
      let stmt = self.stmt()?;

      self.nodes.push(self.to_node(stmt));
    }

    Ok(())
  }

  fn to_node(&self, stmt: Stmt) -> Node {
    Node {
      line: self.lex.line,
      stmt
    }
  }

  fn stmt(&mut self) -> Result<Stmt, String> {
    macro_rules! stmt {
      ($i:expr) => {
        { let res = $i; self.test_next(Token::Semi); return Ok(res) }
      };
    }

    match self.token {
      Token::LeftBrace => stmt!(self.block_stmt()?),
      Token::While => stmt!(self.while_stmt()?),
      Token::Let => stmt!(self.let_stmt()?),
      Token::If => stmt!(self.if_stmt()?),
      Token::Fn => stmt!(self.fn_stmt()?),

      _ => stmt!(Stmt::Expr(self.expr()?))
    }
  }

  fn let_stmt(&mut self) -> Result<Stmt, String> {
    self.expect(Token::Name)?;

    let name = self.lex.buf.clone();

    self.expect(Token::Equal)?;
    self.next();

    let value = self.expr()?;
    Ok(Stmt::Let(name, value))
  }

  fn if_stmt(&mut self) -> Result<Stmt, String> {
    self.expect_next(Token::LeftParen)?;

    let cond = self.expr()?;

    self.check_next(Token::RightParen)?;

    let body = self.stmt()?;
    let mut r#else: Option<Node> = None;

    if self.test_next(Token::Else) {
      let stmt = self.stmt()?;
      r#else = Some(self.to_node(stmt));
    }

    Ok(Stmt::If(cond, Box::new((self.to_node(body), r#else))))
  }

  fn fn_stmt(&mut self) -> Result<Stmt, String> {
    self.expect(Token::Name)?;

    let name = self.lex.buf.clone();

    self.expect_next(Token::LeftParen)?;

    let mut params = Vec::new();

    if self.token != Token::RightParen {
      loop {
        self.check(Token::Name)?;
        params.push(self.lex.buf.clone());
        self.next();
        if self.token != Token::Comma { break }
        self.next();
      }
    }

    self.check_next(Token::RightParen)?;

    let body = self.block_stmt()?;

    Ok(Stmt::Fn(name, params, Box::new(self.to_node(body))))
  }

  fn while_stmt(&mut self) -> Result<Stmt, String> {
    self.expect_next(Token::LeftParen)?;

    let cond = self.expr()?;

    self.check_next(Token::RightParen)?;

    let body = self.stmt()?;

    Ok(Stmt::While(cond, Box::new(self.to_node(body))))
  }

  fn block_stmt(&mut self) -> Result<Stmt, String> {
    self.check_next(Token::LeftBrace)?;

    let mut body = Vec::new();

    while self.token != Token::RightBrace && self.token != Token::Eof {
      let stmt = self.stmt()?;
      body.push(self.to_node(stmt));
    }
    self.check_next(Token::RightBrace)?;

    Ok(Stmt::Block(body))
  }

  fn prefix_expr(&mut self) -> Result<Expr, String> {
    match self.token {
      Token::LeftParen => {
        self.next();
        let exp = self.expr();
        self.check(Token::RightParen)?;
        exp
      }

      Token::Name => Ok(Expr::Name(self.lex.buf.clone())),

      _ => Err(self.error("unexpected token", self.token))
    }
  }

  fn primary_expr(&mut self) -> Result<Expr, String> {
    let mut exp = self.prefix_expr()?;
    self.next();

    loop {
      match self.token {
        // index exp, if some sort of hashmap or array implemented

        Token::LeftParen => {
          exp = self.call(exp)?;
        }

        _ => return Ok(exp)
      }
    }
  }

  fn simple_expr(&mut self) -> Result<Expr, String> {
    macro_rules! simple {
      ($t:ident, $v:expr) => {
        { let v = $v; self.next(); Ok(Expr::$t(v)) }
      };
    }

    match self.token {
      Token::True | Token::False => simple!(Bool, self.token == Token::True),
      Token::String => simple!(String, self.lex.buf.clone()),
      Token::Number => simple!(Number, self.lex.buf.parse().unwrap()), // this shouldn't error
      Token::Nil => { self.next(); Ok(Expr::Nil) },

      _ => self.primary_expr()
    }
  }

  fn sub_expr(&mut self, priority: u8) -> Result<Expr, String> {
    let unop = self.get_unop();

    let mut left = if let Some(op) = unop {
      self.next();
      Expr::Unary(op, self.simple_expr()?.boxed())
    } else {
      self.simple_expr()?
    };

    while let Some(op) = self.get_binop() {
      if op.priority() > priority {
        self.next();

        let right = self.sub_expr(op.priority())?;
        left = Expr::Binary(left.boxed(), op, right.boxed());

        self.check_bin_exp(left.clone())?
      } else {
        break
      }
    }

    Ok(left)
  }

  #[inline(always)]
  pub fn expr(&mut self) -> Result<Expr, String> {
    self.sub_expr(0)
  }

  fn call(&mut self, func: Expr) -> Result<Expr, String> {
    self.next();
    let mut args = Vec::new();

    if self.token != Token::RightParen {
      args.push(self.expr()?);

      while self.token == Token::Comma {
        self.next();
        args.push(self.expr()?)
      }
    }

    self.check_next(Token::RightParen)?;

    Ok(Expr::Call(func.boxed(), args))
  }

  fn get_unop(&self) -> Option<UnOp> {
    match self.token {
      Token::Bang => Some(UnOp::Not),
      Token::Dash => Some(UnOp::Neg),
      _ => None
    }
  }

  fn get_binop(&self) -> Option<BinOp> {
    match self.token {
      Token::Neq => Some(BinOp::Neq),
      Token::Eq => Some(BinOp::Eq),
      Token::Ge => Some(BinOp::Ge),
      Token::Gt => Some(BinOp::Gt),
      Token::Le => Some(BinOp::Le),
      Token::Lt => Some(BinOp::Lt),

      Token::Plus => Some(BinOp::Add),
      Token::Dash => Some(BinOp::Sub),
      Token::Star => Some(BinOp::Mul),
      Token::Slash => Some(BinOp::Div),
      Token::Equal => Some(BinOp::Assign),

      _ => None
    }
  }

  // util functions

  fn check_bin_exp(&self, e: Expr) -> Result<(), String> {
    if let Expr::Binary(lhs, op, _rhs) = e {
      match op {
        BinOp::Assign => {
          if !matches!(*lhs, Expr::Name(_)) {
            return Err(self.error("unexpected token", Token::Equal))
          }
        }

        _ => {}
      }
    }
    Ok(())
  }

  /*fn test(&mut self, token: Token) -> bool {
    return self.token == token
  }*/

  fn test_next(&mut self, token: Token) -> bool {
    if self.token == token {
      self.next();
      return true
    }
    false
  }

  fn check(&self, token: Token) -> Result<(), String> {
    if self.token != token {
      return Err(self.error_expected(token))
    }
    Ok(())
  }

  fn check_next(&mut self, token: Token) -> Result<(), String> {
    let res = self.check(token);
    if res.is_ok() { self.next() }
    res
  }

  fn expect(&mut self, token: Token) -> Result<(), String> {
    self.next();
    self.check(token)
  }

  fn expect_next(&mut self, token: Token) -> Result<(), String> {
    self.next();
    self.check_next(token)
  }

  fn error_expected(&self, expected: Token) -> String {
    self.error(&format!("expected '{}'", self.token2str(expected)), self.token)
  }

  fn next(&mut self) {
    let res = self.lex.lex_next();

    if let Err(e) = res {
      eprintln!("{}", e);
      drop(self);
      exit(1);
    }

    self.token = self.lex.token;
  }

  fn token2str(&self, token: Token) -> String {
    self.lex.token2str(token)
  }

  fn error(&self, err: &str, token: Token) -> String {
    self.lex.error(err, token)
  }
}
