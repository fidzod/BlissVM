pub mod codegen;
pub mod parser;
pub mod tokeniser;

use codegen::CodegenError;
use parser::ParseError;
use tokeniser::TokeniserError;

#[derive(Debug)]
pub enum AssembleError {
    Tokenise(TokeniserError),
    Parse(ParseError),
    Codegen(CodegenError),
}

impl From<TokeniserError> for AssembleError {
    fn from(e: TokeniserError) -> Self {
        AssembleError::Tokenise(e)
    }
}

impl From<ParseError> for AssembleError {
    fn from(e: ParseError) -> Self {
        AssembleError::Parse(e)
    }
}

impl From<CodegenError> for AssembleError {
    fn from(e: CodegenError) -> Self {
        AssembleError::Codegen(e)
    }
}

pub fn assemble(src: &str) -> Result<Vec<u8>, AssembleError> {
    let tokens = tokeniser::tokenise(src)?;
    let items = parser::parse(tokens)?;
    Ok(codegen::codegen(&items)?)
}
