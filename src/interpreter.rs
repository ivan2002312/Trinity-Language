use crate::ast::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Value { Void, Bool(bool), Char(char), Int(i64), Float(f64), String(String), Array(Vec<Value>), Object(HashMap<String, Value>), Null }

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self { Value::Void => write!(f, "void"), Value::Bool(b) => write!(f, "{}", b), Value::Char(c) => write!(f, "{}", c), Value::Int(n) => write!(f, "{}", n), Value::Float(x) => write!(f, "{}", x), Value::String(s) => write!(f, "{}", s), Value::Array(a) => write!(f, "[{}]", a.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(", ")), Value::Null => write!(f, "null"), Value::Object(o) => write!(f, "{{{}}}", o.iter().map(|(k,v)| format!("{}: {}", k, v)).collect::<Vec<_>>().join(", ")), }
    }
}

impl Value {
    fn as_bool(&self) -> bool { match self { Value::Bool(b) => *b, Value::Int(n) => *n != 0, Value::Null => false, _ => true } }
    fn as_int(&self) -> i64 { match self { Value::Int(n) => *n, Value::Float(x) => *x as i64, _ => 0 } }
}

pub struct Interpreter {
    scopes: Vec<HashMap<String, Value>>,
    functions: HashMap<String, FunctionDecl>,
    return_val: Option<Value>,
    break_flag: bool,
    continue_flag: bool,
}

impl Interpreter {
    pub fn new() -> Self { Interpreter { scopes: vec![HashMap::new()], functions: HashMap::new(), return_val: None, break_flag: false, continue_flag: false } }

