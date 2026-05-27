use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;
use std::process::Command;

mod ast;
mod error;
mod lexer;
mod parser;
mod interpreter;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Trinity v2.0");
        eprintln!("  trinity <file.tr> --run     Run (full features)");
        eprintln!("  trinity <file.tr> --build   Compile to standalone exe");
        eprintln!("  trinity trpip <cmd>        Package manager");
        return;
    }
    let cmd = &args[1];
    if cmd == "trpip" { run_trpip(&args[2..]); return; }
    let input = PathBuf::from(cmd);
    let build = args.iter().any(|a| a == "--build");
    let source = fs::read_to_string(&input).unwrap_or_else(|e| { eprintln!("Error: {}", e); std::process::exit(1); });
    let base_dir = input.parent().unwrap_or(Path::new("."));
    let tokens = lexer::tokenize(&source).unwrap_or_else(|e| { for err in &e { eprintln!("{}", err); } std::process::exit(1); });
    let module = parser::parse(tokens, &source).unwrap_or_else(|e| { for err in &e { eprintln!("{}", err); } std::process::exit(1); });
    if build { compile_to_exe(&module, &source, &input, base_dir); return; }
    let mut interp = interpreter::Interpreter::new();
    load_modules(&mut interp, &module, base_dir);
    match interp.execute_module(&module) { Ok(v) => { if !matches!(v, interpreter::Value::Void) { println!("{}", v); } } Err(e) => eprintln!("Error: {}", e) }
}

fn compile_to_exe(module: &ast::Module, source: &str, input: &Path, base_dir: &Path) {
    let name = input.file_stem().unwrap().to_string_lossy().to_string();
    let out_dir = PathBuf::from("trinity_build"); fs::create_dir_all(&out_dir).ok();
    println!("Compiling {} to standalone exe...", name);
    
    // Save .tr file
    let tr_file = out_dir.join(format!("{}.tr", name));
    fs::write(&tr_file, source).ok();
    
    // Copy module files
    for imp in &module.imports {
        for c in &[base_dir.join(format!("{}.trm",imp)), packages_dir().join(imp).join(format!("{}.trm",imp))] {
            if c.exists() { 
                let dest = out_dir.join(format!("{}.trm", imp));
                fs::copy(c, &dest).ok();
                println!("  Packed: {}", imp);
                break;
            }
        }
    }
    
    // Generate Rust wrapper that embeds Trinity interpreter
    let rust_code = format!(r##"
use std::process::Command;
use std::fs;
use std::io::Write;

fn main() {{
    // Extract embedded Trinity source
    let source = include_str!("{name}.tr");
    
    // Write to temp file
    let tmp = std::env::temp_dir().join("{name}.tr");
    fs::write(&tmp, source).ok();
    
    // Copy module files if present
    // (modules are compiled alongside the exe)
    
    // Find trinity executable
    let trinity = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("trinity.exe");
    
    // Run with interpreter
    let output = if trinity.exists() {{
        Command::new(&trinity).args([tmp.to_str().unwrap(), "--run"]).output()
    }} else {{
        Command::new("trinity").args([tmp.to_str().unwrap(), "--run"]).output()
    }};
    
    match output {{
        Ok(out) => {{
            print!("{{}}", String::from_utf8_lossy(&out.stdout));
            eprint!("{{}}", String::from_utf8_lossy(&out.stderr));
        }}
        Err(e) => println!("Error: {{}}. Install Trinity: cargo install trinity", e),
    }}
    
    fs::remove_file(&tmp).ok(); println!();  let _ = std::io::stdin().read_line(&mut String::new());
    println!();
    
    
}}
"##, name = name);
    
    fs::write(out_dir.join(format!("{}.rs", name)), &rust_code).ok();
    fs::write(out_dir.join("Cargo.toml"), format!("[package]\nname = \"{}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[[bin]]\nname = \"{}\"\npath = \"{}.rs\"", name, name, name)).ok();
    
    println!("Building (requires Rust)...");
    let s = Command::new("cargo").args(["build","--release"]).current_dir(&out_dir).status();
    if s.map_or(false, |s| s.success()) {
        fs::copy(out_dir.join("target/release").join(format!("{}.exe",name)), format!("{}.exe",name)).ok();
        println!("Done: {}.exe", name);
        fs::remove_dir_all(&out_dir).ok();
    } else { eprintln!("Build failed. Install Rust: https://rustup.rs"); }
}

fn load_modules(interp: &mut interpreter::Interpreter, module: &ast::Module, base_dir: &Path) {
    let dir = packages_dir();
    for imp in &module.imports {
        let mut loaded = false;
        for c in &[base_dir.join(format!("{}.trm",imp)), dir.join(format!("{}/{}.trm",imp,imp))] {
            if c.exists() { if let Ok(s) = fs::read_to_string(c) { if let Ok(m) = parser::parse(lexer::tokenize(&s).unwrap_or_default(),&s) { let mut f = HashMap::new(); for d in &m.declarations { if let ast::Declaration::Function(x)=d { f.insert(x.name.clone(),x.clone()); } if let ast::Declaration::Class(x)=d { for m in &x.methods { f.insert(m.name.clone(),m.clone()); } } } interp.register_module(imp.clone(),f); loaded=true; break; } } }
        }
        if !loaded { eprintln!("Module '{}' not found", imp); }
    }
}

fn run_trpip(a: &[String]) { match a.get(0).map(|s|s.as_str()).unwrap_or("") { "install" => if let Some(n)=a.get(1) { install_module(n); } "search" => search_modules(a.get(1).map(|s|s.as_str()).unwrap_or("")), "list" => list_modules(), _ => eprintln!("trpip: install search list") } }
fn exe_dir() -> PathBuf { std::env::current_exe().unwrap_or(PathBuf::from(".")).parent().unwrap_or(Path::new(".")).to_path_buf() } fn packages_dir() -> PathBuf { exe_dir().join("packages") }
fn install_module(name: &str) {
    let dir = packages_dir().join(name);
    fs::create_dir_all(&dir).ok();
    let url = format!("https://raw.githubusercontent.com/ivan2002312/Trinity-Module/main/{}/{}.trm", name, name);
    println!("Downloading {}...", name);
    match ureq::get(&url).call() {
        Ok(resp) => {
            let mut body = String::new();
            use std::io::Read;
            if resp.into_reader().read_to_string(&mut body).is_ok() {
                fs::write(dir.join(format!("{}.trm", name)), &body).ok();
                println!("Installed: {}", name);
            }
        }
        Err(e) => println!("Failed: {}", e),
    }
}
fn search_modules(q: &str) {
    println!("Searching modules...");
    let url = "https://api.github.com/repos/ivan2002312/Trinity-Module/contents";
    match ureq::get(url).set("User-Agent", "Trinity").call() {
        Ok(resp) => {
            let mut body = String::new();
            use std::io::Read;
            if resp.into_reader().read_to_string(&mut body).is_ok() {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
                    if let Some(items) = json.as_array() {
                        for item in items {
                            let name = item["name"].as_str().unwrap_or("");
                            if item["type"].as_str().unwrap_or("") == "dir" && name != ".git" {
                                if q.is_empty() || name.contains(q) { println!("  {}", name); }
                            }
                        }
                    }
                }
            }
        }
        Err(e) => println!("  Cannot connect: {}", e),
    }
}
fn list_modules() { let d = packages_dir(); if d.exists() { for e in fs::read_dir(&d).unwrap().flatten() { println!("  {}", e.file_name().to_string_lossy()); } } }

