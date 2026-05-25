use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;

mod ast;
mod error;
mod lexer;
mod parser;
mod interpreter;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Trinity v1.0");
        eprintln!("Usage: trinity <file.tr> [--run|--verbose]");
        return;
    }

    let input = PathBuf::from(&args[1]);
    let verbose = args.iter().any(|a| a == "--verbose");

    let source = match fs::read_to_string(&input) {
        Ok(s) => s,
        Err(e) => { eprintln!("Error: {}", e); return; }
    };

    let base_dir = input.parent().unwrap_or(Path::new("."));

    let tokens = lexer::tokenize(&source).unwrap_or_else(|e| {
        for err in &e { eprintln!("{}", err); }
        std::process::exit(1);
    });

    let main_module = parser::parse(tokens, &source).unwrap_or_else(|e| {
        for err in &e { eprintln!("{}", err); }
        std::process::exit(1);
    });

    let mut interp = interpreter::Interpreter::new();
    
    for import_path in &main_module.imports {
        let candidates = vec![
            base_dir.join(format!("{}.trm", import_path)),
            base_dir.join(format!("{}.tr", import_path)),
        ];
        for c in &candidates {
            if c.exists() {
                if let Ok(src) = fs::read_to_string(&c) {
                    let t = lexer::tokenize(&src).unwrap_or_default();
                    if let Ok(module) = parser::parse(t, &src) {
                        let mut funcs = HashMap::new();
                        for decl in &module.declarations {
                            if let ast::Declaration::Function(f) = decl { funcs.insert(f.name.clone(), f.clone()); }
                            if let ast::Declaration::Class(c) = decl { for m in &c.methods { funcs.insert(m.name.clone(), m.clone()); } }
                        }
                        let count = funcs.len();
                        interp.register_module(import_path.clone(), funcs);
                        if verbose { eprintln!("Loaded {} ({} funcs)", import_path, count); }
                    }
                }
            }
        }
    }

    match interp.execute_module(&main_module) {
        Ok(val) => { if !matches!(val, interpreter::Value::Void) { println!("{}", val); } }
        Err(e) => eprintln!("Error: {}", e),
    }
}
