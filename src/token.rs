// First stage in compilation.
// Converts a given program taken as a string into a a tokenized form, for easier compilation.
// This stage does not detect any errors, but may produce invalid sets of tokens from invalid input.

use utils::*;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Token {
    Assign, AssignSymbol,
    Symbol, Goto,
    IfGoto,
    Call, Return,
    Identifier(String), Tag,
    Dereference, Address,
    Equals, Multiply,
    Divide, Add,
    Subtract, Not,
    Compare, And,
    Or, Xor,
    Greater, Less,
    GreaterEqual, LessEqual,
    OpenBrace, CloseBrace,
    LineEnd, Raw,
    Number(i32), Register(Register),
    Other(String), Null
}

pub fn build_tokens(mut prog: String) -> Vec<Token> {
    let mut tokens = vec![Token::Null];

    let mut previous_chr = '\0';
    let mut previous_chr2 = '\0';
    let mut previous_word = String::new();
    let mut word = String::new();
    let mut raw = false;
    let mut comment = false;
    let mut previous_whitespace = false;

    while !prog.is_empty() {
        let mut token = Token::Null;
        let chr = prog.chars().nth(0).unwrap();
        let mut word_end = true;
        let mut is_char = true;
        let mut pop = false;
        let mut special = false;

        if raw {
            word_end = false;
            is_char = false;
            special = true;

            if chr != '`' {
                word.push(chr);
            }
        }

        if comment {
            if chr == '\n' {
                comment = false;
            } else {
                special = true;
            }
        } else {
            if chr == '`' {
                raw = !raw;
                special = true;
            }
        }

        if special {
            if prog.len() > 1 {
                prog = prog[1..].to_string();
                continue;
            } else {
                tokens.push(Token::Null);
                break;
            }
        }

        match chr {
            '\n' | ' ' => is_char = false,
            '=' => token = if let Some(t) = tokens.last() {
                if !previous_whitespace {
                    match *t {
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
                } else {
                    Token::AssignSymbol
                }
            } else { Token::AssignSymbol },
            ';' => token = Token::LineEnd,
            '#' => token = Token::Dereference,
            '@' => token = Token::Address,
            '*' => token = Token::Multiply,
            '/' => token = {
                if let Some(&Token::Divide) = tokens.last() {
                    if !previous_whitespace {
                        pop = true;
                        comment = true;
                        Token::Null
                    } else {
                        Token::Divide
                    }
                } else {
                    Token::Divide
                }
            },
            '+' => token = Token::Add,
            '~' => token = Token::Subtract,
            '!' => token = Token::Not,
            '&' => token = Token::And,
            '|' => token = Token::Or,
            '^' => token = Token::Xor,
            '>' => token = Token::Greater,
            '<' => token = Token::Less,
            '{' => token = Token::OpenBrace,
            '}' => token = Token::CloseBrace,
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
                         "ifgoto", "call",
                         "raw"];

            let symbols = ['>', '<',
                           '!', '&',
                           '|', '^',
                           '+', '~',
                           '*', '/',
                           '#', '@',
                           '=', '?'];

            if is_element(&previous_word, &words) {
                if !word.is_empty() {
                    tokens.push(Token::Identifier(word.clone()));
                }
            } else if is_element(&previous_chr, &symbols) &&
                      previous_chr2 != '?' &&
                      (previous_chr, previous_chr2) != ('/', '/') {
                // this magically lets you leave out semicolons in tags
                previous_chr2 = previous_chr;

                if word.len() > 0 {
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
                            if !word.is_empty() {
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

        previous_whitespace = chr.is_whitespace();

        if prog.len() > 1 {
            prog = prog[1..].to_string();
        } else {
            tokens.push(Token::Null);
            break;
        }
    }

    tokens.iter().map(|t| if let &Token::Identifier(ref ident) = t {
        if let Ok(v) = ident.parse::<i32>() {
            Token::Number(v)
        } else {
            match &ident as &str {
                "$int1" => Token::Register(Register::Int1),
                "$int2" => Token::Register(Register::Int2),
                "$accum" => Token::Register(Register::Accum),
                "$flag" => Token::Register(Register::Flag),
                "$error" => Token::Register(Register::Error),
                "$segment" => Token::Register(Register::Segment),
                "$pcounter" => Token::Register(Register::PCounter),
                _ => t.clone()
            }
        }} else {
            t.clone()
        }
    ).collect()
}
