use std::iter::Peekable;
use std::vec::IntoIter;

use bliss::register::Register;

use crate::tokeniser::{LocatedToken, Token};

#[derive(Debug, PartialEq)]
pub enum Operand {
    Reg(Register),
    Imm(i64),
    MemRef { base: Register, offset: i32 },
    Label(String),
}

#[derive(Debug, PartialEq)]
pub enum Item {
    Label(String),
    Instruction {
        mnemonic: String,
        operands: Vec<Operand>,
    },
    Directive {
        name: String,
        values: Vec<i64>,
    },
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    UnexpectedToken {
        line: u64,
        got: Token,
        expected: Option<&'static str>,
    },
    UnexpectedEof,
    UnknownRegister {
        line: u64,
        name: String,
    },
}

struct Parser {
    tokens: Peekable<IntoIter<LocatedToken>>,
}

impl Parser {
    fn peek(&mut self) -> Option<Token> {
        self.tokens.peek().map(|lt| lt.token.clone())
    }

    fn advance(&mut self) -> LocatedToken {
        self.tokens.next().expect("advance on exhausted stream")
    }

    fn expect(&mut self, expected: Token) -> Result<LocatedToken, ParseError> {
        match self.tokens.next() {
            Some(lt) => {
                if expected == lt.token {
                    Ok(lt)
                } else {
                    Err(ParseError::UnexpectedToken {
                        line: lt.line,
                        got: lt.token,
                        expected: Some(expected.into()),
                    })
                }
            }
            None => Err(ParseError::UnexpectedEof),
        }
    }

    fn expect_ident(&mut self) -> Result<String, ParseError> {
        let LocatedToken { token, line } = self.advance();
        match token {
            Token::Ident(s) => Ok(s),
            other => Err(ParseError::UnexpectedToken {
                line,
                got: other,
                expected: Some("identifier"),
            }),
        }
    }

    fn expect_int(&mut self) -> Result<i64, ParseError> {
        let LocatedToken { token, line } = self.advance();
        match token {
            Token::Int(i) => Ok(i),
            Token::Minus => {
                let LocatedToken { token, line } = self.advance();
                match token {
                    Token::Int(i) => Ok(-i),
                    other => Err(ParseError::UnexpectedToken {
                        line,
                        got: other,
                        expected: Some("integer"),
                    }),
                }
            }
            other => Err(ParseError::UnexpectedToken {
                line,
                got: other,
                expected: Some("integer"),
            }),
        }
    }

    fn parse_label(&mut self, name: String) -> Result<Item, ParseError> {
        self.advance();
        Ok(Item::Label(name))
    }

    fn parse_register(r: &str) -> Option<Register> {
        match r {
            "r0" => Some(Register::R0),
            "r1" => Some(Register::R1),
            "r2" => Some(Register::R2),
            "r3" => Some(Register::R3),
            "r4" => Some(Register::R4),
            "r5" => Some(Register::R5),
            "r6" => Some(Register::R6),
            "r7" => Some(Register::R7),
            "r8" => Some(Register::R8),
            "r9" => Some(Register::R9),
            "r10" => Some(Register::R10),
            "r11" => Some(Register::R11),
            "r12" => Some(Register::R12),
            "r13" | "sp" => Some(Register::SP),
            "r14" | "lr" => Some(Register::LR),
            "r15" | "pc" => Some(Register::PC),
            _ => None,
        }
    }

    fn parse_memref(&mut self) -> Result<Operand, ParseError> {
        let LocatedToken { token, line } = self.advance();
        let reg = match token {
            Token::Ident(s) => {
                Parser::parse_register(&s).ok_or(ParseError::UnknownRegister { line, name: s })
            }
            other => Err(ParseError::UnexpectedToken {
                line,
                got: other,
                expected: Some("register"),
            }),
        }?;

        let offset: i32 = match self.peek() {
            Some(Token::RBracket) => Ok(0),
            None => Err(ParseError::UnexpectedEof),
            Some(Token::Plus) => { self.advance(); Ok(self.expect_int()? as i32) }
            Some(Token::Eof) => Err(ParseError::UnexpectedEof),
            _ => Ok(self.expect_int()? as i32),
        }?;
        self.expect(Token::RBracket)?;

        Ok(Operand::MemRef { base: reg, offset })
    }

