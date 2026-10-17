pub mod ast;
pub mod lexer;
pub mod parser;
pub mod codegen;

#[derive(Debug, Clone)]
pub struct Error { pub line: usize, pub column: usize, pub message: String }
impl Error {
    pub fn new(line: usize, column: usize, message: impl Into<String>) -> Self {
        Self { line, column, message: message.into() }
    }
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}: {}", self.line, self.column, self.message)
    }
}
impl std::error::Error for Error {}

pub fn compile(source: &str) -> Result<String, Error> {
    let tokens = lexer::lex(source)?;
    let program = parser::Parser::new(tokens).program()?;
    if !program.links.is_empty() {
        return Err(Error::new(1,1,"External links require the CLI module resolver. Browser compilation accepts a single self-contained module."));
    }
    codegen::generate(&program)
}