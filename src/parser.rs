use crate::ast::*;
use crate::lexer::Token;
use crate::error::CompilerError;

pub fn parse(tokens: Vec<(Token, Span)>, _source: &str) -> Result<Module, Vec<CompilerError>> {
    let mut pos = 0;
    
    fn peek(tokens: &[(Token, Span)], pos: usize) -> Option<Token> { tokens.get(pos).map(|(t,_)| t.clone()) }
    fn next(tokens: &[(Token, Span)], pos: &mut usize) -> Option<(Token, Span)> { let t = tokens.get(*pos).cloned(); *pos += 1; t }
    
    // module Name;
    next(&tokens, &mut pos); // Module
    let name = match next(&tokens, &mut pos) {
        Some((Token::Identifier(n), _)) => n, _ => String::new()
    };
    next(&tokens, &mut pos); // Semicolon
    
    let mut decls = vec![];
    
    while pos < tokens.len() {
        match peek(&tokens, pos) {
            Some(Token::Import) => {
                while pos < tokens.len() && peek(&tokens, pos) != Some(Token::Semicolon) { next(&tokens, &mut pos); }
                next(&tokens, &mut pos); // ;
            }
            Some(Token::Class) => {
                next(&tokens, &mut pos); // class
                let class_name = match next(&tokens, &mut pos) {
                    Some((Token::Identifier(n), _)) => n, _ => { continue; }
                };
                next(&tokens, &mut pos); // {
                
                let mut methods = vec![];
                
                while pos < tokens.len() && peek(&tokens, pos) != Some(Token::RBrace) {
                    // Skip modifiers
                    while matches!(peek(&tokens, pos), Some(Token::Static) | Some(Token::Public) | Some(Token::Private)) {
                        next(&tokens, &mut pos);
                    }
                    
                    // Parse: type name (params) { body }
                    if let Some(Token::Identifier(_)) = peek(&tokens, pos) {
                        let saved = pos;
                        if let Some((Token::Identifier(ret_type), _)) = next(&tokens, &mut pos) {
                            if let Some(Token::Identifier(method_name)) = peek(&tokens, pos) {
                                next(&tokens, &mut pos);
                                
                                if peek(&tokens, pos) == Some(Token::LParen) {
                                    // It's a method
                                    next(&tokens, &mut pos); // (
                                    let mut depth = 1;
                                    while depth > 0 && pos < tokens.len() {
                                        match next(&tokens, &mut pos) {
                                            Some((Token::LParen, _)) => depth += 1,
                                            Some((Token::RParen, _)) => depth -= 1,
                                            _ => {}
                                        }
                                    }
                                    
                                    let body = if peek(&tokens, pos) == Some(Token::LBrace) {
                                        next(&tokens, &mut pos); // {
                                        let stmts = parse_block(&tokens, &mut pos);
                                        Some(stmts)
                                    } else if peek(&tokens, pos) == Some(Token::Semicolon) {
                                        next(&tokens, &mut pos);
                                        None
                                    } else {
                                        None
                                    };
                                    
                                    methods.push(FunctionDecl {
                                        name: method_name,
                                        params: vec![],
                                        return_type: str_to_type(&ret_type),
                                        body,
                                        is_extern: false,
                                        span: Span { start: 0, end: 0 },
                                    });
                                } else {
                                    // Field: type name;
                                    pos = saved;
                                    next(&tokens, &mut pos); // type
                                    next(&tokens, &mut pos); // name
                                    while pos < tokens.len() && peek(&tokens, pos) != Some(Token::Semicolon) { next(&tokens, &mut pos); }
                                    next(&tokens, &mut pos); // ;
                                }
                            }
                        }
                    } else {
                        next(&tokens, &mut pos);
                    }
                }
                if peek(&tokens, pos) == Some(Token::RBrace) { next(&tokens, &mut pos); }
                
                decls.push(Declaration::Class(ClassDecl {
                    name: class_name, bases: vec![], fields: vec![], methods,
                    span: Span { start: 0, end: 0 },
                }));
            }
            _ => { next(&tokens, &mut pos); }
        }
    }
    
    Ok(Module { name, imports: vec![], declarations: decls, span: Span { start: 0, end: 0 } })
}

fn parse_block(tokens: &[(Token, Span)], pos: &mut usize) -> Vec<Statement> {
    let mut stmts = vec![];
    while *pos < tokens.len() && peek(tokens, *pos) != Some(Token::RBrace) {
        if let Some(stmt) = parse_statement(tokens, pos) {
            stmts.push(stmt);
        }
    }
    if peek(tokens, *pos) == Some(Token::RBrace) { next(tokens, pos); }
    stmts
}

fn peek(tokens: &[(Token, Span)], pos: usize) -> Option<Token> { tokens.get(pos).map(|(t,_)| t.clone()) }
fn next(tokens: &[(Token, Span)], pos: &mut usize) -> Option<(Token, Span)> { let t = tokens.get(*pos).cloned(); *pos += 1; t }

