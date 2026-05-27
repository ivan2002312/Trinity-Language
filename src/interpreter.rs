use crate::ast::*;
use std::collections::HashMap;
use std::io::Read;

#[derive(Debug, Clone)]
pub enum Value { Void, Bool(bool), Int(i64), Float(f64), String(String), Array(Vec<Value>), Null }

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self { Value::Void => write!(f, ""), Value::Bool(b) => write!(f, "{}", b), Value::Int(n) => write!(f, "{}", n), Value::Float(x) => write!(f, "{}", x), Value::String(s) => write!(f, "{}", s), Value::Array(a) => write!(f, "[{}]", a.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(", ")), Value::Null => write!(f, "null") }
    }
}

impl Value {
    fn as_bool(&self) -> bool { match self { Value::Bool(b) => *b, Value::Int(n) => *n != 0, Value::Null => false, _ => true } }
    fn as_int(&self) -> i64 { match self { Value::Int(n) => *n, _ => 0 } }
    fn as_str(&self) -> String { format!("{}", self) }
}

pub struct Interpreter {
    scopes: Vec<HashMap<String, Value>>,
    functions: HashMap<String, FunctionDecl>,
    modules: HashMap<String, HashMap<String, FunctionDecl>>,
    return_val: Option<Value>,
    break_flag: bool,
    continue_flag: bool,
}

impl Interpreter {
    pub fn new() -> Self { Interpreter { scopes: vec![HashMap::new()], functions: HashMap::new(), modules: HashMap::new(), return_val: None, break_flag: false, continue_flag: false } }
    fn push(&mut self) { self.scopes.push(HashMap::new()); }
    fn pop(&mut self) { self.scopes.pop(); }
    fn set(&mut self, k: &str, v: Value) { self.scopes.last_mut().unwrap().insert(k.to_string(), v); }
    fn get(&self, k: &str) -> Result<Value, String> { for s in self.scopes.iter().rev() { if let Some(v) = s.get(k) { return Ok(v.clone()); } } Err(format!("Undefined: {}", k)) }
    pub fn register_module(&mut self, name: String, funcs: HashMap<String, FunctionDecl>) { self.modules.insert(name, funcs); }

    pub fn execute_module(&mut self, m: &Module) -> Result<Value, String> {
        for d in &m.declarations {
            if let Declaration::Function(f) = d { self.functions.insert(f.name.clone(), f.clone()); }
            if let Declaration::Class(c) = d { for m in &c.methods { self.functions.insert(m.name.clone(), m.clone()); } }
        }
        if let Some(f) = self.functions.get("main").cloned() { return self.call_fn(&f, &[]); }
        for d in &m.declarations {
            if let Declaration::Class(c) = d { for m in &c.methods { if m.name == "main" { return self.call_fn(m, &[]); } } }
        }
        Err("No main".into())
    }

    fn call_fn(&mut self, f: &FunctionDecl, args: &[Value]) -> Result<Value, String> {
        self.push();
        for (i, p) in f.params.iter().enumerate() { let v = if i < args.len() { args[i].clone() } else { Value::Null }; self.set(&p.name, v); }
        self.return_val = None;
        if let Some(b) = &f.body { self.exec_block(b)?; }
        let r = self.return_val.clone().unwrap_or(Value::Void);
        self.pop();
        Ok(r)
    }

    fn exec_block(&mut self, stmts: &[Statement]) -> Result<(), String> {
        for s in stmts { if self.return_val.is_some() || self.break_flag || self.continue_flag { break; } self.exec_stmt(s)?; }
        Ok(())
    }

    fn exec_stmt(&mut self, s: &Statement) -> Result<(), String> {
        match s {
            Statement::Variable(n, _, init, _) => { let v = init.as_ref().map(|e| self.eval(e)).unwrap_or(Ok(Value::Null))?; self.set(n, v); }
            Statement::Expression(e, _) => { self.eval(e)?; }
            Statement::Block(stmts, _) => { self.push(); self.exec_block(stmts)?; self.pop(); }
            Statement::If(c, t, e, _) => { if self.eval(c)?.as_bool() { self.exec_stmt(t)?; } else if let Some(els) = e { self.exec_stmt(els)?; } }
            Statement::While(c, b, _) => { while self.eval(c)?.as_bool() { self.exec_stmt(b)?; if self.break_flag { self.break_flag = false; break; } } }
            Statement::For(i, c, inc, b, _) => { self.push(); self.exec_stmt(i)?; while c.as_ref().map_or(true, |x| self.eval(x).map_or(false, |v| v.as_bool())) { self.exec_stmt(b)?; if self.break_flag { self.break_flag = false; break; } if let Some(inc) = inc { self.eval(inc)?; } } self.pop(); }
            Statement::Return(e, _) => { self.return_val = Some(e.as_ref().map(|x| self.eval(x)).unwrap_or(Ok(Value::Void))?); }
            Statement::Break(_) => self.break_flag = true,
            _ => {}
        }
        Ok(())
    }

