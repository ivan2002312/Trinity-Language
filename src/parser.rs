use crate::ast::*;
use crate::lexer::Token;
use crate::error::CompilerError;

pub fn parse(tokens: Vec<(Token, Span)>, _source: &str) -> Result<Module, Vec<CompilerError>> {
    let mut pos = 0;
    
    fn peek(tokens: &[(Token, Span)], pos: usize) -> Option<Token> { tokens.get(pos).map(|(t,_)| t.clone()) }
    fn next(tokens: &[(Token, Span)], pos: &mut usize) -> Option<(Token, Span)> { let t = tokens.get(*pos).cloned(); *pos += 1; t }
    
    next(&tokens, &mut pos);
    let name = match next(&tokens, &mut pos) { Some((Token::Identifier(n), _)) => n, _ => String::new() };
    next(&tokens, &mut pos);
    
    let mut imports = vec![];
    let mut decls = vec![];
    
    while pos < tokens.len() {
        match peek(&tokens, pos) {
            Some(Token::Import) => {
                next(&tokens, &mut pos);
                let mut path = String::new();
                while pos < tokens.len() && peek(&tokens, pos) != Some(Token::Semicolon) {
                    match peek(&tokens, pos) {
                        Some(Token::Identifier(_)) => { if let Some((Token::Identifier(p), _)) = next(&tokens, &mut pos) { path.push_str(&p); } }
                        Some(Token::Dot) => { next(&tokens, &mut pos); path.push('.'); }
                        _ => { next(&tokens, &mut pos); }
                    }
                }
                next(&tokens, &mut pos);
                imports.push(path);
            }
            Some(Token::Class) => {
                next(&tokens, &mut pos);
                let class_name = match next(&tokens, &mut pos) { Some((Token::Identifier(n), _)) => n, _ => { continue; } };
                next(&tokens, &mut pos);
                let mut methods = vec![];
                while pos < tokens.len() && peek(&tokens, pos) != Some(Token::RBrace) {
                    while matches!(peek(&tokens, pos), Some(Token::Static) | Some(Token::Public) | Some(Token::Private)) { next(&tokens, &mut pos); }
                    if let Some(Token::Identifier(_)) = peek(&tokens, pos) {
                        let saved = pos;
                        if let Some((Token::Identifier(ret_type), _)) = next(&tokens, &mut pos) {
                            if let Some(Token::Identifier(method_name)) = peek(&tokens, pos) {
                                next(&tokens, &mut pos);
                                if peek(&tokens, pos) == Some(Token::LParen) {
                                    next(&tokens, &mut pos); // (
                                    let mut params = vec![];
                                    while peek(&tokens, pos) != Some(Token::RParen) && pos < tokens.len() {
                                        if let Some((Token::Identifier(pt), _)) = next(&tokens, &mut pos) {
                                            if let Some((Token::Identifier(pn), ps)) = next(&tokens, &mut pos) {
                                                params.push(Parameter { name: pn, ty: str_to_type(&pt), span: ps });
                                            }
                                        }
                                        if peek(&tokens, pos) == Some(Token::Comma) { next(&tokens, &mut pos); }
                                    }
                                    next(&tokens, &mut pos); // )
                                    
                                    let body = if peek(&tokens, pos) == Some(Token::LBrace) {
                                        next(&tokens, &mut pos);
                                        Some(parse_block(&tokens, &mut pos))
                                    } else if peek(&tokens, pos) == Some(Token::Semicolon) { next(&tokens, &mut pos); None } else { None };
                                    methods.push(FunctionDecl { name: method_name, params, return_type: str_to_type(&ret_type), body, is_extern: false, span: Span { start: 0, end: 0 } });
                                } else { pos = saved; next(&tokens, &mut pos); next(&tokens, &mut pos); while pos < tokens.len() && peek(&tokens, pos) != Some(Token::Semicolon) { next(&tokens, &mut pos); } next(&tokens, &mut pos); }
                            }
                        }
                    } else { next(&tokens, &mut pos); }
                }
                if peek(&tokens, pos) == Some(Token::RBrace) { next(&tokens, &mut pos); }
                decls.push(Declaration::Class(ClassDecl { name: class_name, bases: vec![], fields: vec![], methods, span: Span { start: 0, end: 0 } }));
            }
            _ => { next(&tokens, &mut pos); }
        }
    }
    
    Ok(Module { name, imports, declarations: decls, span: Span { start: 0, end: 0 } })
}

fn parse_block(tokens: &[(Token, Span)], pos: &mut usize) -> Vec<Statement> {
    let mut stmts = vec![];
    while *pos < tokens.len() && peek(tokens, *pos) != Some(Token::RBrace) {
        if let Some(stmt) = parse_statement(tokens, pos) { stmts.push(stmt); }
    }
    if peek(tokens, *pos) == Some(Token::RBrace) { next(tokens, pos); }
    stmts
}

