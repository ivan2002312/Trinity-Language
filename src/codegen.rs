// src/codegen.rs
use crate::ast::Module;
use std::path::Path;

pub fn compile_to_c(module: &Module, output: &Path) -> Result<(), String> {
    let mut c_code = String::new();
    
    // Заголовки
    c_code.push_str("#include <stdio.h>\n");
    c_code.push_str("#include <stdlib.h>\n\n");
    
    // Генерируем функции
    for decl in &module.declarations {
        if let crate::ast::Declaration::Function(func) = decl {
            c_code.push_str(&format!("int {}() {{\n", func.name));
            c_code.push_str("    // TODO: generate function body\n");
            c_code.push_str("    return 0;\n");
            c_code.push_str("}\n\n");
        }
    }
    
    // Главная функция
    c_code.push_str("int main() {\n");
    c_code.push_str("    printf(\"Trinity program\\n\");\n");
    c_code.push_str("    return 0;\n");
    c_code.push_str("}\n");
    
    std::fs::write(output.with_extension("c"), c_code)
        .map_err(|e| e.to_string())?;
    
    Ok(())
}