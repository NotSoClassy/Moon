use crate::lexer::Token;

pub struct Lexer {
  src: Vec<char>,
  current: char,
  pub name: String,
  pub pos: usize,
  pub line: usize,
  pub buf: String,
  pub token: Token
}

fn resolve(res: Result<(), String>, token: Token) -> Result<Token, String> {
  if let Err(e) = res {
    return Err(e.to_string())
  }
  Ok(token)
}

impl Lexer {
  pub fn new(src: String, name: String) -> Self {
    let src = src.chars().collect::<Vec<char>>();
    let current = src[0];

    Lexer {
      src,
      name,
      pos: 0,
      line: 1,
      buf: String::new(),
      token: Token::None,
      current,
    }
  }

  pub fn lex_next(&mut self) -> Result<(), String> {
    let res = self.lex();

    if let Err(e) = res {
      Err(e)
    } else {
      self.token = res.unwrap();
      Ok(())
    }
  }

  fn lex(&mut self) -> Result<Token, String> {
    macro_rules! next_ret {
      ($tkn:expr) => {
        { self.next(); Ok($tkn) }
      };
    }

    macro_rules! cmp_op {
      ($id:expr, $id2:expr) => {{
        self.next();
        if self.current != '=' {
          return Ok($id)
        } else {
          self.next(); return Ok($id2)
        }}
      };
    }

    self.buf.clear();

    match self.current {
      '(' => next_ret!(Token::LeftParen),
      ')' => next_ret!(Token::RightParen),
      '[' => next_ret!(Token::LeftSquare),
      ']' => next_ret!(Token::RightSquare),
      '{' => next_ret!(Token::LeftBrace),
      '}' => next_ret!(Token::RightBrace),
      '+' => next_ret!(Token::Plus),
      '-' => next_ret!(Token::Dash),
      '*' => next_ret!(Token::Star),
      ';' => next_ret!(Token::Semi),
      ',' => next_ret!(Token::Comma),
      '.' => next_ret!(Token::Dot),
      ':' => next_ret!(Token::Colon),
      '%' => next_ret!(Token::Percent),

      '&' => {
        self.next();
        if self.current == '&' {
          self.next();
          Ok(Token::And)
        } else {
          Err(self.error_near("unexpected token", Token::SC('&')))
        }
      }

      '|' => {
        self.next();
        if self.current == '|' {
          self.next();
          Ok(Token::Or)
        } else {
          Ok(Token::Line)
        }
      }

      '/' => {
        self.next();
        if self.current == '/' || self.current == '*' {
          self.comment()?;
          self.lex()
        } else {
          return Ok(Token::Slash)
        }
      },

      '=' => cmp_op!(Token::Equal, Token::Eq),
      '<' => cmp_op!(Token::Gt, Token::Ge),
      '>' => cmp_op!(Token::Lt, Token::Le),
      '!' => cmp_op!(Token::Bang, Token::Neq),

      '\0' => Ok(Token::Eof),

      '\'' | '"' => resolve(self.read_string(self.current), Token::String),


      '\n' => {
        self.line += 1;
        self.next();
        self.lex()
      }

      ' ' | '\t' | '\r' => {
        self.next();
        self.lex()
      }

      _ => {
        if self.is_ident() {
          self.read_name();

          if let Some(tkn) = self.is_keyword() {
            Ok(tkn)
          } else {
            Ok(Token::Name)
          }
        } else if self.is_num() {
          resolve(self.read_number(), Token::Number)
        } else {
          Err(self.error_near("unexpected token", Token::SC(self.current)))
        }
      }
    }
  }

  fn comment(&mut self) -> Result<(), String> {
    match self.current {
      '*' => {
        self.next();
        loop {
          if self.current == '\0' { return Err(self.error_near("unfinished comment", Token::Eof)) }
          if self.current == '*' {
            self.next();
            if self.current == '/' {
              self.next();
              return Ok(())
            }
          }
          self.next();
        }
      },

      '/' => {
        while self.current != '\n' && self.current != '\0' {
          self.next();
        }
        self.next();
        Ok(())
      },

      _ => panic!("this is impossible")
    }
  }

