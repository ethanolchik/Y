use crate::frontend::utils::{
    ast::*,
    token::{Token, TokenKind, Span}
};

use crate::errors::Error;

pub struct Parser<'src> {
    pub tokens: &'src [Token],
    pub current: usize,
    pub had_error: bool,
    pub source: &'src str,
    pub filename: String,

    pub errors: usize,

    current_modifier: AccessModifier,

    type_stack: usize,
    generic_stack: usize,

    type_var_only: bool,

    module: Module,
}

impl<'src> Parser<'src> {
    pub fn new(tokens: &'src [Token], source: &'src str, filename: String) -> Self {
        Parser {
            tokens,
            current: 0,
            had_error: false,
            source,
            filename,
            errors: 0,
            current_modifier: AccessModifier::None,
            type_stack: 0,
            generic_stack: 0,
            type_var_only: false,
            module: Module {
                name: Token::new(TokenKind::Identifier, "module".to_string(), 1, Span::default()),
                imports: vec![],
                stmts: vec![],
                span: Span::default(),
            },
        }
    }

    pub fn parse(&mut self) -> Module {
        self.consume(TokenKind::Module, "Expected 'module' at the start of the file");

        self.module = self.parse_module();

        self.consume(TokenKind::Semicolon, "Expected ';' after module declaration");

        while !self.is_at_end() {
            let stmt = self.declaration();
            if self.had_error {
                self.synchronise();
                self.had_error = false;
                continue;
            }
            self.module.stmts.push(stmt);
        }
        
        self.module.clone()
    }

    fn declaration(&mut self) -> StatementKind {
        if self.match_token(TokenKind::Pub) {
            self.current_modifier = AccessModifier::Public;
        } else if self.match_token(TokenKind::Priv) {
            self.current_modifier = AccessModifier::Private;
        } else if self.match_token(TokenKind::Protected) {
            self.current_modifier = AccessModifier::Protected;
        }

        if self.match_token(TokenKind::Func) {
            return StatementKind::Function(self.parse_function("func"));
        } else if self.match_token(TokenKind::Struct) {
            return StatementKind::Struct(self.parse_struct());
        } else if self.match_token(TokenKind::Enum) {
            return StatementKind::Enum(self.parse_enum());
        } else if self.match_token(TokenKind::Extend) {
            return StatementKind::Extend(self.parse_extend());
        } else if self.match_token(TokenKind::Trait) {
            return StatementKind::Trait(self.parse_trait());
        } else if self.match_token(TokenKind::Import) {
            return StatementKind::Import(self.parse_import());
        } else {
            if self.current_modifier != AccessModifier::None {
                self.error("Access modifier must be used with a declaration");
            }
            
            self.current_modifier = AccessModifier::None;
            return StatementKind::Statement(self.parse_statement());
        }
    }

    fn parse_statement(&mut self) -> Statement {
        if self.match_token(TokenKind::Let) {
            return self.parse_let_statement();
        } else if self.match_token(TokenKind::If) {
            return self.parse_if_statement();
        } else if self.match_token(TokenKind::While) {
            return self.parse_while_statement();
        } else if self.match_token(TokenKind::For) {
            return self.parse_for_statement();
        } else if self.match_token(TokenKind::Match) {
            return self.parse_match_statement();
        } else if self.match_token(TokenKind::Return) {
            return self.parse_return_statement();
        } else if self.match_token(TokenKind::Break) {
            return self.parse_break_statement();
        } else if self.match_token(TokenKind::Continue) {
            return self.parse_continue_statement();
        } else if self.match_token(TokenKind::Lbrace) {
            return self.parse_block();
        } else {
            return self.parse_expression_statement();
        }
    }

    fn parse_function(&mut self, kind: &str) -> Function {
        let is_method = match kind {
            "func" => false,
            "method" => true,
            _ => false,
        };

        let start = self.peek().span.start;
        let access = self.current_modifier.clone();

        self.current_modifier = AccessModifier::None;

        let name = self.consume(TokenKind::Identifier, "Expected function name").clone();

        self.consume(TokenKind::Lparen, "Expected '(' after function name");
        let params = self.parse_parameters(TokenKind::Rparen);
        self.consume(TokenKind::Rparen, "Expected ')' after function parameters");

        let return_type: Type;

        if self.match_token(TokenKind::Arrow) {
            return_type = self.type_expression();
        } else {
            return_type = Type::Primitive {
                name: Token::new(TokenKind::Identifier, "void".to_string(), 1, Span::default()),
                span: Span::default(),
            }
        }

        self.consume(TokenKind::Lbrace, "Expected '{' after function declaration");

        let body = self.parse_block();

        Function {
            access,
            name,
            params,
            return_type,
            body,
            span: Span::new(start, self.peek().span.start),
            is_method,
        }
    }

