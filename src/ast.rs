use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span { pub start: usize, pub end: usize }

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Void, Bool, Char, I32, I64, F32, F64, String,
    Pointer(Box<Type>), Array(Box<Type>, Option<usize>),
    Class(String), Infer,
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Type::Void => write!(f, "void"), Type::Bool => write!(f, "bool"),
            Type::Char => write!(f, "char"), Type::I32 => write!(f, "int"), Type::I64 => write!(f, "i64"),
            Type::F32 => write!(f, "f32"), Type::F64 => write!(f, "f64"), Type::String => write!(f, "string"),
            Type::Pointer(t) => write!(f, "*{}", t),
            Type::Array(t, Some(s)) => write!(f, "{}[{}]", t, s),
            Type::Array(t, None) => write!(f, "{}[]", t),
            Type::Class(n) => write!(f, "{}", n),
            Type::Infer => write!(f, "auto"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(i64), Float(f64), String(String), Char(char), Bool(bool), Null,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add, Sub, Mul, Div, Mod, And, Or, BitAnd, BitOr, Xor, Shl, Shr,
    Eq, Neq, Lt, Gt, Le, Ge, Assign,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp { Neg, Not, Deref, AddrOf }

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub ty: Type,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct FunctionDecl {
    pub name: String,
    pub params: Vec<Parameter>,
    pub return_type: Type,
    pub body: Option<Vec<Statement>>,
    pub is_extern: bool,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ClassDecl {
    pub name: String,
    pub bases: Vec<String>,
    pub fields: Vec<FieldDecl>,
    pub methods: Vec<FunctionDecl>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct FieldDecl {
    pub name: String,
    pub ty: Type,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Variable(String, Type, Option<Expression>, Span),
    Expression(Expression, Span),
    Block(Vec<Statement>, Span),
    If(Expression, Box<Statement>, Option<Box<Statement>>, Span),
    While(Expression, Box<Statement>, Span),
    For(Box<Statement>, Option<Expression>, Option<Expression>, Box<Statement>, Span),
    ForEach(String, Expression, Box<Statement>, Span),
    Return(Option<Expression>, Span),
    Break(Span), Continue(Span),
    Switch(Expression, Vec<(Expression, Vec<Statement>)>, Option<Box<Statement>>, Span),
    Try(Vec<Statement>, Option<String>, Vec<Statement>, Option<Vec<Statement>>, Span),
    Throw(Expression, Span),
}

#[derive(Debug, Clone)]
pub enum Expression {
    Literal(Literal, Span),
    Identifier(String, Span),
    Binary(Box<Expression>, BinOp, Box<Expression>, Span),
    Unary(UnaryOp, Box<Expression>, Span),
    Ternary(Box<Expression>, Box<Expression>, Box<Expression>, Span),
    Call(String, Vec<Expression>, Span),
    Index(Box<Expression>, Box<Expression>, Span),
    MemberAccess(Box<Expression>, String, Span),
    New(Type, Vec<Expression>, Span),
    NewArray(Type, Vec<Expression>, Span),
    Cast(Type, Box<Expression>, Span),
    SizeOf(Type, Span),
    ArrayLiteral(Vec<Expression>, Span),
    ObjectLiteral(Vec<(String, Expression)>, Span),
    Range(Option<Box<Expression>>, Option<Box<Expression>>, Span),
}

#[derive(Debug, Clone)]
pub struct Module {
    pub name: String,
    pub imports: Vec<String>,
    pub declarations: Vec<Declaration>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Declaration {
    Function(FunctionDecl),
    Class(ClassDecl),
    Variable(String, Type, Option<Expression>),
}

pub fn str_to_type(s: &str) -> Type {
    match s {
        "void" => Type::Void, "bool" => Type::Bool, "char" => Type::Char,
        "int" | "i32" => Type::I32, "i64" | "long" => Type::I64,
        "float" | "f32" => Type::F32, "f64" | "double" => Type::F64,
        "string" | "String" => Type::String, "auto" | "var" => Type::Infer,
        _ => Type::Class(s.to_string()),
    }
}
