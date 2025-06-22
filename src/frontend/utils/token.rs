#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub line: usize,
    pub span: Span,
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct Span {
    pub start: usize,
    pub end: usize
}

impl Span {
    pub fn new(start: usize, end: usize) -> Span {
        Span { start, end }
    }

    pub fn default() -> Span {
        Span { start: 0, end: 0 }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Symbols
    Plus, Minus, Star, Slash, Gt, Lt, Eq, Bang, Amp, Pipe, Caret,   // + - * / > < = ! & | ^
    Mod, Question, Colon, Semicolon, Comma, Dot, Lparen, Rparen,    // % ? : ; , . ( )
    Lbrace, Rbrace, Lbracket, Rbracket, GtEq, LtEq, EqEq, BangEq,   // { } [ ] >= <= == !=
    AmpAmp, PipePipe, PlusEq, MinusEq, StarEq, SlashEq, ModEq,      // && || += -= *= /= %=
    AmpEq, PipeEq, CaretEq, Range, Arrow, Hash, Pow,                // &= |= ^= .. -> # **
    QuestionQuestion, Underscore,                                   // ?? _

    // Keywords
    If, Else, While, For, In, Break, Continue, Return, Func,        // if else while for in break continue return func
    Struct, Enum, Import, As, Match, Case, Trait, Extend,           // struct enum import as match case trait extend
    Pub, Priv, Protected, Type, True, False, Null,                  // pub priv protected type true false null
    Module, Extern, Let,                                            // module extern let

    // Literals
    Integer, Float, String, Char, Identifier,                       // integer float string char identifier

    // End of file
    Eof,

    // Misc
    Error
}

impl Token {
    pub fn new(kind: TokenKind, lexeme: String, line: usize, span: Span) -> Self {
        Token {
            kind,
            lexeme,
            line,
            span
        }
    }

    pub fn to_string(&self) -> String {
        format!("Token {{ kind: {:?}, lexeme: {}, line: {}, span: {:?} }}", self.kind, self.lexeme, self.line, self.span)
    }
}

impl TokenKind {
    pub fn assignment_operators() -> Vec<Self> {
        vec![
            TokenKind::PlusEq,
            TokenKind::MinusEq,
            TokenKind::StarEq,
            TokenKind::SlashEq,
            TokenKind::ModEq,
            TokenKind::AmpEq,
            TokenKind::PipeEq,
            TokenKind::CaretEq,
        ]
    }
}