    fn parse_parameters(&mut self, tkn: TokenKind) -> Vec<Parameter> {
        let mut params = vec![];

        while !self.check(tkn.clone()) {
            if params.len() > 255 {
                self.error("Too many parameters, maximum is 255");
                return params;
            }

            let name = self.consume(TokenKind::Identifier, "Expected parameter name").clone();
            self.consume(TokenKind::Colon, "Expected ':' after parameter name");
            let ty = self.type_expression();

            params.push(Parameter {
                name,
                ty,
                span: Span::new(self.previous().span.start, self.peek().span.start),
            });

            if !self.check(tkn.clone()) {
                self.consume(TokenKind::Comma, "Expected ',' after parameter");
            }
        }

        params
    }

    fn type_expression(&mut self) -> Type {
        let start = self.peek().span.start;
        self.type_stack += 1;

        if self.type_var_only {
            if self.match_token(TokenKind::Identifier) {
                let name = self.previous().clone();
                self.type_stack -= 1;
                return Type::TypeVar {
                    name: name.clone(),
                    span: Span::new(start, self.peek().span.start),
                };
            } else {
                self.error("Generic expression only accepts type variables here");
            }
        }

        if self.match_token(TokenKind::Identifier) {
            let name = self.previous().clone();

            // Check if the type is a primitive type
            match name.lexeme.clone().as_str() {
                "int" | "float" | "string" | "char" | "bool" => {
                    self.type_stack -= 1;
                    return Type::Primitive {
                        name: name.clone(),
                        span: Span::new(start, self.peek().span.start),
                    };
                }
                _ => {}
            }

            // Check if the outer type is a generic type
            let mut generics: Vec<Type> = vec![];

            if self.match_token(TokenKind::Lt) {
                self.generic_stack += 1;
                loop {
                    generics.push(self.type_expression());

                    if !self.match_token(TokenKind::Comma) {
                        break;
                    }
                }

                self.consume(TokenKind::Gt, "Expected '>' after generic type");
                self.generic_stack -= 1;
            }
            self.type_stack -= 1;
            if generics.is_empty() && self.generic_stack > 0{
                return Type::TypeVar {
                    name: name.clone(),
                    span: Span::new(start, self.peek().span.start),
                };
            }
            return Type::Named {
                name: name.clone(),
                generics: generics,
                span: Span::new(start, self.peek().span.start),
            };
        }

        // Parse multi-dimensional arrays like [[[T]]]
        let mut array_depth = 0;
        while self.match_token(TokenKind::Lbracket) {
            array_depth += 1;
        }
        if array_depth > 0 {
            let mut inner_type = self.type_expression();
            for _ in 0..array_depth {
            self.consume(TokenKind::Rbracket, "Expected ']' after type");
            inner_type = Type::Array {
                element: Box::new(inner_type),
                size: None,
                span: Span::new(start, self.peek().span.start),
            };
            }
            return inner_type;
        }

        // Parse tuples like (T, U)
        if self.match_token(TokenKind::Lparen) {
            let mut elements = vec![];
            while !self.check(TokenKind::Rparen) {
                let element = self.type_expression();
                elements.push(element);
                if !self.check(TokenKind::Rparen) {
                    self.consume(TokenKind::Comma, "Expected ',' after type");
                }
            }
            self.consume(TokenKind::Rparen, "Expected ')' after type");

            // Parse function types like (T, U) -> V
            if self.match_token(TokenKind::Arrow) {
                let return_type = self.type_expression();
                return Type::Function {
                    params: elements,
                    return_type: Box::new(return_type),
                    span: Span::new(start, self.peek().span.start),
                };
            }

            return Type::Tuple {
                elements,
                span: Span::new(start, self.peek().span.start),
            };
        }

        self.error("Expected type expression");

        self.type_stack -= 1;
        Type::Error(Span::new(start, self.peek().span.start))
    }

    fn parse_block(&mut self) -> Statement {
        let start = self.peek().span.start;
        let mut stmts = vec![];

        while !self.check(TokenKind::Rbrace) {
            stmts.push(self.parse_statement());
        }

        self.consume(TokenKind::Rbrace, "Expected '}' after block");

        Statement::Block(stmts, Span::new(start, self.peek().span.start))
    }

    fn parse_struct(&mut self) -> Struct {
        let start = self.peek().span.start;
        let access = self.current_modifier.clone();
        self.current_modifier = AccessModifier::None;
        let name = self.consume(TokenKind::Identifier, "Expected struct name").clone();

        let mut generics = vec![];
        if self.match_token(TokenKind::Lt) {
            self.generic_stack += 1;
            while !self.check(TokenKind::Gt) {
                // For now only allow type variables as generics
                self.type_var_only = true;
                let generic_type = self.type_expression();
                self.type_var_only = false;
                generics.push(generic_type);
                if !self.check(TokenKind::Gt) {
                    self.consume(TokenKind::Comma, "Expected ',' after generic type");
                }
            }
            self.consume(TokenKind::Gt, "Expected '>' after generic type");
            self.generic_stack -= 1;
        }

        self.consume(TokenKind::Lbrace, "Expected '{' after struct name");

        let mut fields = vec![];

        while !self.check(TokenKind::Rbrace) {
            fields = self.struct_fields();
            if !self.check(TokenKind::Rbrace) {
                self.consume(TokenKind::Comma, "Expected ',' after field");
            }
        }

        self.consume(TokenKind::Rbrace, "Expected '}' after struct declaration");

        Struct {
            access,
            name,
            fields,
            generics,
            span: Span::new(start, self.peek().span.start),
        }
    }

