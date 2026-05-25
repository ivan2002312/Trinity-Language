use logos::Logos;
use crate::ast::Span;
use crate::error::CompilerError;

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"[ \t\r\n\f]+")]
#[logos(skip r"//[^\n\r]*")]
#[logos(skip r"/\*([^*]|\*[^/])*\*/")]
pub enum Token {
    #[token("module")] Module, #[token("import")] Import, #[token("class")] Class,
    #[token("struct")] Struct, #[token("interface")] Interface, #[token("enum")] Enum,
    #[token("template")] Template, #[token("where")] Where, #[token("using")] Using,
    #[token("namespace")] Namespace,

    #[token("public")] Public, #[token("private")] Private, #[token("protected")] Protected,
    #[token("static")] Static, #[token("virtual")] Virtual, #[token("override")] Override,
    #[token("abstract")] Abstract, #[token("const")] Const, #[token("extern")] Extern,
    #[token("unsafe")] Unsafe,

    #[token("var")] Var, #[token("let")] Let, #[token("auto")] Auto,
    #[token("return")] Return, #[token("if")] If, #[token("else")] Else,
    #[token("switch")] Switch, #[token("case")] Case, #[token("default")] Default,
    #[token("while")] While, #[token("do")] Do, #[token("for")] For,
    #[token("foreach")] ForEach, #[token("in")] In, #[token("break")] Break,
    #[token("continue")] Continue, #[token("goto")] Goto,
    #[token("try")] Try, #[token("catch")] Catch, #[token("finally")] Finally,
    #[token("throw")] Throw, #[token("new")] New, #[token("delete")] Delete,
    #[token("sizeof")] SizeOf, #[token("typeof")] TypeOf,
    #[token("as")] As, #[token("is")] Is, #[token("ref")] Ref,
    #[token("operator")] Operator,

    #[token("true")] True, #[token("false")] False, #[token("null")] Null,

    #[regex(r"0x[0-9a-fA-F]+", |lex| i64::from_str_radix(&lex.slice()[2..], 16).unwrap_or(0))]
    #[regex(r"0b[01]+", |lex| i64::from_str_radix(&lex.slice()[2..], 2).unwrap_or(0))]
    #[regex(r"[0-9]+", |lex| lex.slice().parse().unwrap_or(0))]
    IntLit(i64),

    #[regex(r"[0-9]+\.[0-9]*", |lex| lex.slice().parse().unwrap_or(0.0))]
    FloatLit(f64),

    #[regex(r#""[^"]*""#, |lex| lex.slice()[1..lex.slice().len()-1].to_string())]
    StringLit(String),

    #[regex(r"'[^']*'", |lex| lex.slice()[1..lex.slice().len()-1].chars().next().unwrap_or('\0'))]
    CharLit(char),

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),

    #[token("==")] EqEq, #[token("!=")] NotEq, #[token("<=")] LtEq, #[token(">=")] GtEq,
    #[token("&&")] AndAnd, #[token("||")] OrOr,
    #[token("<<")] Shl, #[token(">>")] Shr,
    #[token("+=")] PlusEq, #[token("-=")] MinusEq, #[token("*=")] StarEq,
    #[token("/=")] SlashEq, #[token("%=")] PercentEq,
    #[token("&=")] AndEq, #[token("|=")] OrEq, #[token("^=")] XorEq,
    #[token("<<=")] ShlEq, #[token(">>=")] ShrEq,
    #[token("++")] PlusPlus, #[token("--")] MinusMinus,
    #[token("=>")] Arrow, #[token("->")] ThinArrow,
    #[token("::")] ColonColon, #[token("..")] Range, #[token("...")] VarArgs,

    #[token("+")] Plus, #[token("-")] Minus, #[token("*")] Star, #[token("/")] Slash,
    #[token("%")] Percent, #[token("=")] Equals, #[token("<")] Lt, #[token(">")] Gt,
    #[token("!")] Bang, #[token("~")] Tilde, #[token("&")] Amp, #[token("|")] Pipe,
    #[token("^")] Caret, #[token("?")] Question,

    #[token(".")] Dot, #[token(",")] Comma, #[token(":")] Colon, #[token(";")] Semicolon,
    #[token("(")] LParen, #[token(")")] RParen, #[token("{")] LBrace, #[token("}")] RBrace,
    #[token("[")] LBracket, #[token("]")] RBracket,
}

pub fn tokenize(source: &str) -> Result<Vec<(Token, Span)>, Vec<CompilerError>> {
    let mut lexer = Token::lexer(source);
    let mut tokens = Vec::new();
    let mut errors = Vec::new();
    while let Some(tok) = lexer.next() {
        let s = lexer.span();
        match tok {
            Ok(t) => tokens.push((t, Span { start: s.start, end: s.end })),
            Err(_) => errors.push(CompilerError::LexerError {
                message: format!("Unknown token at {}", s.start),
                location: crate::error::SourceLocation { file: "input".into(), line: 1, column: s.start, span: Span { start: s.start, end: s.end } },
            }),
        }
    }
    if errors.is_empty() { Ok(tokens) } else { Err(errors) }
}
