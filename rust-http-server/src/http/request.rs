use super::method::Method;
use std::convert::TryFrom;
use std::error::Error;
use std::fmt::{Result as FmtResult, Display, Formatter, Debug};
use std::str;

pub struct Request {
    path: String,
    query_string: Option<String>,
    method: Method,
}

// implementing the train for this type
impl TryFrom<&[u8]> for Request {
    type Error = ParseError;

    // GET /search?name=abc&sort=1 HTTP/1.1
    fn try_from(buf: &[u8]) -> Result<Self, Self::Error> {
        match str::from_utf8(buf) {
          Ok(request) => {

          },
          Err(_) => return Err(ParseError::InvalidEncoding),
        }

        // another way to do this is 
        let request = str::from_utf8(buf).or(Err(ParseError::InvalidEncoding))?;
    }
}

pub enum ParseError {
    InvalidRequest,
    // when the request is not UTF-8 encoded
    InvalidEncoding,
    // when the request has a different http version than 1.1
    InvalidProtocol,
    InvalidMethod,
}

impl ParseError {
  fn message(&self) -> &str {
    match self {
      Self::InvalidRequest => "Invalid Request", 
      Self::InvalidEncoding => "Invalid Encoding", 
      Self::InvalidMethod => "Invalid Method", 
      Self::InvalidProtocol => "Invalid Protocol", 
    }
  } 
}

impl Display for ParseError {
  fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
      // generate the string we want to output
      // and write it to the formatter
      write!(f, "{}", self.message())
  }
}

impl Debug for ParseError {
  fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
      // generate the string we want to output
      // and write it to the formatter
      write!(f, "{}", self.message())
  }
}

impl Error for ParseError {}

// to write idiomatic Rust, we have to use a trait form the standard library called Error