    fn push(&mut self) { self.scopes.push(HashMap::new()); }
    fn pop(&mut self) { self.scopes.pop(); }
    fn set(&mut self, k: &str, v: Value) { self.scopes.last_mut().unwrap().insert(k.to_string(), v); }
    fn get(&self, k: &str) -> Result<Value, String> { for s in self.scopes.iter().rev() { if let Some(v) = s.get(k) { return Ok(v.clone()); } } Err(format!("Undefined: {}", k)) }

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
        for s in stmts {
            if self.return_val.is_some() || self.break_flag || self.continue_flag { break; }
            self.exec_stmt(s)?;
        }
        Ok(())
    }

    fn exec_stmt(&mut self, s: &Statement) -> Result<(), String> {
        match s {
            Statement::Variable(n, _, init, _) => { let v = init.as_ref().map(|e| self.eval(e)).unwrap_or(Ok(Value::Null))?; self.set(n, v); }
            Statement::Expression(e, _) => { self.eval(e)?; }
            Statement::Block(stmts, _) => { self.push(); self.exec_block(stmts)?; self.pop(); }
            Statement::If(c, t, e, _) => { if self.eval(c)?.as_bool() { self.exec_stmt(t)?; } else if let Some(els) = e { self.exec_stmt(els)?; } }
            Statement::While(c, b, _) => { while self.eval(c)?.as_bool() { self.exec_stmt(b)?; if self.break_flag { self.break_flag = false; break; } if self.continue_flag { self.continue_flag = false; } } }
            Statement::For(i, c, inc, b, _) => { self.push(); self.exec_stmt(i)?; while c.as_ref().map_or(true, |x| self.eval(x).map_or(false, |v| v.as_bool())) { self.exec_stmt(b)?; if self.break_flag { self.break_flag = false; break; } if let Some(inc) = inc { self.eval(inc)?; } } self.pop(); }
            Statement::Return(e, _) => { self.return_val = Some(e.as_ref().map(|x| self.eval(x)).unwrap_or(Ok(Value::Void))?); }
            Statement::Break(_) => self.break_flag = true,
            Statement::Continue(_) => self.continue_flag = true,
            Statement::Switch(expr, cases, def, _) => {
                let v = self.eval(expr)?; let mut matched = false;
                for (c, b) in cases { if matched || format!("{}", self.eval(c)?) == format!("{}", v) { matched = true; self.exec_block(b)?; if self.break_flag { self.break_flag = false; break; } } }
                if !matched { if let Some(d) = def { self.exec_stmt(d)?; } }
            }
            Statement::Try(body, evar, cbody, fin, _) => {
                if let Err(e) = self.exec_block(body) { if let Some(var) = evar { self.push(); self.set(var, Value::String(e)); self.exec_block(cbody)?; self.pop(); } }
                if let Some(f) = fin { self.exec_block(f)?; }
            }
            Statement::Throw(e, _) => return Err(format!("{}", self.eval(e)?)),
            _ => {}
        }
        Ok(())
    }

    pub fn eval(&mut self, e: &Expression) -> Result<Value, String> {
        match e {
            Expression::Literal(l, _) => Ok(match l { Literal::Int(n) => Value::Int(*n), Literal::Float(f) => Value::Float(*f), Literal::String(s) => Value::String(s.clone()), Literal::Char(c) => Value::Char(*c), Literal::Bool(b) => Value::Bool(*b), Literal::Null => Value::Null }),
            Expression::Identifier(n, _) => self.get(n),
            Expression::Binary(l, op, r, _) => {
                let lv = self.eval(l)?;
                if *op == BinOp::And && !lv.as_bool() { return Ok(Value::Bool(false)); }
                if *op == BinOp::Or && lv.as_bool() { return Ok(Value::Bool(true)); }
                let rv = self.eval(r)?;
                match (&lv, &rv) {
                    (Value::Int(a), Value::Int(b)) => Ok(Value::Int(match op { BinOp::Add => a+b, BinOp::Sub => a-b, BinOp::Mul => a*b, BinOp::Div => a/b, BinOp::Mod => a%b, BinOp::Eq => return Ok(Value::Bool(a==b)), BinOp::Neq => return Ok(Value::Bool(a!=b)), BinOp::Lt => return Ok(Value::Bool(a<b)), BinOp::Gt => return Ok(Value::Bool(a>b)), BinOp::Le => return Ok(Value::Bool(a<=b)), BinOp::Ge => return Ok(Value::Bool(a>=b)), _ => 0 })),
                    (Value::Float(a), Value::Float(b)) => Ok(Value::Float(match op { BinOp::Add => a+b, BinOp::Sub => a-b, BinOp::Mul => a*b, BinOp::Div => a/b, _ => 0.0 })),
                    (Value::String(a), Value::String(b)) if *op == BinOp::Add => Ok(Value::String(format!("{}{}", a, b))),
                    _ => Err(format!("Cannot {:?} {:?} and {:?}", op, lv, rv)),
                }
            }
            Expression::Unary(op, e, _) => { let v = self.eval(e)?; match op { UnaryOp::Neg => Ok(Value::Int(-v.as_int())), UnaryOp::Not => Ok(Value::Bool(!v.as_bool())), _ => Err("Unsupported unary".into()) } }
            Expression::Ternary(c, t, f, _) => { if self.eval(c)?.as_bool() { self.eval(t) } else { self.eval(f) } }
            Expression::Call(name, args, _) => {
                let mut av = vec![]; for a in args { av.push(self.eval(a)?); }
                match name.as_str() {
                    "print" | "println" => { for a in &av { print!("{}", a); } if name == "println" { println!(); } Ok(Value::Void) }
                    "read_line" => { let mut s = String::new(); std::io::stdin().read_line(&mut s).ok(); Ok(Value::String(s.trim().to_string())) }
                    _ => if let Some(f) = self.functions.get(name).cloned() { self.call_fn(&f, &av) } else { Err(format!("Unknown function: {}", name)) }
                }
            }
            Expression::MemberAccess(obj, m, _) => {
                let v = self.eval(obj)?;
                match v { Value::String(s) if m == "length" || m == "len" => Ok(Value::Int(s.len() as i64)), Value::Array(a) if m == "length" || m == "len" => Ok(Value::Int(a.len() as i64)), _ => Err(format!("No member {}", m)) }
            }
            Expression::Index(obj, idx, _) => {
                let v = self.eval(obj)?; let i = self.eval(idx)?.as_int() as usize;
                match v { Value::Array(a) => a.get(i).cloned().ok_or("Index OOB".into()), Value::String(s) => s.chars().nth(i).map(Value::Char).ok_or("Index OOB".into()), _ => Err("Cannot index".into()) }
            }
            Expression::NewArray(_, sizes, _) => { let total = sizes.iter().map(|s| self.eval(s).map(|v| v.as_int() as usize)).sum::<Result<usize, _>>()?; Ok(Value::Array(vec![Value::Null; total])) }
            Expression::ArrayLiteral(items, _) => { let mut a = vec![]; for i in items { a.push(self.eval(i)?); } Ok(Value::Array(a)) }
            Expression::ObjectLiteral(fields, _) => { let mut m = HashMap::new(); for (k, v) in fields { m.insert(k.clone(), self.eval(v)?); } Ok(Value::Object(m)) }
            Expression::Range(s, e, _) => {
                let start = s.as_ref().map(|x| self.eval(x).map(|v| v.as_int())).unwrap_or(Ok(0))?;
                let end = e.as_ref().map(|x| self.eval(x).map(|v| v.as_int())).unwrap_or(Ok(start))?;
                Ok(Value::Array((start..end).map(Value::Int).collect()))
            }
            _ => Err(format!("Not implemented: {:?}", e)),
        }
    }
}
