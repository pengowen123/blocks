use token::Token;

use std::error::Error;
use std::fmt;

const ErrorMessages: [&'static str; 2] = [
    "Found $0 but expected one of:\n$1",
    "Unknown error"
];

#[derive(Clone, Debug)]
pub enum ErrorKind {
    UnexpectedToken,
    Other
}

#[derive(Clone, Debug)]
pub struct BlocksError {
    kind: ErrorKind,
    extra: (Token, Vec<Token>)
}

impl BlocksError {
    pub fn new(kind: ErrorKind, extra: (Token, &[Token])) -> BlocksError {
        BlocksError {
            kind: kind,
            extra: (extra.0, extra.1.to_vec())
        }
    }
}

impl fmt::Display for BlocksError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let code = self.kind.clone() as usize;
        let message = ErrorMessages[code];
        try!(write!(f, "Error (code {}):\n", code));

        match self.kind {
            ErrorKind::UnexpectedToken => {
                let message = message.replace("$0", &format!("{:?}", self.extra.0))
                                     .replace("$1", &format!("{:?}", self.extra.1));
                write!(f, "{}", message)
            },
            ErrorKind::Other => {
                write!(f, "{}", message)
            }
        }
    }
}

impl Error for BlocksError {
    fn description(&self) -> &str {
        "An error occured while compiling"
    }
}