    fn parse_operand(&mut self) -> Result<Operand, ParseError> {
        match self.peek() {
            Some(Token::Int(_)) | Some(Token::Minus) => Ok(Operand::Imm(self.expect_int()?)),
            Some(Token::Ident(_)) => {
                let s = self.expect_ident()?;
                Ok(Parser::parse_register(&s).map_or(Operand::Label(s), Operand::Reg))
            }
            Some(Token::LBracket) => {
                self.advance();
                self.parse_memref()
            }
            None => Err(ParseError::UnexpectedEof),
            _ => {
                let LocatedToken { token, line } = self.advance();
                Err(ParseError::UnexpectedToken { line, got: token, expected: None })
            }
        }
    }

    fn parse_instruction(&mut self, mnemonic: String) -> Result<Item, ParseError> {
        let mut operands = Vec::new();
        loop {
            match self.peek() {
                None | Some(Token::Newline) | Some(Token::Eof) => break,
                Some(Token::Comma) => {
                    self.advance();
                }
                _ => {
                    operands.push(self.parse_operand()?);
                }
            }
        }
        Ok(Item::Instruction { mnemonic, operands })
    }

    fn parse_label_or_instruction(&mut self) -> Result<Item, ParseError> {
        let ident = self.expect_ident()?;
        match self.peek() {
            Some(Token::Colon) => self.parse_label(ident),
            _ => self.parse_instruction(ident),
        }
    }

    fn parse_directive(&mut self) -> Result<Item, ParseError> {
        self.advance(); // consume Dot
        let name = self.expect_ident()?;

        let mut values = Vec::new();
        loop {
            match self.peek() {
                None | Some(Token::Newline) | Some(Token::Eof) => break,
                Some(Token::Comma) => { self.advance(); }
                _ => values.push(self.expect_int()?),
            }
        }

        Ok(Item::Directive { name, values })
    }

    fn parse(&mut self) -> Result<Vec<Item>, ParseError> {
        let mut items = Vec::new();
        loop {
            match self.peek() {
                Some(Token::Newline) => {
                    self.advance();
                }
                Some(Token::Eof) | None => break,
                Some(Token::Ident(_)) => items.push(self.parse_label_or_instruction()?),
                Some(Token::Dot) => items.push(self.parse_directive()?),
                _ => {
                    let LocatedToken { line, token } = self.advance();
                    return Err(ParseError::UnexpectedToken {
                        line,
                        got: token,
                        expected: None,
                    });
                }
            }
        }
        Ok(items)
    }
}

pub fn parse(tokens: Vec<LocatedToken>) -> Result<Vec<Item>, ParseError> {
    let mut parser = Parser {
        tokens: tokens.into_iter().peekable(),
    };
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;
    use bliss::register::Register;

    fn tok(token: Token) -> LocatedToken {
        LocatedToken { token, line: 1 }
    }

    #[test]
    fn parse_label() {
        let tokens = vec![tok(Token::Ident("loop".into())), tok(Token::Colon), tok(Token::Eof)];
        assert_eq!(parse(tokens).unwrap(), vec![Item::Label("loop".into())]);
    }

    #[test]
    fn parse_instruction_with_registers() {
        let tokens = vec![
            tok(Token::Ident("add".into())),
            tok(Token::Ident("r0".into())),
            tok(Token::Comma),
            tok(Token::Ident("r1".into())),
            tok(Token::Comma),
            tok(Token::Ident("r2".into())),
            tok(Token::Eof),
        ];
        assert_eq!(parse(tokens).unwrap(), vec![Item::Instruction {
            mnemonic: "add".into(),
            operands: vec![Operand::Reg(Register::R0), Operand::Reg(Register::R1), Operand::Reg(Register::R2)],
        }]);
    }

    #[test]
    fn parse_directive_with_values() {
        let tokens = vec![
            tok(Token::Dot),
            tok(Token::Ident("word".into())),
            tok(Token::Int(1)),
            tok(Token::Comma),
            tok(Token::Minus),
            tok(Token::Int(2)),
            tok(Token::Eof),
        ];
        assert_eq!(parse(tokens).unwrap(), vec![Item::Directive {
            name: "word".into(),
            values: vec![1, -2],
        }]);
    }

    #[test]
    fn parse_memref_with_offset() {
        let tokens = vec![
            tok(Token::Ident("ldm32".into())),
            tok(Token::Ident("r0".into())),
            tok(Token::Comma),
            tok(Token::LBracket),
            tok(Token::Ident("r1".into())),
            tok(Token::Minus),
            tok(Token::Int(4)),
            tok(Token::RBracket),
            tok(Token::Eof),
        ];
        assert_eq!(parse(tokens).unwrap(), vec![Item::Instruction {
            mnemonic: "ldm32".into(),
            operands: vec![
                Operand::Reg(Register::R0),
                Operand::MemRef { base: Register::R1, offset: -4 },
            ],
        }]);
    }
}
