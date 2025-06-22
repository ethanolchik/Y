use crate::frontend::utils::ast::*;
use crate::frontend::utils::visitor::Visitor;
use crate::sema::utils::MultiStageSymbolTable;
use std::collections::HashMap;
use crate::frontend::utils::token::{Token, Span, TokenKind};
use crate::sema::utils::symbol_table::{Symbol, SymbolKind};

#[derive(Debug)]
pub struct TypeChecker {
    pub table: MultiStageSymbolTable,
    pub errors: Vec<String>,
    pub current_return_type: Option<Type>,
    pub type_vars: HashMap<String, Type>,
}

impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker {
            table: MultiStageSymbolTable::new(),
            errors: Vec::new(),
            current_return_type: None,
            type_vars: HashMap::new(),
        }
    }

    fn error(&mut self, message: String, span: &Span) {
        self.errors.push(format!("Type error at {:?}: {}", span, message));
    }

    fn check_type_compatibility(&self, expected: &Type, actual: &Type) -> bool {
        match (expected, actual) {
            (Type::Primitive { name: n1, .. }, Type::Primitive { name: n2, .. }) => n1.lexeme == n2.lexeme,
            (Type::Named { name: n1, generics: g1, .. }, Type::Named { name: n2, generics: g2, .. }) => {
                n1.lexeme == n2.lexeme && g1.len() == g2.len() && 
                g1.iter().zip(g2.iter()).all(|(t1, t2)| self.check_type_compatibility(t1, t2))
            }
            (Type::Array { element: e1, size: s1, .. }, Type::Array { element: e2, size: s2, .. }) => {
                s1 == s2 && self.check_type_compatibility(e1, e2)
            }
            (Type::Tuple { elements: e1, .. }, Type::Tuple { elements: e2, .. }) => {
                e1.len() == e2.len() && 
                e1.iter().zip(e2.iter()).all(|(t1, t2)| self.check_type_compatibility(t1, t2))
            }
            (Type::Function { params: p1, return_type: r1, .. }, Type::Function { params: p2, return_type: r2, .. }) => {
                p1.len() == p2.len() && 
                p1.iter().zip(p2.iter()).all(|(t1, t2)| self.check_type_compatibility(t1, t2)) &&
                self.check_type_compatibility(r1, r2)
            }
            (Type::TypeVar { name: n1, .. }, Type::TypeVar { name: n2, .. }) => n1.lexeme == n2.lexeme,
            _ => false,
        }
    }

    fn infer_type(&mut self, expr: &Expr) -> Option<Type> {
        match expr {
            Expr::Identifier(token, _) => {
                if let Some(symbol) = self.table.values.get(&token.lexeme) {
                    symbol.ty.clone()
                } else {
                    self.error(format!("Undefined variable '{}'", token.lexeme), &token.span);
                    None
                }
            }
            Expr::Literal(lit) => match lit {
                Literal::Integer(_, span) => Some(Type::Primitive { 
                    name: Token { 
                        lexeme: "int".to_string(), 
                        span: span.clone(),
                        kind: TokenKind::Identifier,
                        line: 0,
                    },
                    span: span.clone()
                }),
                Literal::Float(_, span) => Some(Type::Primitive { 
                    name: Token { 
                        lexeme: "float".to_string(), 
                        span: span.clone(),
                        kind: TokenKind::Identifier,
                        line: 0,
                    },
                    span: span.clone()
                }),
                Literal::Bool(_, span) => Some(Type::Primitive { 
                    name: Token { 
                        lexeme: "bool".to_string(), 
                        span: span.clone(),
                        kind: TokenKind::Identifier,
                        line: 0,
                    },
                    span: span.clone()
                }),
                Literal::Null(span) => Some(Type::Primitive { 
                    name: Token { 
                        lexeme: "null".to_string(), 
                        span: span.clone(),
                        kind: TokenKind::Identifier,
                        line: 0,
                    },
                    span: span.clone()
                }),
                Literal::Token(_, span) => Some(Type::Primitive { 
                    name: Token { 
                        lexeme: "string".to_string(), 
                        span: span.clone(),
                        kind: TokenKind::Identifier,
                        line: 0,
                    },
                    span: span.clone()
                }),
            },
            Expr::Binary { left, op, right, span } => {
                let left_ty = self.infer_type(left);
                let right_ty = self.infer_type(right);
                
                match (&left_ty, &right_ty) {
                    (Some(Type::Primitive { name: n1, .. }), Some(Type::Primitive { name: n2, .. })) => {
                        match (n1.lexeme.as_str(), n2.lexeme.as_str(), op.lexeme.as_str()) {
                            ("int", "int", _) | ("float", "float", _) => left_ty,
                            ("int", "float", _) | ("float", "int", _) => Some(Type::Primitive {
                                name: Token { 
                                    lexeme: "float".to_string(), 
                                    span: span.clone(),
                                    kind: TokenKind::Identifier,
                                    line: 0,
                                },
                                span: span.clone()
                            }),
                            ("bool", "bool", "&&" | "||") => left_ty,
                            _ => {
                                self.error(format!("Invalid binary operation: {} {} {}", 
                                    n1.lexeme, op.lexeme, n2.lexeme), span);
                                None
                            }
                        }
                    }
                    _ => {
                        self.error("Invalid operands for binary operation".to_string(), span);
                        None
                    }
                }
            }
            Expr::Call { callee, args, generic_args, span } => {
                let callee_ty = self.infer_type(callee);
                if let Some(Type::Function { params, return_type, .. }) = callee_ty {
                    if params.len() != args.len() {
                        self.error(format!("Expected {} arguments, got {}", params.len(), args.len()), span);
                        return None;
                    }
                    
                    for (param_ty, arg) in params.iter().zip(args.iter()) {
                        let arg_ty = self.infer_type(arg);
                        if let Some(arg_ty) = arg_ty {
                            if !self.check_type_compatibility(param_ty, &arg_ty) {
                                self.error(format!("Type mismatch in function call"), span);
                                return None;
                            }
                        }
                    }
                    
                    Some(*return_type)
                } else {
                    self.error("Expression is not callable".to_string(), span);
                    None
                }
            }
            // Add more expression type inference cases here
            _ => None,
        }
    }
}

impl Visitor for TypeChecker {
    fn visit_function(&mut self, function: &Function) -> Result<(), String> {
        let old_return_type = self.current_return_type.take();
        self.current_return_type = Some(function.return_type.clone());
        
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
        self.current_return_type = old_return_type;
        Ok(())
    }

    fn visit_statement(&mut self, statement: &Statement) -> Result<(), String> {
        match statement {
            Statement::Let { name, ty, value, span } => {
                if let Some(value) = value {
                    let value_ty = self.infer_type(value);
                    if let Some(value_ty) = value_ty {
                        if let Some(declared_ty) = ty {
                            if !self.check_type_compatibility(declared_ty, &value_ty) {
                                self.error(format!("Type mismatch in let binding"), span);
                            }
                        }
                    }
                }
            }
            Statement::Return(expr, span) => {
                if let Some(return_type) = &self.current_return_type {
                    if let Some(expr) = expr {
                        // Clone the return type to avoid the borrow checker issue
                        let return_type = return_type.clone();
                        let expr_ty = self.infer_type(expr);
                        if let Some(expr_ty) = expr_ty {
                            if !self.check_type_compatibility(&return_type, &expr_ty) {
                                self.error(format!("Return type mismatch"), span);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        Visitor::visit_statement(self, statement)
    }
} 