    fn struct_fields(&mut self) -> Vec<Field> {
        let mut fields = vec![];

        while !self.check(TokenKind::Rbrace) {
            let mut access = AccessModifier::None;

            if self.match_token(TokenKind::Pub) {
                access = AccessModifier::Public;
            } else if self.match_token(TokenKind::Priv) {
                access = AccessModifier::Private;
            } else if self.match_token(TokenKind::Protected) {
                access = AccessModifier::Protected;
            }

            let name = self.consume(TokenKind::Identifier, "Expected field name").clone();
            self.consume(TokenKind::Colon, "Expected ':' after field name");
            let ty = self.type_expression();

            fields.push(Field {
                access,
                name,
                ty,
                span: Span::new(self.previous().span.start, self.peek().span.start),
            });

            if !self.check(TokenKind::Rbrace) {
                self.consume(TokenKind::Comma, "Expected ',' after field");
            }
        }

        fields
    }

    fn parse_enum(&mut self) -> Enum {
        let start = self.peek().span.start;
        let access = self.current_modifier.clone();
        self.current_modifier = AccessModifier::None;
        let name = self.consume(TokenKind::Identifier, "Expected enum name").clone();

        self.consume(TokenKind::Lbrace, "Expected '{' after enum name");

        let mut variants = vec![];

        while !self.check(TokenKind::Rbrace) {
            let variant = self.parse_enum_variant();
            variants.push(variant);
            if !self.check(TokenKind::Rbrace) {
                self.consume(TokenKind::Comma, "Expected ',' after enum variant");
            }
        }

        self.consume(TokenKind::Rbrace, "Expected '}' after enum declaration");

        Enum {
            access,
            name,
            variants,
            span: Span::new(start, self.peek().span.start),
        }
    }

    fn parse_enum_variant(&mut self) -> EnumVariant {
        let start = self.peek().span.start;
        let name = self.consume(TokenKind::Identifier, "Expected enum variant name").clone();

        let mut fields = vec![];

        if self.match_token(TokenKind::Lparen) {
            while !self.check(TokenKind::Rparen) {
                let field = self.type_expression();
                fields.push(field);
                if !self.check(TokenKind::Rparen) {
                    self.consume(TokenKind::Comma, "Expected ',' after enum variant field");
                }
            }
            self.consume(TokenKind::Rparen, "Expected ')' after enum variant fields");
        }

        EnumVariant {
            name,
            fields,
            span: Span::new(start, self.peek().span.start),
        }
    }

    fn parse_trait(&mut self) -> Trait {
        let start = self.peek().span.start;
        let access = self.current_modifier.clone();
        self.current_modifier = AccessModifier::None;
        let name = self.consume(TokenKind::Identifier, "Expected trait name").clone();

        let mut generics = vec![];
        if self.match_token(TokenKind::Lt) {
            self.generic_stack += 1;
            while !self.check(TokenKind::Gt) {
                // For now only allow type variables as generics
                self.type_var_only = true;
                let generic_type = self.type_expression();
                self.type_var_only = false;
                generics.push(generic_type);
                if !self.check(TokenKind::Gt) {
                    self.consume(TokenKind::Comma, "Expected ',' after generic type");
                }
            }
            self.consume(TokenKind::Gt, "Expected '>' after generic type");
            self.generic_stack -= 1;
        }

        self.consume(TokenKind::Lbrace, "Expected '{' after trait name");

        let mut methods = vec![];

        while !self.check(TokenKind::Rbrace) {
            let method = self.parse_function("method");
            methods.push(method);
            if !self.check(TokenKind::Rbrace) {
                self.consume(TokenKind::Comma, "Expected ',' after trait method");
            }
        }

        self.consume(TokenKind::Rbrace, "Expected '}' after trait declaration");

        Trait {
            access,
            name,
            methods,
            generics,
            span: Span::new(start, self.peek().span.start),
        }
    }

    fn parse_import(&mut self) -> Import {
        let start = self.peek().span.start;
        let path = self.consume(TokenKind::String, "Expected import name").clone();

        self.consume(TokenKind::As, "Expected 'as' after import name");
        let alias = self.consume(TokenKind::Identifier, "Expected alias name").clone();

        self.consume(TokenKind::Semicolon, "Expected ';' after import declaration");

        Import {
            path,
            alias,
            span: Span::new(start, self.peek().span.start)
        }
    }

