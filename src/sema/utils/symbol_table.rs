use std::collections::HashMap;
use crate::frontend::utils::ast::*;
use crate::frontend::utils::token::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolKind {
    Variable,
    Function,
    Struct,
    Enum,
    Trait,
    Type,
    Parameter,
    Field,
    Module,
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub ty: Option<Type>,
    pub span: Option<Span>,
    pub struct_fields: Option<Vec<Field>>, // For struct fields
    pub enum_variants: Option<Vec<EnumVariant>>, // For enum variants
}

#[derive(Debug, Default, Clone)]
pub struct Scope {
    pub symbols: HashMap<String, Symbol>,
}

impl Scope {
    pub fn new() -> Self {
        Scope { symbols: HashMap::new() }
    }
    pub fn insert(&mut self, symbol: Symbol) {
        self.symbols.insert(symbol.name.clone(), symbol);
    }
    pub fn get(&self, name: &str) -> Option<&Symbol> {
        self.symbols.get(name)
    }
}

#[derive(Debug, Default)]
pub struct SymbolTable {
    pub scopes: Vec<Scope>,
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable { scopes: vec![Scope::new()] }
    }
    pub fn enter_scope(&mut self) {
        self.scopes.push(Scope::new());
    }
    pub fn exit_scope(&mut self) {
        self.scopes.pop();
    }
    pub fn insert(&mut self, symbol: Symbol) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(symbol);
        }
    }
    pub fn get(&self, name: &str) -> Option<&Symbol> {
        for scope in self.scopes.iter().rev() {
            if let Some(sym) = scope.get(name) {
                return Some(sym);
            }
        }
        None
    }
    pub fn current_scope(&self) -> Option<&Scope> {
        self.scopes.last()
    }
}