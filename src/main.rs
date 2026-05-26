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
        eprintln!("  trinity <file.tr> --run       Run program");
        eprintln!("  trinity trpip install <mod>  Install module");
        eprintln!("  trinity trpip search         Search modules");
        eprintln!("  trinity trpip list          List installed");
        return;
    }

    let cmd = &args[1];
    
    if cmd == "trpip" {
        let subcmd = args.get(2).map(|s| s.as_str()).unwrap_or("help");
        match subcmd {
            "install" => {
                if let Some(name) = args.get(3) { install_module(name); }
                else { eprintln!("Usage: trinity trpip install <name>"); }
            }
            "search" => search_modules(args.get(3).map(|s| s.as_str()).unwrap_or("")),
            "list" => list_modules(),
            "remove" => {
                if let Some(name) = args.get(3) { remove_module(name); }
            }
            _ => eprintln!("Commands: install, search, list, remove"),
        }
        return;
    }

    // Run mode
    let input = PathBuf::from(cmd);
    let verbose = args.iter().any(|a| a == "--verbose");
    let source = fs::read_to_string(&input).unwrap_or_else(|e| { eprintln!("Error: {}", e); std::process::exit(1); });
    let base_dir = input.parent().unwrap_or(Path::new("."));
    let tokens = lexer::tokenize(&source).unwrap_or_else(|e| { for err in &e { eprintln!("{}", err); } std::process::exit(1); });
    let main_module = parser::parse(tokens, &source).unwrap_or_else(|e| { for err in &e { eprintln!("{}", err); } std::process::exit(1); });

    let mut interp = interpreter::Interpreter::new();
    load_modules(&mut interp, &main_module, base_dir, verbose);

    match interp.execute_module(&main_module) {
        Ok(val) => { if !matches!(val, interpreter::Value::Void) { println!("{}", val); } }
        Err(e) => eprintln!("Error: {}", e),
    }
}

fn load_modules(interp: &mut interpreter::Interpreter, module: &ast::Module, base_dir: &Path, verbose: bool) {
    let repo_base = "https://raw.githubusercontent.com/ivan2002312/Trinity-Module/main";
    let install_dir = dirs_cache().join("modules");
    
    for import_path in &module.imports {
        let candidates = vec![
            base_dir.join(format!("{}.trm", import_path)),
            install_dir.join(format!("{}/{}.trm", import_path, import_path)),
        ];
        
        let mut loaded = false;
        for c in &candidates {
            if c.exists() {
                if let Ok(src) = fs::read_to_string(c) {
                    if let Ok(module) = parser::parse(lexer::tokenize(&src).unwrap_or_default(), &src) {
                        let mut funcs = HashMap::new();
                        for decl in &module.declarations {
                            if let ast::Declaration::Function(f) = decl { funcs.insert(f.name.clone(), f.clone()); }
                            if let ast::Declaration::Class(c) = decl { for m in &c.methods { funcs.insert(m.name.clone(), m.clone()); } }
                        }
                        interp.register_module(import_path.clone(), funcs);
                        if verbose { eprintln!("Loaded: {}", c.display()); }
                        loaded = true;
                        break;
                    }
                }
            }
        }
        
        if !loaded {
            eprintln!("Downloading {}...", import_path);
            if download_module(import_path, &install_dir, repo_base) {
                let c = install_dir.join(format!("{}/{}.trm", import_path, import_path));
                if c.exists() {
                    if let Ok(src) = fs::read_to_string(&c) {
                        if let Ok(module) = parser::parse(lexer::tokenize(&src).unwrap_or_default(), &src) {
                            let mut funcs = HashMap::new();
                            for decl in &module.declarations {
                                if let ast::Declaration::Function(f) = decl { funcs.insert(f.name.clone(), f.clone()); }
                                if let ast::Declaration::Class(c) = decl { for m in &c.methods { funcs.insert(m.name.clone(), m.clone()); } }
                            }
                            interp.register_module(import_path.clone(), funcs);
                            eprintln!("Loaded: {}", c.display());
                        }
                    }
                }
            }
        }
    }
}

fn download_module(name: &str, install_dir: &Path, repo_base: &str) -> bool {
    let url = format!("{}/{}/{}.trm", repo_base, name, name);
    let dest_dir = install_dir.join(name);
    fs::create_dir_all(&dest_dir).ok();
    let dest_file = dest_dir.join(format!("{}.trm", name));
    
    match ureq::get(&url).call() {
        Ok(resp) => {
            let mut body = String::new();
            if resp.into_reader().read_to_string(&mut body).is_ok() {
                fs::write(&dest_file, &body).ok();
                eprintln!("Downloaded: {}", name);
                return true;
            }
        }
        Err(e) => eprintln!("Download error: {}", e),
    }
    false
}

fn search_modules(query: &str) {
    println!("Searching modules...");
    println!("Repository: https://github.com/ivan2002312/Trinity-Module");
    println!();
    
    let url = "https://api.github.com/repos/ivan2002312/Trinity-Module/contents";
    
    match ureq::get(url).set("User-Agent", "Trinity").call() {
        Ok(resp) => {
            let mut body = String::new();
            if resp.into_reader().read_to_string(&mut body).is_ok() {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
                    if let Some(items) = json.as_array() {
                        for item in items {
                            let name = item["name"].as_str().unwrap_or("");
                            let item_type = item["type"].as_str().unwrap_or("");
                            if item_type == "dir" && name != ".git" && (query.is_empty() || name.contains(query)) {
                                println!("  {}", name);
                            }
                        }
                    }
                }
            }
        }
        Err(e) => println!("HTTP error: {}", e),
    }
}

fn install_module(name: &str) {
    let install_dir = dirs_cache().join("modules");
    let repo_base = "https://raw.githubusercontent.com/ivan2002312/Trinity-Module/main";
    
    println!("Installing {}...", name);
    if download_module(name, &install_dir, repo_base) {
        println!("Module '{}' installed!", name);
    } else {
        eprintln!("Failed to install '{}'", name);
    }
}

fn list_modules() {
    let install_dir = dirs_cache().join("modules");
    println!("Installed modules:");
    println!("Location: {}", install_dir.display());
    if let Ok(entries) = fs::read_dir(&install_dir) {
        for entry in entries.flatten() {
            println!("  {}", entry.file_name().to_string_lossy());
        }
    }
}

fn remove_module(name: &str) {
    let dir = dirs_cache().join("modules").join(name);
    if dir.exists() { fs::remove_dir_all(&dir).ok(); println!("Removed: {}", name); }
    else { eprintln!("Not found: {}", name); }
}

fn dirs_cache() -> PathBuf {
    let exe = std::env::current_exe().unwrap_or(PathBuf::from("."));
    exe.parent().unwrap_or(Path::new(".")).join("modules")
}