    fn parse_extend(&mut self) -> Extend {
        let start = self.peek().span.start;
        let first_name = self.consume(TokenKind::Identifier, "Expected struct or trait name").clone();

        let mut first_generics = vec![];
        if self.match_token(TokenKind::Lt) {
            self.generic_stack += 1;
            while !self.check(TokenKind::Gt) {
                // For now only allow type variables as generics
                self.type_var_only = true;
                let generic_type = self.type_expression();
                self.type_var_only = false;
                first_generics.push(generic_type);
                if !self.check(TokenKind::Gt) {
                    self.consume(TokenKind::Comma, "Expected ',' after generic type");
                }
            }
            self.consume(TokenKind::Gt, "Expected '>' after generic type");
            self.generic_stack -= 1;
        }

        let mut second_name = None;
        let mut second_generics = vec![];

        if self.match_token(TokenKind::For) {
            second_name = Some(self.consume(TokenKind::Identifier, "Expected trait name").clone());

            if self.match_token(TokenKind::Lt) {
                self.generic_stack += 1;
                while !self.check(TokenKind::Gt) {
                    // For now only allow type variables as generics
                    self.type_var_only = true;
                    let generic_type = self.type_expression();
                    self.type_var_only = false;
                    second_generics.push(generic_type);
                    if !self.check(TokenKind::Gt) {
                        self.consume(TokenKind::Comma, "Expected ',' after generic type");
                    }
                }
                self.consume(TokenKind::Gt, "Expected '>' after generic type");
                self.generic_stack -= 1;
            }
        }

        self.consume(TokenKind::Lbrace, "Expected '{' after extend declaration");

        let mut methods = vec![];
        while !self.check(TokenKind::Rbrace) {
            if self.match_token(TokenKind::Pub) {
                self.current_modifier = AccessModifier::Public;
            } else if self.match_token(TokenKind::Priv) {
                self.current_modifier = AccessModifier::Private;
            } else if self.match_token(TokenKind::Protected) {
                self.current_modifier = AccessModifier::Protected;
            }

            self.consume(TokenKind::Func, "Expected 'func' before extend method");

            let method = self.parse_function("method");
            methods.push(method);
        }
        self.consume(TokenKind::Rbrace, "Expected '}' after extend declaration");
        
        if second_name.is_none() {
            return Extend {
                name: first_name,
                trait_name: None,
                methods,
                first_generics,
                second_generics: vec![],
                span: Span::new(start, self.peek().span.start),
            };
        }

        Extend {
            name: second_name.unwrap(),
            trait_name: Some(first_name),
            methods,
            first_generics,
            second_generics,
            span: Span::new(start, self.peek().span.start),
        }
    }

    fn parse_module(&mut self) -> Module{
        let start = self.peek().span.start;
        let name = self.consume(TokenKind::Identifier, "Expected module name").clone();

        return Module {
            name,
            imports: vec![],
            stmts: vec![],
            span: Span::new(start, self.peek().span.start),
        };
    }

    fn parse_let_statement(&mut self) -> Statement {
        let start = self.peek().span.start;
        let name = self.consume(TokenKind::Identifier, "Expected variable name").clone();

        let mut ty = None;
        if self.match_token(TokenKind::Colon) {
            ty = Some(self.type_expression());
        }

        let mut value = None;
        if self.match_token(TokenKind::Eq) {
            value = Some(self.expression());
        }

        self.consume(TokenKind::Semicolon, "Expected ';' after variable declaration");

        Statement::Let {
            name,
            ty,
            value,
            span: Span::new(start, self.peek().span.start),
        }
    }

    fn parse_if_statement(&mut self) -> Statement {
        let start = self.peek().span.start;
        self.consume(TokenKind::Lparen, "Expected '(' after 'if'");
        let cond = self.expression();
        self.consume(TokenKind::Rparen, "Expected ')' after 'if' condition");
        let then_branch = Box::new(self.parse_statement());

        let mut else_branch = None;
        if self.match_token(TokenKind::Else) {
            else_branch = Some(Box::new(self.parse_statement()));
        }

        Statement::If {
            cond,
            then_branch,
            else_branch,
            span: Span::new(start, self.peek().span.start),
        }
    }
    fn parse_while_statement(&mut self) -> Statement {
        let start = self.peek().span.start;
        self.consume(TokenKind::Lparen, "Expected '(' after 'while'");
        let cond = self.expression();
        self.consume(TokenKind::Rparen, "Expected ')' after 'while' condition");
        let body = Box::new(self.parse_statement());

        Statement::While {
            cond,
            body,
            span: Span::new(start, self.peek().span.start),
        }
    }