    pub fn eval(&mut self, e: &Expression) -> Result<Value, String> {
        match e {
            Expression::Literal(l, _) => Ok(match l { Literal::Int(n) => Value::Int(*n), Literal::Float(f) => Value::Float(*f), Literal::String(s) => Value::String(s.clone()), Literal::Bool(b) => Value::Bool(*b), Literal::Null => Value::Null, _ => Value::Null }),
            Expression::Identifier(n, _) => self.get(n),
            Expression::Binary(l, op, r, _) => {
                let lv = self.eval(l)?; let rv = self.eval(r)?;
                match (&lv, &rv) {
                    (Value::Int(a), Value::Int(b)) => Ok(Value::Int(match op { BinOp::Add => a+b, BinOp::Sub => a-b, BinOp::Mul => a*b, BinOp::Div => a/b, BinOp::Mod => a%b, BinOp::Eq => return Ok(Value::Bool(a==b)), BinOp::Lt => return Ok(Value::Bool(a<b)), BinOp::Gt => return Ok(Value::Bool(a>b)), _ => 0 })),
                    (Value::String(a), Value::String(b)) if *op == BinOp::Add => Ok(Value::String(format!("{}{}", a, b))),
                    _ => Err("Cannot op".into()),
                }
            }
            Expression::Call(name, args, _) => {
                let mut av = vec![]; for a in args { av.push(self.eval(a)?); }
                
                match name.as_str() {
                    "read_line" => { let mut s = String::new(); std::io::stdin().read_line(&mut s).ok(); return Ok(Value::String(s.trim().to_string())); } "print" | "println" => { for a in &av { print!("{}", a); } if name == "println" { println!(); } return Ok(Value::Void); }
                    
                    "http_get" => {
                        let url = av.get(0).map(|v| v.as_str()).unwrap_or_default();
                        match ureq::get(&url).call() {
                            Ok(resp) => {
                                let mut body = String::new();
                                resp.into_reader().read_to_string(&mut body).ok();
                                return Ok(Value::String(body));
                            }
                            Err(e) => return Err(format!("HTTP: {}", e)),
                        }
                    }
                    "http_post" => {
                        let url = av.get(0).map(|v| v.as_str()).unwrap_or_default();
                        let data = av.get(1).map(|v| v.as_str()).unwrap_or_default();
                        match ureq::post(&url).send_string(&data) {
                            Ok(resp) => {
                                let mut body = String::new();
                                resp.into_reader().read_to_string(&mut body).ok();
                                return Ok(Value::String(body));
                            }
                            Err(e) => return Err(format!("HTTP: {}", e)),
                        }
                    }
                    "http_post_json" => {
                        let url = av.get(0).map(|v| v.as_str()).unwrap_or_default();
                        let json = av.get(1).map(|v| v.as_str()).unwrap_or_default();
                        match ureq::post(&url).set("Content-Type", "application/json").send_string(&json) {
                            Ok(resp) => {
                                let mut body = String::new();
                                resp.into_reader().read_to_string(&mut body).ok();
                                return Ok(Value::String(body));
                            }
                            Err(e) => return Err(format!("HTTP: {}", e)),
                        }
                    }
                    "http_download" => {
                        let url = av.get(0).map(|v| v.as_str()).unwrap_or_default();
                        let path = av.get(1).map(|v| v.as_str()).unwrap_or_default();
                        match ureq::get(&url).call() {
                            Ok(resp) => {
                                let mut file = std::fs::File::create(&path).map_err(|e| format!("File: {}", e))?;
                                let mut reader = resp.into_reader();
                                std::io::copy(&mut reader, &mut file).map_err(|e| format!("Download: {}", e))?;
                                return Ok(Value::Bool(true));
                            }
                            Err(e) => return Err(format!("HTTP: {}", e)),
                        }
                    }
                    _ => {}
                }
                
                if name.contains('.') { let p: Vec<&str> = name.splitn(2, '.').collect(); if p.len() == 2 { if let Some(m) = self.modules.get(p[0]) { if let Some(f) = m.get(p[1]).cloned() { return self.call_fn(&f, &av); } } } return Err(format!("Unknown: {}", name)); }
                if let Some(f) = self.functions.get(name).cloned() { self.call_fn(&f, &av) } else { Err(format!("Unknown: {}", name)) }
            }
            _ => Err("Not implemented".into()),
        }
    }
}
