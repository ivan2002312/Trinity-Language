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
        eprintln!("  trinity <file.tr> --run       Run program");
        eprintln!("  trinity <file.tr> --build     Compile to exe");
        eprintln!("  trinity trpip <cmd>          Package manager");
        return;
    }

    let cmd = &args[1];
    
    if cmd == "trpip" {
        run_trpip(&args[2..]);
        return;
    }
    
    let input = PathBuf::from(cmd);
    let build = args.iter().any(|a| a == "--build");
    let run = args.iter().any(|a| a == "--run") || !build;
    let verbose = args.iter().any(|a| a == "--verbose");

    let source = fs::read_to_string(&input).unwrap_or_else(|e| { eprintln!("Error: {}", e); std::process::exit(1); });
    let base_dir = input.parent().unwrap_or(Path::new("."));
    let tokens = lexer::tokenize(&source).unwrap_or_else(|e| { for err in &e { eprintln!("{}", err); } std::process::exit(1); });
    let module = parser::parse(tokens, &source).unwrap_or_else(|e| { for err in &e { eprintln!("{}", err); } std::process::exit(1); });

    if build {
        let install_dir = exe_dir().join("modules");
        let repo = "https://raw.githubusercontent.com/ivan2002312/Trinity-Module/main";
        compile_to_exe(&module, &source, &input, &base_dir, &install_dir, repo);
        return;
    }

    if run {
        let mut interp = interpreter::Interpreter::new();
        let install_dir = exe_dir().join("modules");
        let repo = "https://raw.githubusercontent.com/ivan2002312/Trinity-Module/main";
        load_modules(&mut interp, &module, base_dir, &install_dir, repo, verbose);
        match interp.execute_module(&module) {
            Ok(val) => { if !matches!(val, interpreter::Value::Void) { println!("{}", val); } }
            Err(e) => eprintln!("Error: {}", e),
        }
    }
}

