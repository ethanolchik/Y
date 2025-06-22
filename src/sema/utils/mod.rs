pub mod symbol_table;

use symbol_table::*;

/// MultiStageSymbolTable: supports staged population and lookup of symbols, variants, and fields.
#[derive(Debug, Default)]
pub struct MultiStageSymbolTable {
    pub types: SymbolTable, // For types (structs, enums, traits, etc.)
    pub values: SymbolTable, // For variables, functions, etc.
    pub enum_variants: SymbolTable, // For enum variants
    pub struct_fields: SymbolTable, // For struct fields
}

impl MultiStageSymbolTable {
    pub fn new() -> Self {
        MultiStageSymbolTable {
            types: SymbolTable::new(),
            values: SymbolTable::new(),
            enum_variants: SymbolTable::new(),
            struct_fields: SymbolTable::new(),
        }
    }
    pub fn has_enum_variant(&self, name: &str) -> bool {
        self.enum_variants.get(name).is_some()
    }
    pub fn has_struct_field(&self, name: &str) -> bool {
        self.struct_fields.get(name).is_some()
    }
    pub fn has_type(&self, name: &str) -> bool {
        self.types.get(name).is_some()
    }
    pub fn has_value(&self, name: &str) -> bool {
        self.values.get(name).is_some()
    }
}