  fn read_name(&mut self) {
    self.buf.push(self.current);
    self.next();

    while self.is_ident() || self.is_num() {
      self.buf.push(self.current);
      self.next();
    }
  }

  fn read_string(&mut self, quote: char) -> Result<(), String> {
    self.buf.push(self.current);
    self.next();

    while self.current != quote {
      match self.current {
        c @ ('\0' | '\n') => {
          let tkn = if c == '\0' { Token::Eof } else { Token::String };

          return Err(self.error_near("unfinished string", tkn))
        }

        '\\' => {
          self.next();

          let c = match self.current {
            '\n' => {
              self.line += 1;
              self.next();
              ""
            }

            'n' | 'r' => "\n",
            't' => "\t",
            '\'' => "'",
            '"' => "\"",
            '\\' => "\\",
            '\0' => continue,

            _ => {
              return Err(self.error_near("invalid escape", Token::String))
            }
          };

          self.next();
          self.buf += c;
        }

        _ => {
          self.buf.push(self.current);
          self.next();
        }
      }
    }

    self.next();
    self.buf = self.buf[1..].to_string();
    Ok(())
  }

  fn read_number(&mut self) -> Result<(), String> {
    macro_rules! get_num_char {
      () => {
        while self.is_num() || self.current == '_' {
          if self.current == '_' { self.next(); continue }
          self.buf.push(self.current);
          self.next();
        }
      };
    }

    get_num_char!();
    if self.current == '.' {
      self.buf.push(self.current);
      self.next();
      get_num_char!();
    }

    if self.is_alnum() {
      Err(self.error_near("malformed number", self.token))
    } else {
      Ok(())
    }
  }

  fn next(&mut self) {
    self.pos += 1;

    if self.pos >= self.src.len() {
      self.current = '\0'
    } else {
      self.current = self.src[self.pos]
    }
  }

  #[inline]
  fn is_alpha(&self) -> bool {
    matches!(self.current, 'a' ..= 'z' | 'A' ..= 'Z')
  }

  #[inline]
  fn is_num(&self) -> bool {
    matches!(self.current, '0' ..= '9')
  }

  #[inline]
  fn is_alnum(&self) -> bool {
    self.is_num() || self.is_alpha()
  }

  #[inline]
  fn is_ident(&self) -> bool {
    self.is_alpha() || self.current == '_'
  }

  fn is_keyword(&self) -> Option<Token> {
    match self.buf.as_str() {
      "let" => Some(Token::Let),
      "if" => Some(Token::If),
      "else" => Some(Token::Else),
      "fn" => Some(Token::Fn),
      "return" => Some(Token::Return),
      "while" => Some(Token::While),
      "true" => Some(Token::True),
      "false" => Some(Token::False),
      "nil" => Some(Token::Nil),
      "for" => Some(Token::For),

      _ => None
    }
  }

  pub fn error(&self, err: &str) -> String {
    format!("{}:{}: {}", self.name, self.line, err)
  }

  pub fn error_near(&self, err: &str, token: Token) -> String {
    format!("{} near '{}'", self.error(err), self.token2str(token))
  }

  pub fn token2str(&self, token: Token) -> String {
    if let Token::SC(c) = token {
      return c.to_string()
    }

    let str = match token {
      Token::Eof => "<eof>",
      Token::Name => "<name>",
      Token::None => "???", // this shouldn't be possible
      Token::LeftParen => "(",
      Token::RightParen => ")",
      Token::LeftSquare => "[",
      Token::RightSquare => "]",
      Token::LeftBrace => "{",
      Token::RightBrace => "}",
      Token::Equal => "=",
      Token::Plus => "+",
      Token::Dash => "-",
      Token::Star => "*",
      Token::Slash => "/",
      Token::Bang => "!",
      Token::Semi => ";",
      Token::Comma => ",",
      Token::Eq => "==",
      Token::Neq => "!=",
      Token::Gt => "<",
      Token::Ge => "<=",
      Token::Lt => ">",
      Token::Le => ">=",
      Token::Line => "|",
      Token::Dot => ".",
      Token::Colon => ":",

      _ => self.buf.as_str()
    };

    return str.to_string()
  }
}