fn parse_statement(tokens: &[(Token, Span)], pos: &mut usize) -> Option<Statement> {
    match peek(tokens, *pos) {
        Some(Token::Var) | Some(Token::Let) => {
            next(tokens, pos); // var
            let name = match next(tokens, pos) { Some((Token::Identifier(n), _)) => n, _ => return None };
            let init = if peek(tokens, *pos) == Some(Token::Equals) { next(tokens, pos); parse_expr(tokens, pos) } else { None };
            while *pos < tokens.len() && peek(tokens, *pos) != Some(Token::Semicolon) { next(tokens, pos); }
            next(tokens, pos); // ;
            Some(Statement::Variable(name, Type::Infer, init, Span { start: 0, end: 0 }))
        }
        Some(Token::Return) => {
            next(tokens, pos);
            let expr = if peek(tokens, *pos) != Some(Token::Semicolon) { parse_expr(tokens, pos) } else { None };
            while *pos < tokens.len() && peek(tokens, *pos) != Some(Token::Semicolon) { next(tokens, pos); }
            next(tokens, pos);
            Some(Statement::Return(expr, Span { start: 0, end: 0 }))
        }
        Some(Token::If) => {
            next(tokens, pos); next(tokens, pos); // if (
            let cond = parse_expr(tokens, pos)?;
            next(tokens, pos); // )
            let then = Box::new(parse_statement(tokens, pos)?);
            let els = if peek(tokens, *pos) == Some(Token::Else) { next(tokens, pos); Some(Box::new(parse_statement(tokens, pos)?)) } else { None };
            Some(Statement::If(cond, then, els, Span { start: 0, end: 0 }))
        }
        Some(Token::While) => {
            next(tokens, pos); next(tokens, pos);
            let cond = parse_expr(tokens, pos)?;
            next(tokens, pos);
            Some(Statement::While(cond, Box::new(parse_statement(tokens, pos)?), Span { start: 0, end: 0 }))
        }
        Some(Token::For) => {
            next(tokens, pos); next(tokens, pos);
            let init = Box::new(parse_statement(tokens, pos)?);
            let cond = if peek(tokens, *pos) != Some(Token::Semicolon) { parse_expr(tokens, pos) } else { None };
            next(tokens, pos);
            let inc = if peek(tokens, *pos) != Some(Token::RParen) { parse_expr(tokens, pos) } else { None };
            next(tokens, pos);
            Some(Statement::For(init, cond, inc, Box::new(parse_statement(tokens, pos)?), Span { start: 0, end: 0 }))
        }
        Some(Token::LBrace) => {
            next(tokens, pos); // {
            let stmts = parse_block(tokens, pos);
            Some(Statement::Block(stmts, Span { start: 0, end: 0 }))
        }
        _ => {
            let expr = parse_expr(tokens, pos)?;
            while *pos < tokens.len() && peek(tokens, *pos) != Some(Token::Semicolon) { next(tokens, pos); }
            next(tokens, pos);
            Some(Statement::Expression(expr, Span { start: 0, end: 0 }))
        }
    }
}

fn parse_expr(tokens: &[(Token, Span)], pos: &mut usize) -> Option<Expression> {
    let (t, sp) = next(tokens, pos)?;
    match t {
        Token::IntLit(n) => Some(Expression::Literal(Literal::Int(n), sp)),
        Token::StringLit(s) => Some(Expression::Literal(Literal::String(s), sp)),
        Token::True => Some(Expression::Literal(Literal::Bool(true), sp)),
        Token::False => Some(Expression::Literal(Literal::Bool(false), sp)),
        Token::Identifier(name) => {
            if peek(tokens, *pos) == Some(Token::LParen) {
                next(tokens, pos); // (
                let mut args = vec![];
                while *pos < tokens.len() && peek(tokens, *pos) != Some(Token::RParen) {
                    if let Some(e) = parse_expr(tokens, pos) { args.push(e); }
                    if peek(tokens, *pos) == Some(Token::Comma) { next(tokens, pos); }
                }
                next(tokens, pos); // )
                Some(Expression::Call(name, args, sp))
            } else {
                let op = match peek(tokens, *pos) {
                    Some(Token::Plus) => Some(BinOp::Add), Some(Token::Minus) => Some(BinOp::Sub),
                    Some(Token::Star) => Some(BinOp::Mul), Some(Token::Slash) => Some(BinOp::Div),
                    Some(Token::EqEq) => Some(BinOp::Eq), Some(Token::Lt) => Some(BinOp::Lt),
                    Some(Token::Gt) => Some(BinOp::Gt), Some(Token::LtEq) => Some(BinOp::Le),
                    Some(Token::GtEq) => Some(BinOp::Ge), Some(Token::AndAnd) => Some(BinOp::And),
                    Some(Token::OrOr) => Some(BinOp::Or),
                    _ => None,
                };
                if let Some(op) = op {
                    next(tokens, pos);
                    let r = parse_expr(tokens, pos)?;
                    Some(Expression::Binary(Box::new(Expression::Identifier(name, sp)), op, Box::new(r), sp))
                } else {
                    Some(Expression::Identifier(name, sp))
                }
            }
        }
        _ => None,
    }
}
