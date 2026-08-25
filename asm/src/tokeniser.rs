use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Ident(String),
    Int(i64),
    Colon,
    Comma,
    LBracket,
    RBracket,
    Plus,
    Minus,
    Dot,
    Newline,
    Eof,
}

impl From<Token> for &'static str {
    fn from(tok: Token) -> Self {
        match tok {
            Token::Ident(_) => "ident",
            Token::Int(_) => "int",
            Token::Colon => "colon",
            Token::Comma => "comma",
            Token::LBracket => "lbracket",
            Token::RBracket => "rbracket",
            Token::Plus => "plus",
            Token::Minus => "minus",
            Token::Dot => "dot",
            Token::Newline => "newline",
            Token::Eof => "eof",
        }
    }
}

#[derive(Debug)]
pub struct LocatedToken {
    pub token: Token,
    pub line: u64,
}

#[derive(Debug, PartialEq)]
pub enum TokeniserError {
    UnexpectedCharacter { line: u64, character: char },
}

pub fn tokenise(input: &str) -> Result<Vec<LocatedToken>, TokeniserError> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();
    let mut line = 1;

    fn push_token(
        chars: &mut Peekable<Chars>,
        tokens: &mut Vec<LocatedToken>,
        token: Token,
        line: u64,
    ) {
        tokens.push(LocatedToken { token, line });
        chars.next();
    }

    while let Some(c) = chars.peek() {
        match c {
            ':' => push_token(&mut chars, &mut tokens, Token::Colon, line),
            ',' => push_token(&mut chars, &mut tokens, Token::Comma, line),
            '[' => push_token(&mut chars, &mut tokens, Token::LBracket, line),
            ']' => push_token(&mut chars, &mut tokens, Token::RBracket, line),
            '+' => push_token(&mut chars, &mut tokens, Token::Plus, line),
            '-' => push_token(&mut chars, &mut tokens, Token::Minus, line),
            '.' => push_token(&mut chars, &mut tokens, Token::Dot, line),
            ';' => {
                take_while(&mut chars, |c| c != '\n');
            }
            '\n' => {
                push_token(&mut chars, &mut tokens, Token::Newline, line);
                line += 1;
            }
            _ if c.is_whitespace() => {
                chars.next();
            }
            _ if c.is_numeric() => tokens.push(LocatedToken {
                token: read_int(&mut chars),
                line,
            }),
            _ if c.is_alphabetic() => tokens.push(LocatedToken {
                token: read_ident(&mut chars),
                line,
            }),
            _ => {
                return Err(TokeniserError::UnexpectedCharacter {
                    line,
                    character: *c,
                });
            }
        }
    }

    tokens.push(LocatedToken {
        token: Token::Eof,
        line,
    });
    Ok(tokens)
}

fn take_while(chars: &mut Peekable<Chars>, pred: impl Fn(char) -> bool) -> String {
    let mut s = String::new();
    while let Some(&c) = chars.peek() {
        if pred(c) {
            s.push(c);
            chars.next();
        } else {
            break;
        }
    }
    s
}

fn read_ident(chars: &mut Peekable<Chars>) -> Token {
    Token::Ident(take_while(chars, |c| char::is_alphanumeric(c) || c == '_'))
}

fn read_int(chars: &mut Peekable<Chars>) -> Token {
    if let Some(&'0') = chars.peek() {
        chars.next();

        if let Some(&'x') = chars.peek() {
            chars.next();
            read_hex(chars)
        } else {
            read_decimal(chars)
        }
    } else {
        read_decimal(chars)
    }
}

fn read_hex(chars: &mut Peekable<Chars>) -> Token {
    let mut num: i64 = 0;
    while let Some(&c) = chars.peek() {
        if let Some(digit) = c.to_digit(16) {
            num = num * 16 + digit as i64;
            chars.next();
        } else {
            break;
        }
    }
    Token::Int(num)
}

fn read_decimal(chars: &mut Peekable<Chars>) -> Token {
    let mut num: i64 = 0;
    while let Some(c) = chars.peek() {
        if let Some(digit) = c.to_digit(10) {
            num = num * 10 + digit as i64;
            chars.next();
        } else {
            break;
        }
    }
    Token::Int(num)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unwrap_token(located_token: LocatedToken) -> Token {
        located_token.token
    }

    #[test]
    fn tokenise_ident() {
        assert_eq!(
            tokenise("add")
                .unwrap()
                .into_iter()
                .map(unwrap_token)
                .collect::<Vec<_>>(),
            vec![Token::Ident("add".into()), Token::Eof]
        );
    }

    #[test]
    fn tokenise_decimal() {
        assert_eq!(
            tokenise("42")
                .unwrap()
                .into_iter()
                .map(unwrap_token)
                .collect::<Vec<_>>(),
            vec![Token::Int(42), Token::Eof]
        );
    }

    #[test]
    fn tokenise_hex() {
        assert_eq!(
            tokenise("0xFF")
                .unwrap()
                .into_iter()
                .map(unwrap_token)
                .collect::<Vec<_>>(),
            vec![Token::Int(255), Token::Eof]
        );
    }

    #[test]
    fn tokenise_ident_with_leading_whitespace() {
        assert_eq!(
            tokenise("  r0")
                .unwrap()
                .into_iter()
                .map(unwrap_token)
                .collect::<Vec<_>>(),
            vec![Token::Ident("r0".into()), Token::Eof]
        );
    }

    #[test]
    fn tokenise_symbols() {
        assert_eq!(
            tokenise(": , [ ] + - .")
                .unwrap()
                .into_iter()
                .map(unwrap_token)
                .collect::<Vec<_>>(),
            vec![
                Token::Colon,
                Token::Comma,
                Token::LBracket,
                Token::RBracket,
                Token::Plus,
                Token::Minus,
                Token::Dot,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn tokenise_newline() {
        assert_eq!(
            tokenise("a\nb")
                .unwrap()
                .into_iter()
                .map(unwrap_token)
                .collect::<Vec<_>>(),
            vec![
                Token::Ident("a".into()),
                Token::Newline,
                Token::Ident("b".into()),
                Token::Eof
            ]
        );
    }

    #[test]
    fn tokenise_label_definition() {
        assert_eq!(
            tokenise("loop:")
                .unwrap()
                .into_iter()
                .map(unwrap_token)
                .collect::<Vec<_>>(),
            vec![Token::Ident("loop".into()), Token::Colon, Token::Eof]
        );
    }

    #[test]
    fn tokenise_program() {
        assert_eq!(
            tokenise(
                "; comment
                 label:
                 hlt
                 nop
                 bal lr, subroutine
                 ldm32 r0, [r1 + 4]"
            )
            .unwrap()
            .into_iter()
            .map(unwrap_token)
            .collect::<Vec<_>>(),
            vec![
                Token::Newline,
                Token::Ident("label".into()),
                Token::Colon,
                Token::Newline,
                Token::Ident("hlt".into()),
                Token::Newline,
                Token::Ident("nop".into()),
                Token::Newline,
                Token::Ident("bal".into()),
                Token::Ident("lr".into()),
                Token::Comma,
                Token::Ident("subroutine".into()),
                Token::Newline,
                Token::Ident("ldm32".into()),
                Token::Ident("r0".into()),
                Token::Comma,
                Token::LBracket,
                Token::Ident("r1".into()),
                Token::Plus,
                Token::Int(4),
                Token::RBracket,
                Token::Eof
            ]
        )
    }
}
