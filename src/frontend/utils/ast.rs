use crate::frontend::utils::token::{Span, Token};

#[derive(Debug, Clone, PartialEq)]
pub enum StatementKind {
    Function(Function),
    Struct(Struct),
    Enum(Enum),
    Extend(Extend),
    Trait(Trait),
    Import(Import),
    Statement(Statement),
}

#[derive(Debug, Clone, PartialEq)]
pub enum AccessModifier {
    Public,
    Private,
    Protected,
    None, // default if not specified
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub access: AccessModifier,
    pub name: Token,
    pub params: Vec<Parameter>,
    pub return_type: Type,
    pub body: Statement,
    pub span: Span,

    pub is_method: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: Token,
    pub ty: Type,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Struct {
    pub access: AccessModifier,
    pub name: Token,
    pub fields: Vec<Field>,
    pub generics: Vec<Type>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub access: AccessModifier,
    pub name: Token,
    pub ty: Type,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Enum {
    pub access: AccessModifier,
    pub name: Token,
    pub variants: Vec<EnumVariant>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumVariant {
    pub name: Token,
    pub fields: Vec<Type>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Trait {
    pub access: AccessModifier,
    pub name: Token,
    pub methods: Vec<Function>,
    pub generics: Vec<Type>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Import {
    pub path: Token,
    pub alias: Token,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    pub name: Token,
    pub imports: Vec<Import>,
    pub stmts: Vec<StatementKind>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Extend {
    pub name: Token, // Name of the struct being extended
    pub trait_name: Option<Token>, // Name of the trait being implemented
    pub methods: Vec<Function>,
    pub first_generics: Vec<Type>,
    pub second_generics: Vec<Type>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Identifier(Token, Span),
    Literal(Literal),
    Wildcard(Span),
    Tuple(Vec<Pattern>, Span),
    Struct {
        fields: Vec<(Token, Pattern)>,
        span: Span,
    },
    Error
}

#[derive(Debug, Clone, PartialEq)]
pub struct Case {
    pub pattern: Pattern,
    pub body: Statement,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Match {
    pub expr: Expr,
    pub cases: Vec<Case>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Let {
        name: Token,
        ty: Option<Type>,
        value: Option<Expr>,
        span: Span,
    },
    Expr(Expr),
    Return(Option<Expr>, Span),
    Break(Span),
    Continue(Span),
    Block(Vec<Statement>, Span),
    If {
        cond: Expr,
        then_branch: Box<Statement>,
        else_branch: Option<Box<Statement>>,
        span: Span,
    },
    While {
        cond: Expr,
        body: Box<Statement>,
        span: Span,
    },
    For {
        var: Token,
        iter: Expr,
        body: Box<Statement>,
        span: Span,
    },
    Match {
        expr: Expr,
        cases: Vec<Case>,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Identifier(Token, Span),
    Literal(Literal),
    Binary {
        left: Box<Expr>,
        op: Token,
        right: Box<Expr>,
        span: Span,
    },
    Unary {
        op: Token,
        expr: Box<Expr>,
        span: Span,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
        generic_args: Vec<Type>,
        span: Span,
    },
    Field {
        base: Box<Expr>,
        field: Token,
        span: Span,
    },
    Index {
        base: Box<Expr>,
        index: Box<Expr>,
        span: Span,
    },
    Assignment {
        left: Box<Expr>,
        op: Token,
        right: Box<Expr>,
        span: Span,
    },
    StructInit {
        name: Token,
        fields: Vec<(Token, Expr)>,
        span: Span,
    },
    Array {
        elements: Vec<Expr>,
        span: Span,
    },
    Tuple {
        elements: Vec<Expr>,
        span: Span,
    },
    Cast {
        expr: Box<Expr>,
        ty: Type,
        span: Span,
    },
    Closure {
        params: Vec<Parameter>,
        body: Box<Statement>,
        ty: Type,
        span: Span,
    },
    TokenInterpolation(TokenInterpolation, Span),
    Grouping(Box<Expr>, Span),
    Error,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Integer(i64, Span),
    Float(f64, Span),
    Bool(bool, Span),
    Null(Span),
    Token(Token, Span),
}

#[derive(Debug, Clone, PartialEq)]
/// Represents a Token literal with possible interpolations <br>
/// For example, "Hello, \(name)!" would be represented as a TokenInterpolation with a segment for "Hello, " and an expression for \(name) <br>
/// The segments are stored in the order they appear in the original Token <br>
/// The span represents the entire Token literal
pub struct TokenInterpolation {
    pub segments: Vec<TokenSegment>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
/// Represents a segment of a Token literal, which can be either a literal Token or an expression <br>
/// Used for Token interpolation
pub enum TokenSegment {
    Literal(Token, Span),
    Expr(Expr, Span),
}

#[derive(Debug, Clone, PartialEq)]
/// Represents a type in the language
pub enum Type {
    /// A primitive type, e.g. "int", "bool", "float"
    Primitive {
        name: Token,
        span: Span,
    },
    /// A named type, e.g. "MyStruct"
    Named {
        name: Token,
        generics: Vec<Type>,
        span: Span,
    },
    /// An array type, e.g. "int[]" or "MyType[10]"
    Array {
        element: Box<Type>,
        size: Option<usize>, // None for unsized arrays
        span: Span,
    },
    /// A tuple type, e.g. "(int, Token)"
    Tuple {
        elements: Vec<Type>,
        span: Span,
    },
    /// A function type, e.g. "(int, Token) -> bool"
    Function {
        params: Vec<Type>,
        return_type: Box<Type>,
        span: Span,
    },
    /// A type variable for generics, e.g. "T"
    TypeVar {
        name: Token,
        span: Span,
    },

    Error(Span), // Represents an error in type resolution
}
