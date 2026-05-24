use crate::ast::*;
use crate::error::{CompilerError, SourceLocation};
use crate::lexer::Token;

pub struct Parser<'a> {
    tokens: Vec<(Token, Span)>,
    pos: usize,
    source: &'a str,
}

impl<'a> Parser<'a> {
    pub fn new(src: &'a str, tokens: Vec<(Token, Span)>) -> Self { Parser { tokens, pos: 0, source: src } }

    fn peek(&self) -> Option<&Token> { self.tokens.get(self.pos).map(|(t,_)| t) }
    fn next(&mut self) -> Option<(Token, Span)> { let t = self.tokens.get(self.pos).cloned(); self.pos += 1; t }

    fn expect_id(&mut self) -> Result<(String, Span), CompilerError> {
        match self.next() {
            Some((Token::Identifier(s), sp)) => Ok((s, sp)),
            Some((t, sp)) => Err(CompilerError::parser_error("Expected identifier", self.loc(sp), vec!["id".into()], &format!("{:?}", t))),
            None => Err(CompilerError::parser_error("Unexpected EOF", self.loc(Span{start:0,end:0}), vec!["id".into()], "EOF")),
        }
    }

    fn expect(&mut self, exp: &[Token]) -> Result<(Token, Span), CompilerError> {
        match self.next() {
            Some((t, sp)) if exp.is_empty() || exp.contains(&t) => Ok((t, sp)),
            Some((t, sp)) => Err(CompilerError::parser_error("Unexpected token", self.loc(sp), exp.iter().map(|x| format!("{:?}",x)).collect(), &format!("{:?}", t))),
            None => Err(CompilerError::parser_error("Unexpected EOF", self.loc(Span{start:0,end:0}), exp.iter().map(|x| format!("{:?}",x)).collect(), "EOF")),
        }
    }

    fn loc(&self, span: Span) -> SourceLocation { SourceLocation { file: "input.tr".into(), line: 1, column: span.start, span } }
    fn match_id(&self, name: &str) -> bool { matches!(self.peek(), Some(Token::Identifier(s)) if s == name) }

    pub fn parse_module(&mut self) -> Result<Module, Vec<CompilerError>> {
        let mut errs = Vec::new();
        self.expect(&[Token::Module]).ok();
        let name = match self.expect_id() { Ok((n,_)) => n, Err(e) => { errs.push(e); return Err(errs); } };
        self.expect(&[Token::Semicolon]).ok();

        let mut imports = vec![];
        let mut decls = vec![];

        while self.peek().is_some() {
            match self.peek() {
                Some(Token::Import) => {
                    self.next();
                    let mut path = String::new();
                    loop {
                        match self.expect(&[]) {
                            Ok((Token::Identifier(p), _)) => path = format!("{}{}", path, p),
                            Ok((Token::Dot, _)) => path.push('.'),
                            Ok((Token::Semicolon, _)) => break,
                            _ => break,
                        }
                    }
                    imports.push(path);
                }
                Some(Token::Class) => {
                    if let Ok(d) = self.parse_class() { decls.push(d); }
                }
                Some(Token::Static) | Some(Token::Identifier(_)) => {
                    let saved = self.pos;
                    if let Ok(func) = self.parse_function() {
                        decls.push(Declaration::Function(func));
                    } else {
                        self.pos = saved;
                        self.next();
                    }
                }
                _ => { self.next(); }
            }
        }

        if errs.is_empty() { Ok(Module { name, imports, declarations: decls, span: Span{start:0,end:self.source.len()} }) }
        else { Err(errs) }
    }

    fn parse_function(&mut self) -> Result<FunctionDecl, CompilerError> {
        let saved = self.pos;
        while matches!(self.peek(), Some(Token::Static) | Some(Token::Extern) | Some(Token::Unsafe) | Some(Token::Public) | Some(Token::Private)) {
            self.next();
        }
        let (ret_or_name, _) = self.expect_id()?;
        let (ret_type, func_name, func_span) = if self.peek() == Some(&Token::LParen) {
            (Type::Infer, ret_or_name, Span{start:0,end:0})
        } else {
            let (name, sp) = self.expect_id()?;
            if self.peek() == Some(&Token::LParen) {
                (str_to_type(&ret_or_name), name, sp)
            } else {
                self.pos = saved;
                return Err(CompilerError::parser_error("Not a function", self.loc(Span{start:0,end:0}), vec![], ""));
            }
        };
        self.expect(&[Token::LParen])?;
        let mut params = vec![];
        while self.peek() != Some(&Token::RParen) && self.peek().is_some() {
            let (pt, _) = self.expect_id()?;
            let (pn, ps) = self.expect_id()?;
            params.push(Parameter { name: pn, ty: str_to_type(&pt), span: ps });
            if self.peek() == Some(&Token::Comma) { self.next(); }
        }
        self.expect(&[Token::RParen])?;
        let body = if self.peek() == Some(&Token::Semicolon) { self.next(); None } else { Some(self.parse_block()?) };
        Ok(FunctionDecl { name: func_name, params, return_type: ret_type, body, is_extern: false, span: func_span })
    }

    fn parse_class(&mut self) -> Result<Declaration, CompilerError> {
        self.expect(&[Token::Class])?;
        let (name, sp) = self.expect_id()?;
        self.expect(&[Token::LBrace])?;
        let mut methods = vec![];
        while self.peek() != Some(&Token::RBrace) && self.peek().is_some() {
            if self.peek() == Some(&Token::Static) { self.next(); }
            if self.match_id("int") || self.match_id("float") || self.match_id("string") || self.match_id("bool") || self.match_id("void") || self.match_id("i32") || self.match_id("i64") || self.match_id("f32") || self.match_id("f64") {
                let (ret_name, _) = self.expect_id()?;
                if let Some(Token::Identifier(_)) = self.peek() {
                    let (fname, fsp) = self.expect_id()?;
                    if self.peek() == Some(&Token::LParen) {
                        self.next();
                        let mut params = vec![];
                        while self.peek() != Some(&Token::RParen) && self.peek().is_some() {
                            let (pt, _) = self.expect_id()?;
                            let (pn, ps) = self.expect_id()?;
                            params.push(Parameter { name: pn, ty: str_to_type(&pt), span: ps });
                            if self.peek() == Some(&Token::Comma) { self.next(); }
                        }
                        self.expect(&[Token::RParen])?;
                        let body = Some(self.parse_block()?);
                        methods.push(FunctionDecl { name: fname, params, return_type: str_to_type(&ret_name), body, is_extern: false, span: fsp });
                    } else { self.expect(&[Token::Semicolon])?; }
                }
            } else { self.next(); }
        }
        self.expect(&[Token::RBrace])?;
        Ok(Declaration::Class(ClassDecl { name, bases: vec![], fields: vec![], methods, span: sp }))
    }

    fn parse_block(&mut self) -> Result<Vec<Statement>, CompilerError> {
        self.expect(&[Token::LBrace])?;
        let mut stmts = vec![];
        while self.peek() != Some(&Token::RBrace) && self.peek().is_some() {
            match self.peek() {
                Some(Token::Var) | Some(Token::Let) => {
                    self.next();
                    let (name, _) = self.expect_id()?;
                    let ty = if self.peek() == Some(&Token::Colon) { self.next(); self.parse_type()? } else { Type::Infer };
                    let init = if self.peek() == Some(&Token::Equals) { self.next(); Some(self.parse_expr()?) } else { None };
                    self.expect(&[Token::Semicolon])?;
                    stmts.push(Statement::Variable(name, ty, init, Span{start:0,end:0}));
                }
                Some(Token::Return) => {
                    self.next();
                    let expr = if self.peek() != Some(&Token::Semicolon) { Some(self.parse_expr()?) } else { None };
                    self.expect(&[Token::Semicolon])?;
                    stmts.push(Statement::Return(expr, Span{start:0,end:0}));
                }
                Some(Token::If) => {
                    self.next();
                    self.expect(&[Token::LParen])?;
                    let cond = self.parse_expr()?;
                    self.expect(&[Token::RParen])?;
                    let then = Box::new(self.parse_stmt()?);
                    let els = if self.peek() == Some(&Token::Else) { self.next(); Some(Box::new(self.parse_stmt()?)) } else { None };
                    stmts.push(Statement::If(cond, then, els, Span{start:0,end:0}));
                }
                Some(Token::While) => {
                    self.next();
                    self.expect(&[Token::LParen])?;
                    let cond = self.parse_expr()?;
                    self.expect(&[Token::RParen])?;
                    let body = Box::new(self.parse_stmt()?);
                    stmts.push(Statement::While(cond, body, Span{start:0,end:0}));
                }
                Some(Token::For) => {
                    self.next();
                    self.expect(&[Token::LParen])?;
                    let init = Box::new(self.parse_stmt()?);
                    let cond = if self.peek() != Some(&Token::Semicolon) { Some(self.parse_expr()?) } else { None };
                    self.expect(&[Token::Semicolon])?;
                    let inc = if self.peek() != Some(&Token::RParen) { Some(self.parse_expr()?) } else { None };
                    self.expect(&[Token::RParen])?;
                    let body = Box::new(self.parse_stmt()?);
                    stmts.push(Statement::For(init, cond, inc, body, Span{start:0,end:0}));
                }
                Some(Token::Break) => { self.next(); self.expect(&[Token::Semicolon])?; stmts.push(Statement::Break(Span{start:0,end:0})); }
                Some(Token::Continue) => { self.next(); self.expect(&[Token::Semicolon])?; stmts.push(Statement::Continue(Span{start:0,end:0})); }
                _ => {
                    let expr = self.parse_expr()?;
                    self.expect(&[Token::Semicolon])?;
                    stmts.push(Statement::Expression(expr, Span{start:0,end:0}));
                }
            }
        }
        self.expect(&[Token::RBrace])?;
        Ok(stmts)
    }

    fn parse_stmt(&mut self) -> Result<Statement, CompilerError> {
        if self.peek() == Some(&Token::LBrace) {
            Ok(Statement::Block(self.parse_block()?, Span{start:0,end:0}))
        } else {
            let mut v: Vec<Statement> = vec![];
            match self.peek() {
                Some(Token::Var) | Some(Token::Let) => {
                    self.next();
                    let (name, _) = self.expect_id()?;
                    let ty = if self.peek() == Some(&Token::Colon) { self.next(); self.parse_type()? } else { Type::Infer };
                    let init = if self.peek() == Some(&Token::Equals) { self.next(); Some(self.parse_expr()?) } else { None };
                    self.expect(&[Token::Semicolon])?;
                    return Ok(Statement::Variable(name, ty, init, Span{start:0,end:0}));
                }
                Some(Token::Return) => {
                    self.next();
                    let expr = if self.peek() != Some(&Token::Semicolon) { Some(self.parse_expr()?) } else { None };
                    self.expect(&[Token::Semicolon])?;
                    return Ok(Statement::Return(expr, Span{start:0,end:0}));
                }
                Some(Token::If) => {
                    self.next();
                    self.expect(&[Token::LParen])?;
                    let cond = self.parse_expr()?;
                    self.expect(&[Token::RParen])?;
                    let then = Box::new(self.parse_stmt()?);
                    let els = if self.peek() == Some(&Token::Else) { self.next(); Some(Box::new(self.parse_stmt()?)) } else { None };
                    return Ok(Statement::If(cond, then, els, Span{start:0,end:0}));
                }
                Some(Token::While) => {
                    self.next();
                    self.expect(&[Token::LParen])?;
                    let cond = self.parse_expr()?;
                    self.expect(&[Token::RParen])?;
                    let body = Box::new(self.parse_stmt()?);
                    return Ok(Statement::While(cond, body, Span{start:0,end:0}));
                }
                Some(Token::For) => {
                    self.next();
                    self.expect(&[Token::LParen])?;
                    let init = Box::new(self.parse_stmt()?);
                    let cond = if self.peek() != Some(&Token::Semicolon) { Some(self.parse_expr()?) } else { None };
                    self.expect(&[Token::Semicolon])?;
                    let inc = if self.peek() != Some(&Token::RParen) { Some(self.parse_expr()?) } else { None };
                    self.expect(&[Token::RParen])?;
                    let body = Box::new(self.parse_stmt()?);
                    return Ok(Statement::For(init, cond, inc, body, Span{start:0,end:0}));
                }
                _ => {}
            }
            let expr = self.parse_expr()?;
            self.expect(&[Token::Semicolon]).ok();
            Ok(Statement::Expression(expr, Span{start:0,end:0}))
        }
    }

    fn parse_type(&mut self) -> Result<Type, CompilerError> {
        let (t, _) = self.expect_id()?;
        Ok(str_to_type(&t))
    }

    fn parse_expr(&mut self) -> Result<Expression, CompilerError> {
        let (t, sp) = self.expect(&[])?;
        match t {
            Token::IntLit(n) => Ok(Expression::Literal(Literal::Int(n), sp)),
            Token::FloatLit(f) => Ok(Expression::Literal(Literal::Float(f), sp)),
            Token::StringLit(s) => Ok(Expression::Literal(Literal::String(s), sp)),
            Token::CharLit(c) => Ok(Expression::Literal(Literal::Char(c), sp)),
            Token::True => Ok(Expression::Literal(Literal::Bool(true), sp)),
            Token::False => Ok(Expression::Literal(Literal::Bool(false), sp)),
            Token::Null => Ok(Expression::Literal(Literal::Null, sp)),
            Token::Identifier(name) => {
                if self.peek() == Some(&Token::LParen) {
                    self.next();
                    let mut args = vec![];
                    while self.peek() != Some(&Token::RParen) && self.peek().is_some() {
                        args.push(self.parse_expr()?);
                        if self.peek() == Some(&Token::Comma) { self.next(); }
                    }
                    self.expect(&[Token::RParen])?;
                    Ok(Expression::Call(name, args, sp))
                } else if self.peek() == Some(&Token::Plus) {
                    self.next(); let r = self.parse_expr()?;
                    Ok(Expression::Binary(Box::new(Expression::Identifier(name, sp)), BinOp::Add, Box::new(r), sp))
                } else if self.peek() == Some(&Token::Minus) {
                    self.next(); let r = self.parse_expr()?;
                    Ok(Expression::Binary(Box::new(Expression::Identifier(name, sp)), BinOp::Sub, Box::new(r), sp))
                } else if self.peek() == Some(&Token::Star) {
                    self.next(); let r = self.parse_expr()?;
                    Ok(Expression::Binary(Box::new(Expression::Identifier(name, sp)), BinOp::Mul, Box::new(r), sp))
                } else if self.peek() == Some(&Token::Slash) {
                    self.next(); let r = self.parse_expr()?;
                    Ok(Expression::Binary(Box::new(Expression::Identifier(name, sp)), BinOp::Div, Box::new(r), sp))
                } else if self.peek() == Some(&Token::EqEq) {
                    self.next(); let r = self.parse_expr()?;
                    Ok(Expression::Binary(Box::new(Expression::Identifier(name, sp)), BinOp::Eq, Box::new(r), sp))
                } else if self.peek() == Some(&Token::Lt) {
                    self.next(); let r = self.parse_expr()?;
                    Ok(Expression::Binary(Box::new(Expression::Identifier(name, sp)), BinOp::Lt, Box::new(r), sp))
                } else if self.peek() == Some(&Token::Gt) {
                    self.next(); let r = self.parse_expr()?;
                    Ok(Expression::Binary(Box::new(Expression::Identifier(name, sp)), BinOp::Gt, Box::new(r), sp))
                } else {
                    Ok(Expression::Identifier(name, sp))
                }
            }
            Token::New => {
                if self.peek() == Some(&Token::LBracket) {
                    self.next();
                    let size = self.parse_expr()?;
                    self.expect(&[Token::RBracket])?;
                    let ty = self.parse_type()?;
                    Ok(Expression::NewArray(ty, vec![size], sp))
                } else {
                    let ty = self.parse_type()?;
                    let args = if self.peek() == Some(&Token::LParen) {
                        self.next(); let mut a = vec![];
                        while self.peek() != Some(&Token::RParen) && self.peek().is_some() {
                            a.push(self.parse_expr()?);
                            if self.peek() == Some(&Token::Comma) { self.next(); }
                        }
                        self.expect(&[Token::RParen])?; a
                    } else { vec![] };
                    Ok(Expression::New(ty, args, sp))
                }
            }
            Token::SizeOf => { self.expect(&[Token::LParen])?; let ty = self.parse_type()?; self.expect(&[Token::RParen])?; Ok(Expression::SizeOf(ty, sp)) }
            Token::LParen => { let expr = self.parse_expr()?; self.expect(&[Token::RParen])?; Ok(expr) }
            Token::LBracket => {
                let mut items = vec![];
                while self.peek() != Some(&Token::RBracket) && self.peek().is_some() {
                    items.push(self.parse_expr()?);
                    if self.peek() == Some(&Token::Comma) { self.next(); }
                }
                self.expect(&[Token::RBracket])?;
                Ok(Expression::ArrayLiteral(items, sp))
            }
            Token::LBrace => {
                let mut fields = vec![];
                while self.peek() != Some(&Token::RBrace) && self.peek().is_some() {
                    let (n, _) = self.expect_id()?;
                    self.expect(&[Token::Colon])?;
                    let v = self.parse_expr()?;
                    fields.push((n, v));
                    if self.peek() == Some(&Token::Comma) { self.next(); }
                }
                self.expect(&[Token::RBrace])?;
                Ok(Expression::ObjectLiteral(fields, sp))
            }
            Token::Bang => { let expr = self.parse_expr()?; Ok(Expression::Unary(UnaryOp::Not, Box::new(expr), sp)) }
            Token::Minus => { let expr = self.parse_expr()?; Ok(Expression::Unary(UnaryOp::Neg, Box::new(expr), sp)) }
            _ => { eprintln!("Warning: unexpected token {:?}, using 0", t); Ok(Expression::Literal(Literal::Int(0), sp)) }
        }
    }
}

pub fn parse(tokens: Vec<(Token, Span)>, source: &str) -> Result<Module, Vec<CompilerError>> {
    Parser::new(source, tokens).parse_module()
}