fn compile_to_exe(module: &ast::Module, _source: &str, input: &Path, base_dir: &Path, install_dir: &Path, repo: &str) {
    let name = input.file_stem().unwrap().to_string_lossy().to_string();
    let out_dir = PathBuf::from("trinity_build");
    fs::create_dir_all(&out_dir).ok();
    
    println!("Compiling {} to exe...", name);
    
    // Collect all module sources
    let mut module_sources: HashMap<String, String> = HashMap::new();
    for import_path in &module.imports {
        let src = load_module_source(import_path, base_dir, install_dir, repo);
        if let Some(s) = src {
            module_sources.insert(import_path.clone(), s);
            println!("  Loaded module: {}", import_path);
        }
    }
    
    let rust_code = generate_rust_code_full(module, &module_sources);
    let rust_file = out_dir.join(format!("{}.rs", name));
    fs::write(&rust_file, &rust_code).unwrap();
    
    let cargo_toml = format!(r#"
[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "{}"
path = "{}.rs"
"#, name, name, name);
    fs::write(out_dir.join("Cargo.toml"), &cargo_toml).ok();
    
    println!("Building with cargo...");
    let status = Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(&out_dir)
        .status();
    
    if status.map_or(false, |s| s.success()) {
        let exe = out_dir.join(format!("target/release/{}.exe", name));
        let final_exe = PathBuf::from(format!("{}.exe", name));
        fs::copy(&exe, &final_exe).ok();
        println!("Done: {}.exe", name);
        fs::remove_dir_all(&out_dir).ok();
    } else {
        eprintln!("Build failed. Is Rust installed?");
    }
}

fn load_module_source(import_path: &str, base_dir: &Path, install_dir: &Path, repo: &str) -> Option<String> {
    let candidates = vec![
        base_dir.join(format!("{}.trm", import_path)),
        install_dir.join(format!("{}/{}.trm", import_path, import_path)),
    ];
    
    for c in &candidates {
        if c.exists() {
            return fs::read_to_string(c).ok();
        }
    }
    
    // Download
    let url = format!("{}/{}/{}.trm", repo, import_path, import_path);
    if let Ok(resp) = ureq::get(&url).call() {
        let mut body = String::new();
        use std::io::Read;
        if resp.into_reader().read_to_string(&mut body).is_ok() {
            let dest = install_dir.join(import_path).join(format!("{}.trm", import_path));
            fs::create_dir_all(dest.parent().unwrap()).ok();
            fs::write(&dest, &body).ok();
            return Some(body);
        }
    }
    None
}

fn generate_rust_code_full(main_module: &ast::Module, module_sources: &HashMap<String, String>) -> String {
    let mut code = String::from("use std::io::Read;\nfn read_line() -> String { let mut s = String::new(); std::io::stdin().read_line(&mut s).ok(); s.trim().to_string() }\n\nfn main() {\n");
    
    // Parse and include module functions
    let mut module_functions: HashMap<String, Vec<String>> = HashMap::new();
    
    for (mod_name, source) in module_sources {
        if let Ok(tokens) = lexer::tokenize(source) {
            if let Ok(module) = parser::parse(tokens, source) {
                let mut func_names = Vec::new();
                for decl in &module.declarations {
                    if let ast::Declaration::Class(c) = decl {
                        for method in &c.methods {
                            let rust_fn = compile_function_rust(method, Some(mod_name));
                            code.push_str(&rust_fn);
                            func_names.push(method.name.clone());
                        }
                    }
                }
                module_functions.insert(mod_name.clone(), func_names);
            }
        }
    }
    
    // Generate main body
    for decl in &main_module.declarations {
        if let ast::Declaration::Class(c) = decl {
            for method in &c.methods {
                if method.name == "main" {
                    if let Some(body) = &method.body {
                        for stmt in body {
                            code.push_str(&compile_statement_full(stmt, &module_functions));
                        }
                    }
                }
            }
        }
    }
    
    code.push_str("}\n");
    code
}

fn compile_function_rust(func: &ast::FunctionDecl, module: Option<&str>) -> String {
    let fn_name = if let Some(mod_name) = module {
        format!("{}_{}", mod_name, func.name)
    } else {
        func.name.clone()
    };
    
    let params: Vec<String> = func.params.iter().map(|p| format!("{}: i32", p.name)).collect();
    let return_type = "i32";
    
    let body_code = if let Some(body) = &func.body {
        let mut b = String::new();
        for stmt in body {
            b.push_str(&compile_statement_simple(stmt));
        }
        b
    } else {
        format!("    0\n")
    };
    
    format!("fn {}({}) -> {} {{\n{}}}\n", fn_name, params.join(", "), return_type, body_code)
}

fn compile_statement_full(stmt: &ast::Statement, modules: &HashMap<String, Vec<String>>) -> String {
    match stmt {
        ast::Statement::Variable(name, _, Some(expr), _) => {
            format!("    let mut {} = {};\n", name, compile_expr_full(expr, modules))
        }
        ast::Statement::Return(Some(expr), _) => {
            format!("    println!(\"{{}}\", {});\n", compile_expr_full(expr, modules))
        }
        ast::Statement::Expression(expr, _) => {
            format!("    {};\n", compile_expr_full(expr, modules))
        }
        ast::Statement::If(cond, then, else_, _) => {
            let c = compile_expr_full(cond, modules);
            let t = compile_statement_full(then, modules);
            let e = if let Some(els) = else_ {
                format!(" else {{\n{}\n    }}", compile_statement_full(els, modules))
            } else { String::new() };
            format!("    if {} {{\n{}\n    }}{}\n", c, t, e)
        }
        ast::Statement::While(cond, body, _) => {
            format!("    while {} {{\n{}\n    }}\n", compile_expr_full(cond, modules), compile_statement_full(body, modules))
        }
        ast::Statement::For(init, cond, inc, body, _) => {
            let i = compile_statement_full(init, modules).trim().to_string();
            let c = cond.as_ref().map(|e| compile_expr_full(e, modules)).unwrap_or("true".to_string());
            let inc_str = inc.as_ref().map(|e| compile_expr_full(e, modules)).unwrap_or_default().trim().to_string();
            format!("    {};\n    while {} {{\n{}\n        {};\n    }}\n", i, c, compile_statement_full(body, modules), inc_str)
        }
        _ => String::new(),
    }
}

fn compile_statement_simple(stmt: &ast::Statement) -> String {
    compile_statement_full(stmt, &HashMap::new())
}

fn compile_expr_full(expr: &ast::Expression, modules: &HashMap<String, Vec<String>>) -> String {
    match expr {
        ast::Expression::Literal(lit, _) => match lit {
            ast::Literal::Int(n) => n.to_string(),
            ast::Literal::String(s) => format!("\"{}\"", s),
            ast::Literal::Bool(b) => format!("{}", b),
            _ => "0".to_string(),
        },
        ast::Expression::Identifier(name, _) => name.clone(),
        ast::Expression::Binary(left, op, right, _) => {
            let l = compile_expr_full(left, modules);
            let r = compile_expr_full(right, modules);
            let op_str = match op {
                ast::BinOp::Add => "+", ast::BinOp::Sub => "-",
                ast::BinOp::Mul => "*", ast::BinOp::Div => "/",
                ast::BinOp::Eq => "==", ast::BinOp::Lt => "<",
                ast::BinOp::Gt => ">", ast::BinOp::Le => "<=",
                ast::BinOp::Ge => ">=",
                _ => "+",
            };
            format!("({} {} {})", l, op_str, r)
        }
        ast::Expression::Call(name, args, _) => {
            // Module function call
            if name.contains('.') {
                let parts: Vec<&str> = name.splitn(2, '.').collect();
                if parts.len() == 2 {
                    let rust_name = format!("{}_{}", parts[0], parts[1]);
                    let args_str: Vec<String> = args.iter().map(|a| compile_expr_full(a, modules)).collect();
                    return format!("{}({})", rust_name, args_str.join(", "));
                }
            }
            // Built-in
            if name == "print" || name == "println" {
                let args_str: Vec<String> = args.iter().map(|a| compile_expr_full(a, modules)).collect();
                if args_str.len() == 1 {
                    format!("{}!(\"{{}}\", {})", name, args_str[0])
                } else {
                    format!("println!(\"{{}} {{}}\", {}, {})", args_str[0], args_str[1])
                }
            } else {
                format!("{}()", name)
            }
        }
        _ => "0".to_string(),
    }
}

// --- trpip functions -----------------------------

fn run_trpip(args: &[String]) {
    let subcmd = args.get(0).map(|s| s.as_str()).unwrap_or("help");
    match subcmd {
        "install" => { if let Some(n) = args.get(1) { install_module(n); } }
        "search" => search_modules(args.get(1).map(|s| s.as_str()).unwrap_or("")),
        "list" => list_modules(),
        _ => eprintln!("trpip: install, search, list"),
    }
}

fn load_modules(interp: &mut interpreter::Interpreter, module: &ast::Module, base_dir: &Path, install_dir: &Path, repo: &str, verbose: bool) {
    for import_path in &module.imports {
        let mut loaded = false;
        let candidates = vec![
            base_dir.join(format!("{}.trm", import_path)),
            install_dir.join(format!("{}/{}.trm", import_path, import_path)),
        ];
        for c in &candidates {
            if c.exists() {
                if let Ok(src) = fs::read_to_string(c) {
                    if let Ok(m) = parser::parse(lexer::tokenize(&src).unwrap_or_default(), &src) {
                        let mut funcs = HashMap::new();
                        for decl in &m.declarations {
                            if let ast::Declaration::Function(f) = decl { funcs.insert(f.name.clone(), f.clone()); }
                            if let ast::Declaration::Class(c) = decl { for m in &c.methods { funcs.insert(m.name.clone(), m.clone()); } }
                        }
                        interp.register_module(import_path.clone(), funcs);
                        loaded = true; break;
                    }
                }
            }
        }
        if !loaded {
            let url = format!("{}/{}/{}.trm", repo, import_path, import_path);
            if let Ok(resp) = ureq::get(&url).call() {
                let mut body = String::new();
                use std::io::Read;
                if resp.into_reader().read_to_string(&mut body).is_ok() {
                    let dest = install_dir.join(import_path).join(format!("{}.trm", import_path));
                    fs::create_dir_all(dest.parent().unwrap()).ok();
                    fs::write(&dest, &body).ok();
                }
            }
        }
    }
}

fn install_module(name: &str) {
    let dir = exe_dir().join("modules");
    let repo = "https://raw.githubusercontent.com/ivan2002312/Trinity-Module/main";
    let url = format!("{}/{}/{}.trm", repo, name, name);
    println!("Installing {}...", name);
    if let Ok(resp) = ureq::get(&url).call() {
        let mut body = String::new();
        use std::io::Read;
        if resp.into_reader().read_to_string(&mut body).is_ok() {
            let dest = dir.join(name).join(format!("{}.trm", name));
            fs::create_dir_all(dest.parent().unwrap()).ok();
            fs::write(&dest, &body).ok();
            println!("Done: {}", name);
        }
    }
}

fn search_modules(query: &str) {
    println!("Searching...");
    let url = "https://api.github.com/repos/ivan2002312/Trinity-Module/contents";
    if let Ok(resp) = ureq::get(url).set("User-Agent", "Trinity").call() {
        let mut body = String::new();
        use std::io::Read;
        if resp.into_reader().read_to_string(&mut body).is_ok() {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
                if let Some(items) = json.as_array() {
                    for item in items {
                        let name = item["name"].as_str().unwrap_or("");
                        if item["type"].as_str().unwrap_or("") == "dir" && name != ".git" {
                            if query.is_empty() || name.contains(query) {
                                println!("  {}", name);
                            }
                        }
                    }
                }
            }
        }
    }
}

fn list_modules() {
    let dir = exe_dir().join("modules");
    println!("Modules: {}", dir.display());
    if let Ok(e) = fs::read_dir(&dir) {
        for entry in e.flatten() {
            println!("  {}", entry.file_name().to_string_lossy());
        }
    }
}

fn exe_dir() -> PathBuf {
    std::env::current_exe().unwrap_or(PathBuf::from(".")).parent().unwrap_or(Path::new(".")).to_path_buf()
}
