#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ty {
    Int, Float, Bool, Char, String, Void, File,
    Vector(Box<Ty>), Object(String), Generic(String),
}
impl Ty {
    pub fn from_name(s: &str) -> Self {
        match s {
            "int" => Self::Int, "float" => Self::Float, "bool" => Self::Bool,
            "char" => Self::Char, "string" => Self::String, "void" => Self::Void,
            "file" => Self::File, _ => Self::Object(s.to_owned()),
        }
    }
    pub fn llvm(&self) -> &'static str {
        match self {
            Self::Int | Self::Char => "i64", Self::Float => "double",
            Self::Bool => "i1", Self::Void => "void", _ => "ptr",
        }
    }
}
#[derive(Clone, Debug)]
pub enum Expr {
    Int(i64), Float(f64), Bool(bool), String(String), Var(String),
    Index(Box<Expr>, Box<Expr>), Member(Box<Expr>, String),
    Unary(String, Box<Expr>), Binary(String, Box<Expr>, Box<Expr>),
    PostInc(Box<Expr>, i64),
}
#[derive(Clone, Debug)]
pub struct Decl {
    pub ty: Ty, pub name: String, pub init: Option<Expr>,
    pub size: Option<Expr>, pub values: Vec<Expr>,
    pub file: Option<(String, String, String)>,
}
#[derive(Clone, Debug)]
pub enum Stmt {
    Decl(Vec<Decl>),
    Flow(Vec<Vec<Expr>>),
    When(Expr, Vec<LocatedStmt>, Vec<LocatedStmt>),
    Repeat(Expr, Vec<LocatedStmt>),
    Iterate(Decl, String, Expr, i64, Vec<LocatedStmt>),
    Return(Option<Expr>), Break, Continue, Action(Expr),
}
#[derive(Clone, Debug)]
pub struct LocatedStmt { pub line: usize, pub stmt: Stmt }
#[derive(Clone, Debug)]
pub struct Function {
    pub name: String, pub params: Vec<(Ty, String)>, pub ret: Ty,
    pub body: Vec<LocatedStmt>, pub line: usize, pub generic_op: Option<String>,
}
#[derive(Clone, Debug)]
pub struct Class { pub name: String, pub fields: Vec<Decl>, pub methods: Vec<Function> }
#[derive(Clone, Debug, Default)]
pub struct Program { pub functions: Vec<Function>, pub classes: Vec<Class>, pub links: Vec<String> }