    fn parse_for_statement(&mut self) -> Statement {
        let start = self.peek().span.start;
        self.consume(TokenKind::Lparen, "Expected '(' after 'for'");
        let var = self.consume(TokenKind::Identifier, "Expected variable name").clone();
        self.consume(TokenKind::In, "Expected 'in' after variable name");
        let iter = self.expression();
        self.consume(TokenKind::Rparen, "Expected ')' after 'for' condition");
        let body = Box::new(self.parse_statement());

        Statement::For {
            var,
            iter,
            body,
            span: Span::new(start, self.peek().span.start),
        }
    }

    fn parse_match_statement(&mut self) -> Statement {
        let start = self.peek().span.start;
        self.consume(TokenKind::Lparen, "Expected '(' after 'match'");
        let cond = self.expression();
        self.consume(TokenKind::Rparen, "Expected ')' after 'match' condition");
        self.consume(TokenKind::Lbrace, "Expected '{' after 'match' condition");

        let mut cases = vec![];
        while !self.check(TokenKind::Rbrace) {
            let case = self.parse_case();
            cases.push(case);
            if !self.check(TokenKind::Rbrace) {
                self.consume(TokenKind::Comma, "Expected ',' after match case");
            }
        }
        self.consume(TokenKind::Rbrace, "Expected '}' after match statement");

        Statement::Match {
            expr: cond,
            cases,
            span: Span::new(start, self.peek().span.start),
        }
    }

    fn parse_case(&mut self) -> Case {
        let start = self.peek().span.start;
        let pattern = self.pattern();
        self.consume(TokenKind::Arrow, "Expected '->' after match case");
        let body = self.parse_statement();

        Case {
            pattern,
            body,
            span: Span::new(start, self.peek().span.start),
        }
    }

    fn pattern(&mut self) -> Pattern {
        let start = self.peek().span.start;
        if self.match_token(TokenKind::Identifier) {
            let name = self.previous().clone();
            return Pattern::Identifier(name.clone(), name.span);
        } else if self.match_token(TokenKind::Integer) {
            let lexeme = self.previous().lexeme.as_str();
            match lexeme.parse() {
                Ok(val) => return Pattern::Literal(Literal::Integer(val, self.previous().span.clone())),
                Err(_) => {
                    self.error("Invalid integer literal");
                    return Pattern::Error;
                }
            }
        } else if self.match_token(TokenKind::String) {
            let value = self.previous().clone();
            return Pattern::Literal(Literal::Token(value.clone(), value.span));
        } else if self.match_token(TokenKind::Char) {
            let value = self.previous().clone();
            return Pattern::Literal(Literal::Token(value.clone(), value.span));
        } else if self.match_token(TokenKind::True) || self.match_token(TokenKind::False) {
            let value = self.previous().clone();
            return Pattern::Literal(Literal::Bool(
                value.lexeme.as_str().parse().unwrap(),
                Span::new(start, self.peek().span.start)
            ));
        } else if self.match_token(TokenKind::Lparen) {
            let mut patterns = vec![];
            while !self.check(TokenKind::Rparen) {
                let pattern = self.pattern();
                patterns.push(pattern);
                if !self.check(TokenKind::Rparen) {
                    self.consume(TokenKind::Comma, "Expected ',' after pattern");
                }
            }
            self.consume(TokenKind::Rparen, "Expected ')' after pattern");
            return Pattern::Tuple(patterns, Span::new(start, self.peek().span.start));
        } else if self.match_token(TokenKind::Lbrace) {
            let mut fields: Vec<(Token, Pattern)> = vec![];
            while !self.check(TokenKind::Rbrace) {
                let name = self.consume(TokenKind::Identifier, "Expected field name").clone();
                self.consume(TokenKind::Colon, "Expected ':' after field name");
                let pattern = self.pattern();
                fields.push((name, pattern));
                if !self.check(TokenKind::Rbrace) {
                    self.consume(TokenKind::Comma, "Expected ',' after field");
                }
            }
            self.consume(TokenKind::Rbrace, "Expected '}' after pattern");
            return Pattern::Struct {
                fields,
                span: Span::new(self.previous().span.start, self.peek().span.start),
            };
        } else if self.match_token(TokenKind::Underscore) {
            return Pattern::Wildcard(Span::new(start, self.peek().span.start));
        }
        self.error("Expected pattern");
        Pattern::Error
    }

    fn parse_return_statement(&mut self) -> Statement {
        let start = self.peek().span.start;
        let mut value = None;
        if !self.check(TokenKind::Semicolon) {
            value = Some(self.expression());
        }

        self.consume(TokenKind::Semicolon, "Expected ';' after return statement");

        Statement::Return(value, Span::new(start, self.peek().span.start))
    }

    fn parse_break_statement(&mut self) -> Statement {
        let start = self.peek().span.start;
        self.consume(TokenKind::Semicolon, "Expected ';' after 'break'");
        Statement::Break(Span::new(start, self.peek().span.start))
    }

