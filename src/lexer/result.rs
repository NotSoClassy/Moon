use crate::lexer::Result::{ Err, Ok };
use crate::lexer::Token;

pub enum Result<T, E> {
  Ok(T),
  Err(E)
}

impl Result<(), String> {
  pub(super) fn resolve(&self, token: Token) -> Result<Token, String> {
    if let Err(e) = self {
      return Err(e.to_string())
    }
    Ok(token)
  }
}

impl Result<Token, String> {
  pub(super) fn get_token(&self) -> Option<Token> {
    if let Ok(tkn) = self {
      return Some(*tkn)
    }
    None
  }
}