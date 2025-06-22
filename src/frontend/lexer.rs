use crate::frontend::utils::token::{
    Token,
    TokenKind,
    Span
};

use crate::errors::{Error, Help};

#[derive(Debug, Clone)]
pub struct Lexer<'src> {
    pub source: &'src str,
    pub filename: String,
    pub tokens: Vec<Token>,

    pub start: usize,
    pub current: usize,
    pub line: usize,
    pub col: usize,
    pub col_start: usize,
    pub col_end: usize,

    pub had_error: bool,
    pub error_tokens: Vec<Token>
}

impl<'src> Lexer<'src> {
    pub fn new(source: &'src str, filename: String) -> Self {
        Lexer {
            source,
            filename,
            tokens: Vec::new(),

            start: 0,
            current: 0,
            line: 1,
            col: 1,
            col_start: 1,
            col_end: 1,
            had_error: false,
            error_tokens: Vec::new()
        }
    }

    pub fn scan_tokens(&mut self) {
        while !self.is_at_end() {
            self.start = self.current;
            self.col_start = self.col_end;
            self.scan_token();
        }

        self.add_token(TokenKind::Eof);

        let error_tokens: Vec<Token> = self.error_tokens.clone();
        for token in error_tokens.iter() {
            let message = format!("Unexpected token '{}'", token.lexeme);
            self.lexerr(&message, token.clone(), vec![Help::new("Remove this character".to_string(), token.line, token.span.clone(), self.filename.clone())]);
        }
    }

    pub fn scan_token(&mut self) {
        let c = self.advance();
        match c {
            '(' => self.add_token(TokenKind::Lparen),
            ')' => self.add_token(TokenKind::Rparen),
            '{' => self.add_token(TokenKind::Lbrace),
            '}' => self.add_token(TokenKind::Rbrace),
            '[' => self.add_token(TokenKind::Lbracket),
            ']' => self.add_token(TokenKind::Rbracket),
            ',' => self.add_token(TokenKind::Comma),
            '.' => {
                let token_kind = if self.match_token('.') { TokenKind::Range } else { TokenKind::Dot };
                self.add_token(token_kind);
            },
            ':' => {
                self.add_token(TokenKind::Colon);
            },
            ';' => self.add_token(TokenKind::Semicolon),
            '+' => {
                let token_kind = if self.match_token('=') { TokenKind::PlusEq } else { TokenKind::Plus };
                self.add_token(token_kind);
            },
            '-' => {
                let token_kind = if self.match_token('=') {
                    TokenKind::MinusEq
                } else {
                    let next_token = self.match_token('>');
                    if next_token {
                        TokenKind::Arrow
                    } else {
                        TokenKind::Minus
                    }
                };
                self.add_token(token_kind);
            },
            '*' => {
                if self.match_token('=') {
                    self.add_token(TokenKind::StarEq);
                } else if self.match_token('*') {
                    self.add_token(TokenKind::Pow);
                } else {
                    self.add_token(TokenKind::Star);
                }
            },
            '/' => {
                if self.match_token('/') {
                    self.scan_comment();
                } else {
                    let token_kind = if self.match_token('=') { TokenKind::SlashEq } else { TokenKind::Slash };
                    self.add_token(token_kind);
                }
            },
            '%' => {
                let is_match = self.match_token('=');
                let token_kind = if is_match { TokenKind::ModEq } else { TokenKind::Mod };
                self.add_token(token_kind);
            },
            '&' => {
                let token_kind = if self.match_token('=') {
                    TokenKind::AmpEq
                } else if self.match_token('&') {
                    TokenKind::AmpAmp
                } else {
                    TokenKind::Amp
                };
                self.add_token(token_kind);
            },
            '|' => {
                let token_kind = if self.match_token('=') {
                    TokenKind::PipeEq
                } else if self.match_token('|') {
                    TokenKind::PipePipe
                } else {
                    TokenKind::Pipe
                };
                self.add_token(token_kind);
            },
            '^' => {
                let token_kind = if self.match_token('=') { TokenKind::CaretEq } else { TokenKind::Caret };
                self.add_token(token_kind);
            },
            '#' => self.add_token(TokenKind::Hash),
            '!' => {
                let is_match = self.match_token('=');
                let token_kind = if is_match { TokenKind::BangEq } else { TokenKind::Bang };
                self.add_token(token_kind);
            },
            '?' => {
                let is_match = self.match_token('?');
                let token_kind = if is_match { TokenKind::QuestionQuestion } else { TokenKind::Question };
                self.add_token(token_kind);
            },
            '=' => {
                let token_kind = if self.match_token('=') { TokenKind::EqEq } else { TokenKind::Eq };
                self.add_token(token_kind);
            },
            '<' => {
                let token_kind = if self.match_token('=') {
                    TokenKind::LtEq
                } else {
                    TokenKind::Lt
                };
                self.add_token(token_kind);
            },
            '>' => {
                let token_kind = if self.match_token('=') {
                    TokenKind::GtEq
                } else {
                    TokenKind::Gt
                };
                self.add_token(token_kind);
            },
            '"' => self.scan_string(c),
            '\'' => self.scan_string(c),
            ' ' | '\r' | '\t' => (),
            '\n' => {
                self.line += 1;
                self.col_end = 0;
                self.col_start = 0;
            },
            _ => {
                if c.is_digit(10) {
                    self.scan_number();
                } else if c.is_alphabetic() {
                    self.scan_identifier();
                } else {
                    self.add_token(TokenKind::Error);
                }
            }
        }
    }
    