fn peek(tokens: &[(Token, Span)], pos: usize) -> Option<Token> { tokens.get(pos).map(|(t,_)| t.clone()) }
fn next(tokens: &[(Token, Span)], pos: &mut usize) -> Option<(Token, Span)> { let t = tokens.get(*pos).cloned(); *pos += 1; t }

fn parse_statement(tokens: &[(Token, Span)], pos: &mut usize) -> Option<Statement> {
    match peek(tokens, *pos) {
        Some(Token::Var) | Some(Token::Let) => {
            next(tokens, pos); let name = match next(tokens, pos) { Some((Token::Identifier(n), _)) => n, _ => return None };
            let init = if peek(tokens, *pos) == Some(Token::Equals) { next(tokens, pos); let e = parse_expr(tokens, pos); if e.is_none() { return None; } e } else { None };
            while *pos < tokens.len() && peek(tokens, *pos) != Some(Token::Semicolon) { next(tokens, pos); }
            next(tokens, pos);
            Some(Statement::Variable(name, Type::Infer, init, Span { start: 0, end: 0 }))
        }
        Some(Token::Return) => {
            next(tokens, pos); let expr = if peek(tokens, *pos) != Some(Token::Semicolon) { parse_expr(tokens, pos) } else { None };
            while *pos < tokens.len() && peek(tokens, *pos) != Some(Token::Semicolon) { next(tokens, pos); }
            next(tokens, pos);
            Some(Statement::Return(expr, Span { start: 0, end: 0 }))
        }
        Some(Token::If) => {
            next(tokens, pos); next(tokens, pos); let cond = parse_expr(tokens, pos)?; next(tokens, pos);
            let then = Box::new(parse_statement(tokens, pos)?);
            let els = if peek(tokens, *pos) == Some(Token::Else) { next(tokens, pos); Some(Box::new(parse_statement(tokens, pos)?)) } else { None };
            Some(Statement::If(cond, then, els, Span { start: 0, end: 0 }))
        }
        Some(Token::While) => {
            next(tokens, pos); next(tokens, pos); let cond = parse_expr(tokens, pos)?; next(tokens, pos);
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
        Some(Token::LBrace) => { next(tokens, pos); let s = parse_block(tokens, pos); Some(Statement::Block(s, Span { start: 0, end: 0 })) }
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
    let mut expr = match t {
        Token::IntLit(n) => Expression::Literal(Literal::Int(n), sp),
        Token::StringLit(s) => Expression::Literal(Literal::String(s), sp),
        Token::True => Expression::Literal(Literal::Bool(true), sp),
        Token::False => Expression::Literal(Literal::Bool(false), sp),
        Token::Identifier(name) => Expression::Identifier(name, sp),
        _ => return None,
    };
    
    loop {
        match peek(tokens, *pos) {
            Some(Token::Dot) => {
                next(tokens, pos);
                if let Some((Token::Identifier(member), _)) = next(tokens, pos) {
                    if peek(tokens, *pos) == Some(Token::LParen) {
                        next(tokens, pos);
                        let mut args = vec![];
                        while *pos < tokens.len() && peek(tokens, *pos) != Some(Token::RParen) {
                            if let Some(e) = parse_expr(tokens, pos) { args.push(e); }
                            if peek(tokens, *pos) == Some(Token::Comma) { next(tokens, pos); }
                        }
                        next(tokens, pos);
                        let name = match &expr { Expression::Identifier(n, _) => format!("{}.{}", n, member), _ => member };
                        expr = Expression::Call(name, args, sp);
                    } else {
                        expr = Expression::MemberAccess(Box::new(expr), member, sp);
                    }
                }
            }
            Some(Token::LParen) => {
                next(tokens, pos);
                let mut args = vec![];
                while *pos < tokens.len() && peek(tokens, *pos) != Some(Token::RParen) {
                    if let Some(e) = parse_expr(tokens, pos) { args.push(e); }
                    if peek(tokens, *pos) == Some(Token::Comma) { next(tokens, pos); }
                }
                next(tokens, pos);
                let name = match &expr { Expression::Identifier(n, _) => n.clone(), _ => String::new() };
                expr = Expression::Call(name, args, sp);
            }
            Some(Token::Plus) | Some(Token::Minus) | Some(Token::Star) | Some(Token::Slash) => {
                let op = match peek(tokens, *pos) {
                    Some(Token::Plus) => BinOp::Add, Some(Token::Minus) => BinOp::Sub,
                    Some(Token::Star) => BinOp::Mul, Some(Token::Slash) => BinOp::Div,
                    _ => break,
                };
                next(tokens, pos);
                let r = parse_expr(tokens, pos)?;
                expr = Expression::Binary(Box::new(expr), op, Box::new(r), sp);
            }
            _ => break,
        }
    }
    
    Some(expr)
}
