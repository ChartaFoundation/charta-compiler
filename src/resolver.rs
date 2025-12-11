use crate::ast;
use crate::error::{CompileError, Result};
use std::collections::HashMap;

/// Symbol table for name resolution
pub struct SymbolTable {
    signals: HashMap<String, ast::SignalDecl>,
    coils: HashMap<String, ast::CoilDecl>,
    blocks: HashMap<String, ast::BlockDecl>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            signals: HashMap::new(),
            coils: HashMap::new(),
            blocks: HashMap::new(),
        }
    }
    
    pub fn add_signal(&mut self, signal: ast::SignalDecl) -> Result<()> {
        if self.signals.contains_key(&signal.name) {
            return Err(CompileError::NameResolution(
                format!("Duplicate signal name: {}", signal.name)
            ));
        }
        self.signals.insert(signal.name.clone(), signal);
        Ok(())
    }
    
    pub fn add_coil(&mut self, coil: ast::CoilDecl) -> Result<()> {
        if self.coils.contains_key(&coil.name) {
            return Err(CompileError::NameResolution(
                format!("Duplicate coil name: {}", coil.name)
            ));
        }
        self.coils.insert(coil.name.clone(), coil);
        Ok(())
    }
    
    pub fn get_signal(&self, name: &str) -> Option<&ast::SignalDecl> {
        self.signals.get(name)
    }
    
    pub fn get_coil(&self, name: &str) -> Option<&ast::CoilDecl> {
        self.coils.get(name)
    }
    
    pub fn resolve_signal(&self, name: &str) -> Result<()> {
        if !self.signals.contains_key(name) {
            return Err(CompileError::NameResolution(
                format!("Undefined signal: {}", name)
            ));
        }
        Ok(())
    }
    
    pub fn resolve_coil(&self, name: &str) -> Result<()> {
        if !self.coils.contains_key(name) {
            return Err(CompileError::NameResolution(
                format!("Undefined coil: {}", name)
            ));
        }
        Ok(())
    }
}

/// Resolve all names in a module
pub fn resolve_names(module: &mut ast::Module) -> Result<()> {
    let mut symbols = SymbolTable::new();
    
    // First pass: collect all declarations
    for signal in &module.signals {
        symbols.add_signal(signal.clone())?;
    }
    
    for coil in &module.coils {
        symbols.add_coil(coil.clone())?;
    }
    
    // Second pass: resolve references in rungs
    for rung in &module.rungs {
        resolve_guard(&rung.guard, &symbols)?;
        for action in &rung.actions {
            symbols.resolve_coil(&action.coil)?;
        }
    }
    
    Ok(())
}

fn resolve_guard(guard: &ast::GuardExpr, symbols: &SymbolTable) -> Result<()> {
    match guard {
        ast::GuardExpr::Contact { name, .. } => {
            symbols.resolve_signal(name)?;
        }
        ast::GuardExpr::And { left, right } => {
            resolve_guard(left, symbols)?;
            resolve_guard(right, symbols)?;
        }
        ast::GuardExpr::Or { left, right } => {
            resolve_guard(left, symbols)?;
            resolve_guard(right, symbols)?;
        }
        ast::GuardExpr::Not { expr } => {
            resolve_guard(expr, symbols)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_names() {
        let mut module = ast::Module {
            name: "test".to_string(),
            context: None,
            intent: None,
            constraints: None,
            signals: vec![ast::SignalDecl {
                name: "input".to_string(),
                parameters: Vec::new(),
                type_: None,
            }],
            coils: vec![ast::CoilDecl {
                name: "output".to_string(),
                parameters: Vec::new(),
                latching: None,
                critical: None,
            }],
            rungs: vec![ast::RungDecl {
                name: "r1".to_string(),
                guard: ast::GuardExpr::Contact {
                    name: "input".to_string(),
                    contact_type: ast::ContactType::NO,
                    arguments: Vec::new(),
                },
                actions: vec![ast::Action {
                    action_type: ast::ActionType::Energise,
                    coil: "output".to_string(),
                    arguments: Vec::new(),
                }],
            }],
            blocks: Vec::new(),
            networks: Vec::new(),
        };
        
        assert!(resolve_names(&mut module).is_ok());
    }
}
