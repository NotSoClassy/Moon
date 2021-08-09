use crate::lexer::Result::{ Ok, Err };
use crate::lexer::Result;
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
      self.token = res.get_token().unwrap();
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
      '/' => next_ret!(Token::Slash),
      ';' => next_ret!(Token::Semi),
      ',' => next_ret!(Token::Comma),

      '=' => cmp_op!(Token::Equal, Token::Eq),
      '<' => cmp_op!(Token::Gt, Token::Ge),
      '>' => cmp_op!(Token::Lt, Token::Le),
      '!' => cmp_op!(Token::Bang, Token::Neq),

      '\0' => Ok(Token::Eof),

      '\'' | '"' => self.read_string(self.current).resolve(Token::String),


      '\n' | '\r' => {
        self.line += 1;
        self.next();
        self.lex()
      }

      ' ' | '\t' => {
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
          self.read_number().resolve(Token::Number)
        } else {
          Err(self.error("unexpected token", Token::SC(self.current)))
        }
      }
    }
  }

  fn read_name(&mut self) {
    self.buf.push(self.current);
    self.next();

    while self.is_ident() {
      self.buf.push(self.current);
      self.next();
    }
  }

  fn read_string(&mut self, quote: char) -> Result<(), String> {
    self.buf.push(self.current);
    self.next();

    while self.current != quote {
      match self.current {
        '\0' => {
          return Err(self.error("unfinished string", Token::Eof))
        }

        '\n' => {
          return Err(self.error("unfinished string", Token::String))
        }

        '\\' => {
          self.next();

          let c = match self.current {
            '\n' => {
              self.line += 1;
              self.next();
              ""
            }

            't' => "\t",
            'n' => "\n",
            '\'' => "'",
            '"' => "\"",
            '\\' => "\\",
            '\0' => continue,

            _ => {
              return Err(self.error("invalid escape", Token::String))
            }
          };

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
      Err(self.error("malformed number", self.token))
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

  #[inline(always)]
  fn is_alpha(&self) -> bool {
    matches!(self.current, 'a' ..= 'z' | 'A' ..= 'Z')
  }

  #[inline(always)]
  fn is_num(&self) -> bool {
    matches!(self.current, '0' ..= '9')
  }

  #[inline(always)]
  fn is_alnum(&self) -> bool {
    self.is_num() || self.is_alpha()
  }

  #[inline(always)]
  fn is_ident(&self) -> bool {
    self.is_alpha() || self.current == '_'
  }

  fn is_keyword(&self) -> Option<Token> {
    match self.buf.as_str() {
      "let" => Some(Token::Let),
      "if" => Some(Token::If),
      "else" => Some(Token::Else),
      "fn" => Some(Token::Fn),
      "while" => Some(Token::While),
      "true" => Some(Token::True),
      "false" => Some(Token::False),
      "nil" => Some(Token::Nil),

      _ => None
    }
  }

  pub fn error(&self, err: &str, token: Token) -> String {
    format!("{}:{}: {} near '{}'", self.name, self.line, err, self.token2str(token))
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

      _ => self.buf.as_str()
    };

    return str.to_string()
  }
}