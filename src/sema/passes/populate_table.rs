use crate::frontend::utils::ast::*;
use crate::frontend::utils::visitor::Visitor;
use crate::sema::utils::{
    MultiStageSymbolTable,
    symbol_table::{Symbol, SymbolKind},
};

/// Pass to populate all stages of the symbol table, including enum variants and struct fields.
pub struct FullSymbolTablePass {
    pub table: MultiStageSymbolTable,
}

impl FullSymbolTablePass {
    pub fn new() -> Self {
        FullSymbolTablePass { table: MultiStageSymbolTable::new() }
    }
}

impl Visitor for FullSymbolTablePass {
    fn visit_function(&mut self, function: &Function) -> Result<(), String> {
        self.table.values.insert(Symbol {
            name: function.name.lexeme.clone(),
            kind: SymbolKind::Function,
            ty: Some(function.return_type.clone()),
            span: Some(function.name.span.clone()),
            struct_fields: None,
            enum_variants: None,
        });
        self.table.values.enter_scope();
        for param in &function.params {
            self.table.values.insert(Symbol {
                name: param.name.lexeme.clone(),
                kind: SymbolKind::Parameter,
                ty: Some(param.ty.clone()),
                span: Some(param.name.span.clone()),
                struct_fields: None,
                enum_variants: None,
            });
        }
        Visitor::visit_function(self, function)?;
        self.table.values.exit_scope();
        Ok(())
    }

    fn visit_struct(&mut self, structure: &Struct) -> Result<(), String> {
        self.table.types.insert(Symbol {
            name: structure.name.lexeme.clone(),
            kind: SymbolKind::Struct,
            ty: None,
            span: Some(structure.name.span.clone()),
            struct_fields: Some(structure.fields.clone()),
            enum_variants: None,
        });

        Ok(())
    }

    fn visit_enum(&mut self, enumeration: &Enum) -> Result<(), String> {
        self.table.types.insert(Symbol {
            name: enumeration.name.lexeme.clone(),
            kind: SymbolKind::Enum,
            ty: None,
            span: Some(enumeration.name.span.clone()),
            struct_fields: None,
            enum_variants: Some(enumeration.variants.clone()),
        });
        Ok(())
    }

    fn visit_statement(&mut self, statement: &Statement) -> Result<(), String> {
        if let Statement::Let { name, ty, .. } = statement {
            self.table.values.insert(Symbol {
                name: name.lexeme.clone(),
                kind: SymbolKind::Variable,
                ty: ty.clone(),
                span: Some(name.span.clone()),
                struct_fields: None,
                enum_variants: None,
            });
        }

        // Don't call the base visitor method here, as we don't want to traverse into the statement body.
        Ok(())
    }
}
