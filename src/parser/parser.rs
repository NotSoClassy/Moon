use crate::lexer::{ Lexer, Token };
use crate::parser::ast::{ Stmt, Expr, UnOp, BinOp };


pub struct Parser {
  lex: Lexer,
  token: Token,
  pub nodes: Vec<Stmt>
}

impl Parser {
  pub fn new(code: String, name: String) -> Self {
    let mut lexer = Lexer::new(code, name);
    lexer.lex_next();

    Parser {
      token: lexer.token,
      lex: lexer,
      nodes: Vec::new()
    }
  }

  pub fn parse(&mut self) -> Result<(), String> {
    if self.token == Token::Eof { return Ok(()) }

    while self.token != Token::Eof {
      let node = self.stmt()?;
      self.nodes.push(node);
    }

    Ok(())
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

  fn unary_exp(&mut self) -> Option<UnOp> {
    let op = match self.token {
      Token::Dash => Some(UnOp::Neg),
      Token::Bang => Some(UnOp::Not),
      _ => None
    };

    if op.is_some() { self.next() }
    op
  }

  fn prefix_exp(&mut self) -> Result<Expr, String> {
    match self.token {
      Token::True | Token::False => Ok(Expr::Bool(self.token == Token::True)),
      Token::Number => Ok(Expr::Number(self.lex.buf.parse().unwrap())), // this shouldn't panic
      Token::String => Ok(Expr::String(self.lex.buf.clone())),
      Token::Name => Ok(Expr::Name(self.lex.buf.clone())),
      Token::Nil => Ok(Expr::Nil),

      Token::LeftParen => {
        self.next();
        let exp = self.expr();
        self.check(Token::RightParen)?;
        exp
      }

      _ => Err(self.error("unexpected token", self.token))
    }
  }

  fn expr(&mut self) -> Result<Expr, String> {
    let unary = self.unary_exp();
    let mut exp = self.prefix_exp()?;
    self.next();

    if let Some(op) = unary {
      exp = Expr::Unary(op, Box::new(exp))
    }

    macro_rules! bin_expr {
      ($op:ident) => {
        {
          self.next();
          let e = Expr::Binary(Box::new(exp), BinOp::$op, Box::new(self.expr()?));
          return Ok(e)
        }
      };
    }

    match self.token {
      Token::LeftParen => self.call_expr(exp),
      Token::Plus => bin_expr!(Add),
      Token::Dash => bin_expr!(Sub),
      Token::Star => bin_expr!(Mul),
      Token::Slash => bin_expr!(Div),
      Token::Eq => bin_expr!(Eq),
      Token::Neq => bin_expr!(Neq),
      Token::Lt => bin_expr!(Lt),
      Token::Le => bin_expr!(Le),
      Token::Gt => bin_expr!(Gt),
      Token::Ge => bin_expr!(Ge),

      Token::Equal => {
        if !matches!(exp, Expr::Name(_)) {
          Err(self.error("unexpected token", self.token))
        } else {
          bin_expr!(Assign)
        }
      },

      _ => Ok(exp)
    }
  }

  fn call_expr(&mut self, prefix: Expr) -> Result<Expr, String> {
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

    Ok(Expr::Call(Box::new(prefix), args))
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
    let mut r#else: Option<Stmt> = None;

    if self.test_next(Token::Else) {
      r#else = Some(self.stmt()?);
    }

    Ok(Stmt::If(cond, Box::new((body, r#else))))
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

    Ok(Stmt::Fn(name, params, Box::new(body)))
  }

  fn block_stmt(&mut self) -> Result<Stmt, String> {
    self.check_next(Token::LeftBrace)?;

    let mut body = Vec::new();

    while self.token != Token::RightBrace && self.token != Token::Eof {
      body.push(self.stmt()?);
    }
    self.check_next(Token::RightBrace)?;

    Ok(Stmt::Block(body))
  }

  fn while_stmt(&mut self) -> Result<Stmt, String> {
    self.expect_next(Token::LeftParen)?;

    let cond = self.expr()?;

    self.check_next(Token::RightParen)?;

    let body = self.stmt()?;

    Ok(Stmt::While(cond, Box::new(body)))
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
    self.lex.lex_next();
    self.token = self.lex.token;
  }

  fn token2str(&self, token: Token) -> String {
    self.lex.token2str(token)
  }

  fn error(&self, err: &str, token: Token) -> String {
    self.lex.error(err, token)
  }
}