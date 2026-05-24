// src/semantic.rs
use crate::ast::*;
use std::collections::HashMap;

pub struct TypeChecker {
    scopes: Vec<HashMap<String, Type>>,
    functions: HashMap<String, FunctionDecl>,
    classes: HashMap<String, ClassDecl>,
}

impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker {
            scopes: vec![HashMap::new()],
            functions: HashMap::new(),
            classes: HashMap::new(),
        }
    }
    
    pub fn check_module(&mut self, module: &Module) -> Result<(), String> {
        println!("Type checking module: {}", module.name);
        
        // Регистрируем все объявления
        for decl in &module.declarations {
            match decl {
                Declaration::Function(func) => {
                    self.functions.insert(func.name.clone(), func.clone());
                }
                Declaration::Class(class) => {
                    self.classes.insert(class.name.clone(), class.clone());
                }
                _ => {}
            }
        }
        
        println!("  Functions: {:?}", self.functions.keys());
        println!("  Classes: {:?}", self.classes.keys());
        
        Ok(())
    }
}