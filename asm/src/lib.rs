mod codegen;
mod parser;
mod tokeniser;
pub mod preprocessor;

use codegen::CodegenError;
use parser::ParseError;
use tokeniser::TokeniserError;
use preprocessor::PreprocessorError;

#[derive(Debug)]
pub enum AssembleError {
    Tokenise(TokeniserError),
    Parse(ParseError),
    Codegen(CodegenError),
    Preprocessor(PreprocessorError)
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

impl From<PreprocessorError> for AssembleError {
    fn from(e: PreprocessorError) -> Self {
        AssembleError::Preprocessor(e)
    }
}

pub fn assemble(src: &str) -> Result<Vec<u8>, AssembleError> {
    let tokens = tokeniser::tokenise(src)?;
    let items = parser::parse(tokens)?;
    Ok(codegen::codegen(&items)?)
}
