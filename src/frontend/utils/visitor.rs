use super::ast::*;

pub trait Visitor {
    fn visit_module(&mut self, module: &Module) -> Result<(), String> {
        walk_module(self, module)
    }
    fn visit_function(&mut self, function: &Function) -> Result<(), String> {
        walk_function(self, function)
    }
    fn visit_struct(&mut self, structure: &Struct) -> Result<(), String> {
        walk_struct(self, structure)
    }
    fn visit_enum(&mut self, enumeration: &Enum) -> Result<(), String> {
        walk_enum(self, enumeration)
    }
    fn visit_extend(&mut self, extend: &Extend) -> Result<(), String> {
        walk_extend(self, extend)
    }
    fn visit_trait(&mut self, trait_: &Trait) -> Result<(), String> {
        walk_trait(self, trait_)
    }
    fn visit_import(&mut self, _import: &Import) -> Result<(), String> {
        Ok(())
    }
    fn visit_statement_kind(&mut self, statement: &StatementKind) -> Result<(), String> {
        walk_statement_kind(self, statement)
    }
    fn visit_statement(&mut self, statement: &Statement) -> Result<(), String> {
        walk_statement(self, statement)
    }
    fn visit_expression(&mut self, expr: &Expr) -> Result<(), String> {
        walk_expr(self, expr)
    }

    fn visit_type(&mut self, _ty: &Type) -> Result<(), String> { Ok(()) }
}

pub fn walk_module<V: Visitor + ?Sized>(v: &mut V, module: &Module) -> Result<(), String> {
    for stmt in &module.stmts {
        v.visit_statement_kind(stmt)?;
    }
    Ok(())
}

pub fn walk_function<V: Visitor + ?Sized>(v: &mut V, function: &Function) -> Result<(), String> {
    // Visit parameter names as identifiers
    for param in &function.params {
        v.visit_expression(&Expr::Identifier(param.name.clone(), param.span.clone()))?;
        v.visit_type(&param.ty)?;
    }
    v.visit_type(&function.return_type)?;
    v.visit_statement(&function.body)
}

pub fn walk_struct<V: Visitor + ?Sized>(v: &mut V, structure: &Struct) -> Result<(), String> {
    for field in &structure.fields {
        v.visit_expression(&Expr::Identifier(field.name.clone(), field.span.clone()))?;
        v.visit_type(&field.ty)?;
    }
    Ok(())
}

pub fn walk_enum<V: Visitor + ?Sized>(v: &mut V, enumeration: &Enum) -> Result<(), String> {
    for variant in &enumeration.variants {
        for field_ty in &variant.fields {
            v.visit_type(field_ty)?;
        }
    }
    Ok(())
}

pub fn walk_extend<V: Visitor + ?Sized>(v: &mut V, extend: &Extend) -> Result<(), String> {
    for method in &extend.methods {
        v.visit_function(method)?;
    }
    Ok(())
}

pub fn walk_trait<V: Visitor + ?Sized>(v: &mut V, trait_: &Trait) -> Result<(), String> {
    for method in &trait_.methods {
        v.visit_function(method)?;
    }
    Ok(())
}

pub fn walk_statement_kind<V: Visitor + ?Sized>(v: &mut V, statement: &StatementKind) -> Result<(), String> {
    match statement {
        StatementKind::Function(f) => v.visit_function(f),
        StatementKind::Struct(s) => v.visit_struct(s),
        StatementKind::Enum(e) => v.visit_enum(e),
        StatementKind::Extend(ext) => v.visit_extend(ext),
        StatementKind::Trait(t) => v.visit_trait(t),
        StatementKind::Import(i) => v.visit_import(i),
        StatementKind::Statement(stmt) => v.visit_statement(stmt),
    }
}

pub fn walk_statement<V: Visitor + ?Sized>(v: &mut V, statement: &Statement) -> Result<(), String> {
    match statement {
        Statement::Expr(expr) => v.visit_expression(expr),
        Statement::Let { name: _, ty, value, .. } => {
            if let Some(val) = value {
                v.visit_expression(val)?;
            }
            if let Some(ty) = ty {
                v.visit_type(ty)?;
            }
            Ok(())
        }
        Statement::Block(stmts, _) => {
            for stmt in stmts {
                v.visit_statement(stmt)?;
            }
            Ok(())
        }
        Statement::If { cond, then_branch, else_branch, .. } => {
            v.visit_expression(cond)?;
            v.visit_statement(then_branch)?;
            if let Some(else_b) = else_branch {
                v.visit_statement(else_b)?;
            }
            Ok(())
        }
        Statement::While { cond, body, .. } => {
            v.visit_expression(cond)?;
            v.visit_statement(body)?;
            Ok(())
        }
        Statement::For { var: _, iter, body, .. } => {
            v.visit_expression(iter)?;
            v.visit_statement(body)?;
            Ok(())
        }
        Statement::Match { expr, cases, .. } => {
            v.visit_expression(expr)?;
            for case in cases {
                v.visit_statement(&case.body)?;
            }
            Ok(())
        }
        Statement::Return(val, _) => {
            if let Some(val) = val {
                v.visit_expression(val)?;
            }
            Ok(())
        }
        Statement::Break(_) | Statement::Continue(_) => Ok(()),
    }
}

pub fn walk_expr<V: Visitor + ?Sized>(v: &mut V, expr: &Expr) -> Result<(), String> {
    match expr {
        Expr::Unary { expr: e, .. } => v.visit_expression(e),
        Expr::Binary { left, right, .. } => {
            v.visit_expression(left)?;
            v.visit_expression(right)
        }
        Expr::Assignment { left, right, .. } => {
            v.visit_expression(left)?;
            v.visit_expression(right)
        }
        Expr::Call { callee, args, .. } => {
            v.visit_expression(callee)?;
            for arg in args {
                v.visit_expression(arg)?;
            }
            Ok(())
        }
        Expr::Field { base, .. } => v.visit_expression(base),
        Expr::Index { base, index, .. } => {
            v.visit_expression(base)?;
            v.visit_expression(index)
        }
        Expr::Cast { expr: e, ty, .. } => {
            v.visit_expression(e)?;
            v.visit_type(ty)?;
            Ok(())
        }
        Expr::Grouping(e, _) => v.visit_expression(e),
        Expr::Array { elements, .. } => {
            for el in elements {
                v.visit_expression(el)?;
            }
            Ok(())
        }
        Expr::Tuple { elements, .. } => {
            for el in elements {
                v.visit_expression(el)?;
            }
            Ok(())
        }
        Expr::StructInit { fields, .. } => {
            for (_, val) in fields {
                v.visit_expression(val)?;
            }
            Ok(())
        }
        Expr::Closure { body, params, .. } => {
            for param in params {
                v.visit_expression(&Expr::Identifier(param.name.clone(), param.span.clone()))?;
                v.visit_type(&param.ty)?;
            }
            v.visit_statement(body)
        }
        Expr::TokenInterpolation(_, _) => Ok(()),
        Expr::Identifier(_, _) | Expr::Literal(_) | Expr::Error => Ok(()),
    }
}