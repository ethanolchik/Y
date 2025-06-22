use crate::frontend::utils::token::{Token, TokenKind, Span};
use std::collections::HashMap;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub struct InterpolatedString {
    pub original: String, // The full string literal
    pub interpolations: Vec<InterpolatedPart>, // All interpolated parts
}

#[derive(Debug, Clone, PartialEq)]
pub struct InterpolatedPart {
    pub expr: String, // The variable or expression inside \(...)
    pub span: Span,   // The position within the string literal (relative to the string)
    pub line: usize, // The line number of the original string token
}

impl InterpolatedString {
    pub fn extract_from_token(token: &Token) -> Option<InterpolatedString> {
        if token.kind != TokenKind::String {
            return None;
        }
        let s = &token.lexeme;
        let mut interpolations = Vec::new();
        let bytes = s.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'\\' && i + 1 < bytes.len() && bytes[i + 1] == b'(' {
                let expr_start = i + 2;
                let mut depth = 1;
                let mut j = expr_start;
                while j < bytes.len() {
                    match bytes[j] {
                        b'(' => depth += 1,
                        b')' => {
                            depth -= 1;
                            if depth == 0 {
                                let expr = &s[expr_start..j];
                                interpolations.push(InterpolatedPart {
                                    expr: expr.to_string(),
                                    span: Span::new(expr_start, j),
                                    line: token.line,
                                });
                                i = j; // Move i to end of interpolation
                                break;
                            }
                        }
                        _ => {}
                    }
                    j += 1;
                }
            }
            i += 1;
        }
        if !interpolations.is_empty() {
            Some(InterpolatedString {
                original: s.clone(),
                interpolations,
            })
        } else {
            None
        }
    }

    /// Tokenizes each interpolated expression using the provided lexer function,
    /// and offsets the resulting token spans by the start of the interpolation in the string.
    /// The lexer_fn should take (expr: &str, offset: usize) -> Vec<Token>
    pub fn tokenize_interpolations<F>(&self, mut lexer_fn: F) -> Vec<Vec<Token>>
    where
        F: FnMut(&str, usize) -> Vec<Token>,
    {
        self.interpolations
            .iter()
            .map(|part| {
                let offset = part.span.start;
                lexer_fn(&part.expr, offset)
            })
            .collect()
    }
}

impl Display for InterpolatedString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "InterpolatedString {{")?;
        writeln!(f, "  original: {:?}", self.original)?;
        writeln!(f, "  interpolations: [")?;
        for part in &self.interpolations {
            writeln!(f, "    {{ expr: {:?}, span: {:?} }},", part.expr, part.span)?;
        }
        writeln!(f, "  ]")?;
        write!(f, "}}")
    }
}

/// Parses a list of tokens and returns a HashMap mapping string token spans to their InterpolatedString info.
pub fn extract_interpolated_strings(tokens: &[Token]) -> HashMap<Span, InterpolatedString> {
    let mut map = HashMap::new();
    for token in tokens {
        if let Some(interp) = InterpolatedString::extract_from_token(token) {
            map.insert(token.span.clone(), interp);
        }
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::utils::token::{Token, TokenKind, Span};
    use crate::frontend::lexer::Lexer;

    #[test]
    fn test_parse_interpolated_strings() {
        let token = Token {
            kind: TokenKind::String,
            lexeme: "Hello, \\(name)!".to_string(),
            line: 1,
            span: Span::new(0, 15),
        };
        let tokens = vec![token.clone()];
        let map = extract_interpolated_strings(&tokens);
        assert!(map.contains_key(&token.span));
        let interp = map.get(&token.span).unwrap();
        assert_eq!(interp.original, "Hello, \\(name)!");
        assert_eq!(interp.interpolations.len(), 1);
        assert_eq!(interp.interpolations[0].expr, "name");

        // Test the tokenization with span offset
        let tokenised_offset = interp.tokenize_interpolations(|expr, offset| {
            let mut sublexer = Lexer::new(expr, "test".to_string());
            // Use the line number of the parent string token for context (default to 1 if not available)
            // If you want to use the line of the parent string token, you can pass it here
            sublexer.set_offset(offset, 1);
            sublexer.scan_tokens();
            // Remove EOF token if present
            if let Some(Token { kind: TokenKind::Eof, .. }) = sublexer.tokens.last() {
                sublexer.tokens.pop();
            }
            sublexer.tokens
        });
        assert_eq!(tokenised_offset.len(), 1);
        assert_eq!(tokenised_offset[0].len(), 1);
        assert_eq!(tokenised_offset[0][0].kind, TokenKind::Identifier);
        assert_eq!(tokenised_offset[0][0].lexeme, "name");
        assert_eq!(tokenised_offset[0][0].line, 1);
        assert_eq!(tokenised_offset[0][0].span, Span::new(10, 14));
    }
}
