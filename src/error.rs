use token::Token;

use std::error::Error;
use std::fmt;

const ERROR_MESSAGES: &'static [&'static str] = &[
    "Unexpected token: $0",
    "Not enough arguments to keyword or operator: $0",
    "Symbol name must be a string",
    "Tag name must be a string",
    "Tag value must be a string or number",
    "Use of undeclared variable: $0",
    "Invalid inline machine code: $0",
    "Unmatched token: $0",
    "Argument to address operator must be an identifier",
    "Invalid write address: $0",
    "Call address must be an identifier",
    "IfGoto address must be an identifier",
    "Unknown tag: $0",
    "Tag error: $0",
    "Unknown error at token: $0"
];

#[derive(Clone, Debug)]
pub enum ErrorKind {
    UnexpectedToken,
    NotEnoughArgs,
    SymbolNameType,
    TagNameType,
    TagValueType,
    UndeclaredVar,
    InvalidRaw,
    UnmatchedToken,
    AddressNameType,
    InvalidAddress,
    CallAddressType,
    IfGotoAddressType,
    UnknownTag,
    TagError,
    Other
}

#[derive(Clone, Debug)]
pub struct BlocksError {
    kind: ErrorKind,
    extra: Token
}

impl BlocksError {
    pub fn new(kind: ErrorKind, extra: Token) -> BlocksError {
        BlocksError {
            kind: kind,
            extra: extra
        }
    }
}

impl fmt::Display for BlocksError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let code = self.kind.clone() as usize;
        let mut message = ERROR_MESSAGES[code].to_string();
        write!(f, "Error (code {}):\n", code)?;

        if let Token::Other(ref s) = self.extra {
            message = message.replace("$0", &s);
        } else {
            message = message.replace("$0", &format!("{:?}", self.extra));
        }
        write!(f, "{}", message)
    }
}

impl Error for BlocksError {
    fn description(&self) -> &str {
        "An error occured while compiling"
    }
}
