mod ast;
mod error;
mod lexer;
mod parser;
mod interpreter;

use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Trinity Compiler v0.3");
        eprintln!("Usage: {} <file.tr> [--lex-only|--parse-only|--run]", args[0]);
        return;
    }

    let input = PathBuf::from(&args[1]);
    let lex_only = args.iter().any(|a| a == "--lex-only");
    let parse_only = args.iter().any(|a| a == "--parse-only");
    let run = args.iter().any(|a| a == "--run") || (!lex_only && !parse_only);

    let source = match std::fs::read_to_string(&input) {
        Ok(s) => s,
        Err(e) => { eprintln!("Error: {}", e); return; }
    };

    eprintln!("Lexing...");
    let tokens = match lexer::tokenize(&source) {
        Ok(t) => t,
        Err(e) => { for err in &e { eprintln!("{}", err); } return; }
    };
    if lex_only { println!("{} tokens:", tokens.len()); for (t,s) in &tokens { println!("  {:?} at {}", t, s.start); } return; }

    eprintln!("Parsing...");
    let module = match parser::parse(tokens, &source) {
        Ok(m) => m,
        Err(e) => { for err in &e { eprintln!("{}", err); } return; }
    };
    if parse_only { println!("{:#?}", module); return; }

    if run {
        eprintln!("Running...");
        let mut interp = interpreter::Interpreter::new();
        match interp.execute_module(&module) {
            Ok(val) => { if !matches!(val, interpreter::Value::Void) { println!("{}", val); } }
            Err(e) => eprintln!("Runtime error: {}", e),
        }
    }
}
