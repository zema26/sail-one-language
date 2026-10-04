use crate::Error;

#[derive(Clone, Debug, PartialEq)]
pub enum Kind { Word(String), Number(String), Str(String), Sym(String), Newline, Eof }
#[derive(Clone, Debug)]
pub struct Token { pub kind: Kind, pub line: usize, pub col: usize }
pub fn lex(source: &str) -> Result<Vec<Token>, Error> {
    let chars: Vec<char> = source.chars().collect();
    let (mut i, mut line, mut col) = (0, 1, 1);
    let mut out = Vec::new();
    while i < chars.len() {
        let c = chars[i];
        if c == '\r' || c == ' ' || c == '\t' { i += 1; col += 1; continue; }
        if c == '/' && chars.get(i + 1) == Some(&'/') {
            while i < chars.len() && chars[i] != '\n' { i += 1; col += 1; }
            continue;
        }
        let start = col;
        let kind = if c == '\n' || c == ';' {
            i += 1;
            if c == '\n' { line += 1; col = 1; } else { col += 1; }
            Kind::Newline
        } else if c == '"' || c == '\'' {
            let quote = c; i += 1; col += 1;
            let mut s = String::new();
            while i < chars.len() && chars[i] != quote {
                if chars[i] == '\n' { return Err(Error::new(line, col, "Unterminated string literal")); }
                if chars[i] == '\\' {
                    i += 1; col += 1;
                    let e = *chars.get(i).ok_or_else(|| Error::new(line, col, "Unterminated escape"))?;
                    s.push(match e { 'n' => '\n', 't' => '\t', 'r' => '\r', '0' => '\0', '\\' => '\\', '"' => '"', '\'' => '\'', _ => return Err(Error::new(line,col,"Unknown string escape")) });
                } else { s.push(chars[i]); }
                i += 1; col += 1;
            }
            if i == chars.len() { return Err(Error::new(line,start,"Unterminated string literal")); }
            i += 1; col += 1; Kind::Str(s)
        } else if c.is_ascii_digit() {
            let from = i;
            while i < chars.len() && chars[i].is_ascii_digit() { i += 1; col += 1; }
            if chars.get(i) == Some(&'.') && chars.get(i+1).is_some_and(char::is_ascii_digit) {
                i += 1; col += 1;
                while i < chars.len() && chars[i].is_ascii_digit() { i += 1; col += 1; }
            }
            Kind::Number(chars[from..i].iter().collect())
        } else if c.is_ascii_alphabetic() || c == '_' {
            let from = i;
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') { i += 1; col += 1; }
            Kind::Word(chars[from..i].iter().collect())
        } else {
            let pair: String = chars[i..usize::min(i+2,chars.len())].iter().collect();
            if ["==","!=","<=",">=","++","--","&&","||"].contains(&pair.as_str()) {
                // || is also an empty vector; the parser handles both roles.
                i += 2; col += 2; Kind::Sym(pair)
            } else if "|><()+-*/%,.!&".contains(c) {
                i += 1; col += 1; Kind::Sym(c.to_string())
            } else { return Err(Error::new(line,col,format!("Unexpected character '{c}'"))); }
        };
        let token_line = if c == '\n' { line - 1 } else { line };
        out.push(Token { kind, line: token_line, col: start });
    }
    out.push(Token { kind: Kind::Eof, line, col });
    Ok(out)
}