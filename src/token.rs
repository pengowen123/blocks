use error::BlocksError;
use error::ErrorKind::*;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Token {
    Assign, AssignSymbol,
    Symbol, Goto,
    IfGoto,
    Call, Return,
    Raw, Identifier(String),
    Tag, Dereference,
    Address, Equals,
    Multiply, Divide,
    Add, Subtract,
    Not, Compare,
    And, Or,
    Xor, Greater,
    Less, GreaterEqual,
    LessEqual, OpenBrace,
    CloseBrace, OpenParen,
    CloseParen, LineEnd,
    Null
}

pub fn build_tokens(mut prog: String) -> Vec<Token> {
    let mut tokens = vec![Token::Null];

    let mut previous_chr = '\0';
    let mut previous_word = String::new();
    let mut word = String::new();

    while !prog.is_empty() {
        let mut token = Token::Null;
        let chr = prog.chars().nth(0).unwrap();
        let mut word_end = true;
        let mut is_char = true;
        let mut pop = false;

        match chr {
            '\n' | ' ' => is_char = false,
            '=' => match tokens.last() {
                Some(t) => {
                    token = match *t {
                        Token::Greater => {
                            pop = true;
                            Token::GreaterEqual
                        },
                        Token::Less => {
                            pop = true;
                            Token::LessEqual
                        },
                        Token::AssignSymbol => {
                            pop = true;
                            Token::Equals
                        },
                        _ => Token::AssignSymbol
                    }
                }
                None => {}
            },
            ';' => token = Token::LineEnd,
            '#' => token = Token::Dereference,
            '@' => token = Token::Address,
            '*' => token = Token::Multiply,
            '/' => token = Token::Divide,
            '+' => token = Token::Add,
            '-' => token = Token::Subtract,
            '!' => token = Token::Not,
            '&' => token = Token::And,
            '|' => token = Token::Or,
            '^' => token = Token::Xor,
            '>' => token = Token::Greater,
            '<' => token = Token::Less,
            '{' => token = Token::OpenBrace,
            '}' => token = Token::CloseBrace,
            '(' => token = Token::OpenParen,
            ')' => token = Token::CloseParen,
            '?' => token = Token::Tag,
            _ => {
                word_end = false;
                is_char = false;
                word.push(chr);
            }
        }

        if pop {
            tokens.pop();
        }

        if word_end {
            let words = ["set", "cmp",
                         ">=", "<=",
                         "symbol", "goto",
                         "ifgoto", "call"];

            let symbols = ['>', '<',
                           '!', '&',
                           '|', '^',
                           '+', '-',
                           '*', '/',
                           '#', '@',
                           '=', '?'];

            if is_element_string(&previous_word, &words) {
                if !word.is_empty() && !word.chars().nth(0).unwrap().is_whitespace() {
                    tokens.push(Token::Identifier(word.clone()));
                }
            } else if is_element_string(&previous_chr, &symbols) {
                if word.len() > 0 && !word.chars().nth(0).unwrap().is_whitespace() {
                    tokens.push(Token::Identifier(word.clone()));
                }
            } else {
                if word == "==" {
                    tokens.push(Token::Equals);
                } else if word == ">=" {
                    tokens.push(Token::GreaterEqual);
                } else if word == "<=" {
                    tokens.push(Token::LessEqual);
                } else if word == "set" {
                    tokens.push(Token::Assign);
                } else if word == "cmp" {
                    tokens.push(Token::Compare);
                } else if word == "symbol" {
                    tokens.push(Token::Symbol);
                } else if word == "goto" {
                    tokens.push(Token::Goto);
                } else if word == "ifgoto" {
                    tokens.push(Token::IfGoto);
                } else if word == "call" {
                    tokens.push(Token::Call);
                } else if word == "return" {
                    tokens.push(Token::Return);
                } else if word == "raw" {
                    tokens.push(Token::Raw);
                } else {
                    if !tokens.is_empty() {
                        let last = tokens[tokens.len() - 1].clone();

                        if let Token::Identifier(..) = last {} else {
                            if !word.is_empty() && !word.chars().nth(0).unwrap().is_whitespace() {
                                tokens.push(Token::Identifier(word.clone()));
                            }        
                        }
                    }
                }
            }

            previous_word = word;
            word = String::new();

            if token != Token::Null {
                tokens.push(token);
            }
        }

        if is_char {
            previous_chr = chr;
        }

        if prog.len() > 1 {
            prog = prog[1..].to_string();
        } else {
            tokens.push(Token::Null);
            break;
        }
    }

    tokens
}

pub fn is_element_token(elem: &Token, slice: &[Token]) -> bool {
    slice.iter().fold(false, |acc, e| {
        if let (&Token::Identifier(..), &Token::Identifier(..)) = (elem, e) {
            true
        } else {
            if elem == e { true } else { acc }
        }
    })
}

fn is_element_string<A: PartialEq<B>, B: Eq>(elem: &A, slice: &[B]) -> bool {
    slice.iter().fold(false, |acc, e| {
        if elem == e { true } else { acc }
    })
}