    fn parse_continue_statement(&mut self) -> Statement {
        let start = self.peek().span.start;
        self.consume(TokenKind::Semicolon, "Expected ';' after 'continue'");
        Statement::Continue(Span::new(start, self.peek().span.start))
    }

    fn parse_expression_statement(&mut self) -> Statement {
        let expr = self.expression();
        self.consume(TokenKind::Semicolon, "Expected ';' after expression");
        Statement::Expr(expr)
    }

    fn expression(&mut self) -> Expr {
        return self.assignment();
    }

    fn assignment(&mut self) -> Expr {
        let expr = self.logical_or();

        if TokenKind::assignment_operators().contains(&self.peek().kind) {
            let op = self.advance().clone();
            let value = self.assignment();
            return Expr::Assignment {
                left: Box::new(expr),
                op,
                right: Box::new(value),
                span: Span::new(self.previous().span.start, self.peek().span.start),
            };
        }

        expr
    }

    fn logical_or(&mut self) -> Expr {
        let mut expr = self.logical_and();

        while self.match_token(TokenKind::PipePipe) {
            let op = self.previous().clone();
            let right = self.logical_and();
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span: Span::new(self.previous().span.start, self.peek().span.start),
            };
        }

        expr
    }

    fn logical_and(&mut self) -> Expr {
        let mut expr = self.nullish_coalesce();

        while self.match_token(TokenKind::AmpAmp) {
            let op = self.previous().clone();
            let right = self.nullish_coalesce();
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span: Span::new(self.previous().span.start, self.peek().span.start),
            };
        }

        expr
    }

    fn nullish_coalesce(&mut self) -> Expr {
        let mut expr = self.bitwise_or();

        while self.match_token(TokenKind::QuestionQuestion) {
            let op = self.previous().clone();
            let right = self.bitwise_or();
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span: Span::new(self.previous().span.start, self.peek().span.start),
            };
        }

        expr
    }

    fn bitwise_or(&mut self) -> Expr {
        let mut expr = self.bitwise_xor();

        while self.match_token(TokenKind::Pipe) {
            let op = self.previous().clone();
            let right = self.bitwise_xor();
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span: Span::new(self.previous().span.start, self.peek().span.start),
            };
        }

        expr
    }

    fn bitwise_xor(&mut self) -> Expr {
        let mut expr = self.bitwise_and();

        while self.match_token(TokenKind::Caret) {
            let op = self.previous().clone();
            let right = self.bitwise_and();
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span: Span::new(self.previous().span.start, self.peek().span.start),
            };
        }

        expr
    }

    fn bitwise_and(&mut self) -> Expr {
        let mut expr = self.equality();

        while self.match_token(TokenKind::Amp) {
            let op = self.previous().clone();
            let right = self.equality();
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span: Span::new(self.previous().span.start, self.peek().span.start),
            };
        }

        expr
    }

    fn equality(&mut self) -> Expr {
        let mut expr = self.comparison();

        while self.match_token(TokenKind::BangEq) || self.match_token(TokenKind::EqEq) {
            let op = self.previous().clone();
            let right = self.comparison();
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span: Span::new(self.previous().span.start, self.peek().span.start),
            };
        }

        expr
    }

    fn comparison(&mut self) -> Expr {
        let mut expr = self.addition();

        while self.match_token(TokenKind::Lt)
            || self.match_token(TokenKind::LtEq)
            || self.match_token(TokenKind::Gt)
            || self.match_token(TokenKind::GtEq)
        {
            let op = self.previous().clone();
            let right = self.addition();
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span: Span::new(self.previous().span.start, self.peek().span.start),
            };
        }

        expr
    }

    fn addition(&mut self) -> Expr {
        let mut expr = self.multiplication();

        while self.match_token(TokenKind::Plus) || self.match_token(TokenKind::Minus) {
            let op = self.previous().clone();
            let right = self.multiplication();
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span: Span::new(self.previous().span.start, self.peek().span.start),
            };
        }

        expr
    }

    fn multiplication(&mut self) -> Expr {
        let mut expr = self.exponent();

        while self.match_token(TokenKind::Star)
            || self.match_token(TokenKind::Slash)
            || self.match_token(TokenKind::Mod)
        {
            let op = self.previous().clone();
            let right = self.exponent();
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span: Span::new(self.previous().span.start, self.peek().span.start),
            };
        }

        expr
    }

    fn exponent(&mut self) -> Expr {
        let mut expr = self.unary();

        while self.match_token(TokenKind::Pow) {
            let op = self.previous().clone();
            let right = self.unary();
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span: Span::new(self.previous().span.start, self.peek().span.start),
            };
        }

        expr
    }

    fn unary(&mut self) -> Expr {
        if self.match_token(TokenKind::Bang) || self.match_token(TokenKind::Minus) {
            let op = self.previous().clone();
            let right = self.cast();
            return Expr::Unary {
                op,
                expr: Box::new(right),
                span: Span::new(self.previous().span.start, self.peek().span.start),
            };
        }

        self.cast()
    }

    fn cast(&mut self) -> Expr {
        let mut expr = self.index();
        while self.match_token(TokenKind::As) {
            let start = self.previous().span.start;
            let ty = self.type_expression();
            expr = Expr::Cast {
                expr: Box::new(expr),
                ty,
                span: Span::new(start, self.peek().span.start),
            };
        }
        expr
    }

    fn index(&mut self) -> Expr {
        let mut expr = self.property_access();

        while self.match_token(TokenKind::Lbracket) {
            let index = self.expression();
            self.consume(TokenKind::Rbracket, "Expected ']' after index");
            expr = Expr::Index {
                base: Box::new(expr),
                index: Box::new(index),
                span: Span::new(self.previous().span.start, self.peek().span.start),
            };
        }

        expr
    }

    fn property_access(&mut self) -> Expr {
        let mut expr = self.call();

        while self.match_token(TokenKind::Dot) {
            let name = self.consume(TokenKind::Identifier, "Expected property name").clone();

            expr = Expr::Field { 
                base: Box::new(expr),
                field: name.clone(),
                span: Span::new(self.previous().span.start, name.span.start),
            };
        }

        expr
    }

    fn call(&mut self) -> Expr {
        let mut expr = self.primary();

        let mut generics: Vec<Type> = vec![];

        loop {
            if self.match_token(TokenKind::Lt) {
                self.generic_stack += 1;
                while !self.check(TokenKind::Gt) {
                    let generic_type = self.type_expression();
                    generics.push(generic_type);
                    if !self.check(TokenKind::Gt) {
                        self.consume(TokenKind::Comma, "Expected ',' after generic type");
                    }
                }
                self.consume(TokenKind::Gt, "Expected '>' after generic type");
                self.generic_stack -= 1;
            }

            if self.match_token(TokenKind::Lparen) {
                expr = self.finish_call(expr, generics.clone());
            } else if self.match_token(TokenKind::Dot) {
                let name = self.consume(TokenKind::Identifier, "Expected property name").clone();
                expr = Expr::Field {
                    base: Box::new(expr),
                    field: name.clone(),
                    span: Span::new(self.previous().span.start, name.span.start),
                };
            } else if self.match_token(TokenKind::Lbracket) {
                let index = self.expression();
                self.consume(TokenKind::Rbracket, "Expected ']' after index");
                expr = Expr::Index {
                    base: Box::new(expr),
                    index: Box::new(index),
                    span: Span::new(self.previous().span.start, self.peek().span.start),
                };
            } else {
                break;
            }
        }

        expr
    }

    fn finish_call(&mut self, callee: Expr, generics: Vec<Type>) -> Expr {
        let mut args = vec![];

        if !self.check(TokenKind::Rparen) {
            loop {
                if args.len() >= 255 {
                    self.error("Cannot have more than 255 arguments");
                }
                args.push(self.expression());
                if !self.check(TokenKind::Rparen) {
                    self.consume(TokenKind::Comma, "Expected ',' after argument");
                } else {
                    break;
                }
            }
        }

        let prev = self.previous().span.start;
        let paren = self.consume(TokenKind::Rparen, "Expected ')' after arguments");

        Expr::Call {
            callee: Box::new(callee),
            args,
            generic_args: generics,
            span: Span::new(prev, paren.span.start),
        }
    }

    fn primary(&mut self) -> Expr {
        if self.match_token(TokenKind::True) {
            return Expr::Literal(Literal::Bool(true, self.previous().span.clone()));
        }
        if self.match_token(TokenKind::False) {
            return Expr::Literal(Literal::Bool(false, self.previous().span.clone()));
        }
        if self.match_token(TokenKind::Null) {
            return Expr::Literal(Literal::Null(self.previous().span.clone()));
        }
        if self.match_token(TokenKind::Integer) {
            let lexeme = self.previous().lexeme.as_str();
            match lexeme.parse() {
                Ok(val) => return Expr::Literal(Literal::Integer(val, self.previous().span.clone())),
                Err(_) => {
                    self.error("Invalid integer literal");
                    return Expr::Error;
                }
            }
        }
        if self.match_token(TokenKind::Float) {
            return Expr::Literal(Literal::Float(
                self.previous().lexeme.as_str().parse().unwrap(),
                self.previous().span.clone(),
            ));
        }
        if self.match_token(TokenKind::String) {
            return Expr::Literal(Literal::Token(
                self.previous().clone(),
                self.previous().span.clone(),
            ));
        }
        if self.match_token(TokenKind::Char) {
            return Expr::Literal(Literal::Token(
                self.previous().clone(),
                self.previous().span.clone(),
            ));
        }

        if self.match_token(TokenKind::Identifier) {
            let x = self.previous().clone();

            if self.match_token(TokenKind::Lbrace) {
                return self.struct_init(x);
            }

            return Expr::Identifier(x.clone(),x.span);
        }

        if self.match_token(TokenKind::Pipe) {
            return self.closure();
        }

        if self.match_token(TokenKind::Lbracket) {
            let mut elements = vec![];
            while !self.check(TokenKind::Rbracket) {
                let element = self.expression();
                elements.push(element);
                if !self.check(TokenKind::Rbracket) {
                    self.consume(TokenKind::Comma, "Expected ',' after array element");
                }
            }
            let prev_start = self.previous().span.start;
            let rbracket = self.consume(TokenKind::Rbracket, "Expected ']' after array literal");
            return Expr::Array {
                elements,
                span: Span::new(prev_start, rbracket.span.start)
            };
        }

        if self.match_token(TokenKind::Lparen) {
            let expr = self.expression();
            if self.match_token(TokenKind::Comma) {
                let mut elements = vec![expr];
                while !self.check(TokenKind::Rparen) {
                    let element = self.expression();
                    elements.push(element);
                    if !self.check(TokenKind::Rparen) {
                        self.consume(TokenKind::Comma, "Expected ',' after tuple element");
                    }
                }
                let prev_start = self.previous().span.start;
                let rparen = self.consume(TokenKind::Rparen, "Expected ')' after tuple literal");
                return Expr::Tuple {
                    elements,
                    span: Span::new(prev_start, rparen.span.start)
                };
            }
            let prev = self.previous().span.start;
            let paren = self.consume(TokenKind::Rparen, "Expected ')' after expression");
            return Expr::Grouping(Box::new(expr), Span::new(prev, paren.span.start));
        }

        self.error("Expected expression");
        Expr::Error
    }

    fn struct_init(&mut self, name: Token) -> Expr {
        let start = self.peek().span.start;
        let mut fields = vec![];

        while !self.check(TokenKind::Rbrace) {
            let field_name = self.consume(TokenKind::Identifier, "Expected field name").clone();

            if self.match_token(TokenKind::Colon) {
                // i.e. Foo { x: 1, y: 2 }
                let value = self.expression();
                fields.push((field_name, value));

            } else {
                // i.e. Foo { x, y }
                // In this case we assume the field name is the same as the variable name
                // and we will resolve it later
                fields.push((field_name.clone(), Expr::Identifier(field_name.clone(), field_name.span.clone())));
            }
    
            if !self.check(TokenKind::Rbrace) {
                self.consume(TokenKind::Comma, "Expected ',' after field");
            }
        }

        let rbrace = self.consume(TokenKind::Rbrace, "Expected '}' after struct literal");

        Expr::StructInit {
            name,
            fields,
            span: Span::new(start, rbrace.span.start),
        }
    }

    fn closure(&mut self) -> Expr {
        let start = self.previous().span.start;
        let mut params = vec![];

        while !self.check(TokenKind::Pipe) {
            params = self.parse_parameters(TokenKind::Pipe);
        }

        self.consume(TokenKind::Pipe, "Expected '|' after closure parameters");

        let ty = self.type_expression();

        let body = Box::new(self.parse_statement());

        Expr::Closure {
            params,
            ty,
            body,
            span: Span::new(start, self.peek().span.start),
        }
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }

        self.previous()
    }

    fn match_token(&mut self, kind: TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            return true;
        }

        false
    }

    fn check(&mut self, kind: TokenKind) -> bool {
        if self.is_at_end() {
            return false;
        }

        self.peek().kind == kind
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn peek(&self) -> Token {
        self.tokens[self.current].clone()
    }

    fn is_at_end(&self) -> bool {
        self.peek().kind == TokenKind::Eof
    }

    fn consume(&mut self, kind: TokenKind, message: &str) -> &Token {
        if self.check(kind) {
            return self.advance();
        }
        self.error(message);

        self.advance();
        self.previous()
    }

    fn error(&mut self, message: &str) -> Error {
        self.had_error = true;
        self.errors += 1;
        let mut e = Error::new(message.to_string(), self.peek().line, self.peek().span.clone(), self.filename.clone());
        e.add_source(self.source.to_string());

        eprintln!("{}", e.to_string());
        e
    }

    fn synchronise(&mut self) {
        self.advance();
        while !self.is_at_end() {
            if self.previous().kind == TokenKind::Semicolon {
                return;
            }
            match self.peek().kind {
                TokenKind::Func | TokenKind::Struct | TokenKind::Enum
                | TokenKind::Trait | TokenKind::Import | TokenKind::Extend => return,
                TokenKind::Let | TokenKind::If | TokenKind::While | TokenKind::For
                | TokenKind::Match | TokenKind::Return | TokenKind::Break
                | TokenKind::Continue => return,
                _ => {}
            }
            self.advance();
        }
    }
}