    fn advance(&mut self) -> char {
        let c = self.peek();
        self.current += 1;
        self.col_end += 1;
        c
    }

    fn match_token(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.peek() != expected {
            return false;
        }
        self.current += 1;
        self.col_end += 1;
        true
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source[self.current..].chars().next().unwrap_or('\0')
        }
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            '\0'
        } else {
            self.source[self.current+1..].chars().next().unwrap_or('\0')
        }
    }

    fn add_token(&mut self, kind: TokenKind) {
        let lexeme = self.source[self.start..self.current].to_string();
        let token = Token::new(kind.clone(), lexeme, self.line, Span::new(self.col_start, self.col_end));
        self.tokens.push(token.clone());

        if kind == TokenKind::Error {
            self.error_tokens.push(token);
        }
    }

    fn scan_string(&mut self, delimiter: char) {
        while self.peek() != delimiter && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            self.add_token(TokenKind::Error);
            return;
        }

        self.advance(); // Closing quote
        self.add_token(TokenKind::String);
    }

    fn scan_number(&mut self) {
        let mut is_float = false;
        while self.peek().is_digit(10) {
            self.advance();
        }

        if self.peek() == '.' && self.peek_next().is_digit(10) {
            is_float = true;
            self.advance(); // Consume the '.'
            while self.peek().is_digit(10) {
                self.advance();
            }
        }

        if is_float {
            self.add_token(TokenKind::Float);
        } else {
            self.add_token(TokenKind::Integer);
        }
    }

    fn scan_identifier(&mut self) {
        while self.peek().is_alphanumeric() || self.peek() == '_' {
            self.advance();
        }

        let text = &self.source[self.start..self.current];
        let kind = match text {
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "while" => TokenKind::While,
            "for" => TokenKind::For,
            "break" => TokenKind::Break,
            "continue" => TokenKind::Continue,
            "return" => TokenKind::Return,
            "struct" => TokenKind::Struct,
            "func" => TokenKind::Func,
            "let" => TokenKind::Let,
            "pub" => TokenKind::Pub,
            "priv" => TokenKind::Priv,
            "protected" => TokenKind::Protected,
            "import" => TokenKind::Import,
            "as" => TokenKind::As,
            "extern" => TokenKind::Extern,
            "extend" => TokenKind::Extend,
            "enum" => TokenKind::Enum,
            "match" => TokenKind::Match,
            "case" => TokenKind::Case,
            "trait" => TokenKind::Trait,
            "type" => TokenKind::Type,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "null" => TokenKind::Null,
            "module" => TokenKind::Module,
            "in" => TokenKind::In,
            "_" => TokenKind::Underscore,
            _ => TokenKind::Identifier,
        };
        self.add_token(kind);
    }

    fn scan_comment(&mut self) {
        while self.peek() != '\n' && !self.is_at_end() {
            self.advance();
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn lexerr(&mut self, message: &str, token: Token, help: Vec<Help>) {
        let mut error = Error::new(message.to_string(), token.line, token.span, self.filename.clone());
        error.add_source(self.source.to_string());

        for h in help {
            error.add_help(h);
        }

        eprintln!("{}", error.to_string());

        self.had_error = true;
    }

    pub fn print_tokens(&self) {
        for token in &self.tokens {
            println!("{}", token.to_string());
        }
    }

    /// Sets the start and current offset for lexing a substring, and resets column/line info.
    /// Now also allows setting the line number.
    pub fn set_offset(&mut self, offset: usize, line: usize) {
        self.start = 0;
        self.current = 0;
        self.col = offset + 1;
        self.col_start = offset + 1;
        self.col_end = offset + 1;
        self.line = line;
    }
}