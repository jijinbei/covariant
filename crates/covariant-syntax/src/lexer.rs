use crate::error::{ErrorKind, SyntaxError};
use crate::span::Span;
use crate::token::{SyntaxKind, Token};

/// Lex the entire source string, returning tokens and any errors.
pub fn lex(source: &str) -> (Vec<Token>, Vec<SyntaxError>) {
    let mut lexer = Lexer::new(source);
    lexer.run();
    (lexer.tokens, lexer.errors)
}

struct Lexer<'src> {
    source: &'src str,
    bytes: &'src [u8],
    pos: u32,
    tokens: Vec<Token>,
    errors: Vec<SyntaxError>,
}

impl<'src> Lexer<'src> {
    fn new(source: &'src str) -> Self {
        Self {
            source,
            bytes: source.as_bytes(),
            pos: 0,
            tokens: Vec::new(),
            errors: Vec::new(),
        }
    }

    fn run(&mut self) {
        while !self.at_end() {
            self.scan_token();
        }
        self.tokens
            .push(Token::new(SyntaxKind::Eof, Span::point(self.pos)));
    }

    // --- Character access ---

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos as usize).copied()
    }

    fn peek_next(&self) -> Option<u8> {
        self.bytes.get(self.pos as usize + 1).copied()
    }

    fn advance(&mut self) -> u8 {
        let b = self.bytes[self.pos as usize];
        self.pos += 1;
        b
    }

    fn at_end(&self) -> bool {
        self.pos as usize >= self.bytes.len()
    }

    // --- Token scanning ---

    fn scan_token(&mut self) {
        self.skip_whitespace();
        if self.at_end() {
            return;
        }

        let start = self.pos;
        let c = self.advance();

        match c {
            b'(' => self.emit(SyntaxKind::LParen, start),
            b')' => self.emit(SyntaxKind::RParen, start),
            b'{' => self.emit(SyntaxKind::LBrace, start),
            b'}' => self.emit(SyntaxKind::RBrace, start),
            b'[' => self.emit(SyntaxKind::LBracket, start),
            b']' => self.emit(SyntaxKind::RBracket, start),
            b'+' => self.emit(SyntaxKind::Plus, start),
            b'*' => self.emit(SyntaxKind::Star, start),
            b'.' => self.emit(SyntaxKind::Dot, start),
            b':' => self.emit(SyntaxKind::Colon, start),
            b',' => self.emit(SyntaxKind::Comma, start),
            b';' => self.emit(SyntaxKind::Semicolon, start),
            b'\n' => self.emit(SyntaxKind::Newline, start),
            b'-' => {
                if self.peek() == Some(b'>') {
                    self.advance();
                    self.emit(SyntaxKind::Arrow, start);
                } else {
                    self.emit(SyntaxKind::Minus, start);
                }
            }
            b'=' => {
                if self.peek() == Some(b'=') {
                    self.advance();
                    self.emit(SyntaxKind::EqEq, start);
                } else if self.peek() == Some(b'>') {
                    self.advance();
                    self.emit(SyntaxKind::FatArrow, start);
                } else {
                    self.emit(SyntaxKind::Eq, start);
                }
            }
            b'!' => {
                if self.peek() == Some(b'=') {
                    self.advance();
                    self.emit(SyntaxKind::BangEq, start);
                } else {
                    self.emit(SyntaxKind::Bang, start);
                }
            }
            b'<' => {
                if self.peek() == Some(b'=') {
                    self.advance();
                    self.emit(SyntaxKind::LtEq, start);
                } else {
                    self.emit(SyntaxKind::Lt, start);
                }
            }
            b'>' => {
                if self.peek() == Some(b'=') {
                    self.advance();
                    self.emit(SyntaxKind::GtEq, start);
                } else {
                    self.emit(SyntaxKind::Gt, start);
                }
            }
            b'&' => {
                if self.peek() == Some(b'&') {
                    self.advance();
                    self.emit(SyntaxKind::AmpAmp, start);
                } else {
                    self.error("expected '&&'", start, ErrorKind::UnexpectedChar);
                }
            }
            b'|' => {
                if self.peek() == Some(b'>') {
                    self.advance();
                    self.emit(SyntaxKind::PipeGt, start);
                } else if self.peek() == Some(b'|') {
                    self.advance();
                    self.emit(SyntaxKind::PipePipe, start);
                } else {
                    self.emit(SyntaxKind::Pipe, start);
                }
            }
            b'/' => {
                if self.peek() == Some(b'/') {
                    self.skip_line_comment();
                } else if self.peek() == Some(b'*') {
                    self.skip_block_comment(start);
                } else {
                    self.emit(SyntaxKind::Slash, start);
                }
            }
            b'"' => self.scan_string(start),
            c if c.is_ascii_digit() => self.scan_number(start),
            c if c.is_ascii_alphabetic() || c == b'_' => self.scan_ident_or_keyword(start),
            _ => self.error(
                &format!("unexpected character '{}'", c as char),
                start,
                ErrorKind::UnexpectedChar,
            ),
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c == b' ' || c == b'\t' || c == b'\r' {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_line_comment(&mut self) {
        // Already consumed the first '/', consume the second '/'
        self.advance();
        while let Some(c) = self.peek() {
            if c == b'\n' {
                break;
            }
            self.advance();
        }
    }

    fn skip_block_comment(&mut self, start: u32) {
        // Already consumed '/', consume '*'
        self.advance();
        let mut depth: u32 = 1;

        while !self.at_end() && depth > 0 {
            if self.peek() == Some(b'/') && self.peek_next() == Some(b'*') {
                self.advance();
                self.advance();
                depth += 1;
            } else if self.peek() == Some(b'*') && self.peek_next() == Some(b'/') {
                self.advance();
                self.advance();
                depth -= 1;
            } else {
                self.advance();
            }
        }

        if depth > 0 {
            self.error(
                "unterminated block comment",
                start,
                ErrorKind::UnterminatedBlockComment,
            );
        }
    }

    fn scan_number(&mut self, start: u32) {
        while self.peek().is_some_and(|c| c.is_ascii_digit()) {
            self.advance();
        }

        // Check for decimal point (not followed by '.' for range or non-digit)
        let is_float = self.peek() == Some(b'.')
            && self.peek_next().is_some_and(|c| c.is_ascii_digit());

        if is_float {
            self.advance(); // consume '.'
            while self.peek().is_some_and(|c| c.is_ascii_digit()) {
                self.advance();
            }
        }

        // Check for unit suffix
        let suffix_start = self.pos;
        if self.peek().is_some_and(|c| c.is_ascii_alphabetic()) {
            while self.peek().is_some_and(|c| c.is_ascii_alphabetic()) {
                self.advance();
            }

            // Only treat as unit if followed by non-identifier char
            // (prevents matching "10min" as "10" + "m" + "in")
            let suffix = &self.source[suffix_start as usize..self.pos as usize];
            match suffix {
                "mm" | "cm" | "m" | "in" => self.emit(SyntaxKind::LengthLit, start),
                "deg" | "rad" => self.emit(SyntaxKind::AngleLit, start),
                _ => {
                    // Not a valid unit: rewind and emit number, then let ident scan pick up
                    self.pos = suffix_start;
                    self.emit_number(is_float, start);
                }
            }
        } else {
            self.emit_number(is_float, start);
        }
    }

    fn emit_number(&mut self, is_float: bool, start: u32) {
        if is_float {
            self.emit(SyntaxKind::FloatLit, start);
        } else {
            self.emit(SyntaxKind::IntLit, start);
        }
    }

    fn scan_string(&mut self, start: u32) {
        while let Some(c) = self.peek() {
            if c == b'"' {
                self.advance();
                self.emit(SyntaxKind::StringLit, start);
                return;
            }
            if c == b'\\' {
                self.advance(); // skip escape backslash
                if !self.at_end() {
                    self.advance(); // skip escaped char
                }
            } else {
                self.advance();
            }
        }

        self.error(
            "unterminated string literal",
            start,
            ErrorKind::UnterminatedString,
        );
    }

    fn scan_ident_or_keyword(&mut self, start: u32) {
        while self
            .peek()
            .is_some_and(|c| c.is_ascii_alphanumeric() || c == b'_')
        {
            self.advance();
        }

        let text = &self.source[start as usize..self.pos as usize];
        let kind = SyntaxKind::keyword(text).unwrap_or(SyntaxKind::Ident);
        self.emit(kind, start);
    }

    // --- Helpers ---

    fn emit(&mut self, kind: SyntaxKind, start: u32) {
        self.tokens
            .push(Token::new(kind, Span::new(start, self.pos)));
    }

    fn error(&mut self, message: &str, start: u32, kind: ErrorKind) {
        self.errors
            .push(SyntaxError::new(message, Span::new(start, self.pos), kind));
        self.tokens
            .push(Token::new(SyntaxKind::Error, Span::new(start, self.pos)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lex_kinds(source: &str) -> Vec<SyntaxKind> {
        let (tokens, errors) = lex(source);
        assert!(errors.is_empty(), "unexpected errors: {errors:?}");
        tokens
            .iter()
            .filter(|t| t.kind != SyntaxKind::Eof && t.kind != SyntaxKind::Newline)
            .map(|t| t.kind)
            .collect()
    }

    fn lex_kinds_with_newlines(source: &str) -> Vec<SyntaxKind> {
        let (tokens, errors) = lex(source);
        assert!(errors.is_empty(), "unexpected errors: {errors:?}");
        tokens
            .iter()
            .filter(|t| t.kind != SyntaxKind::Eof)
            .map(|t| t.kind)
            .collect()
    }

    #[test]
    fn single_char_operators() {
        assert_eq!(
            lex_kinds("+ - * /"),
            vec![
                SyntaxKind::Plus,
                SyntaxKind::Minus,
                SyntaxKind::Star,
                SyntaxKind::Slash
            ]
        );
    }

    #[test]
    fn two_char_operators() {
        assert_eq!(
            lex_kinds("== != <= >= && || |>"),
            vec![
                SyntaxKind::EqEq,
                SyntaxKind::BangEq,
                SyntaxKind::LtEq,
                SyntaxKind::GtEq,
                SyntaxKind::AmpAmp,
                SyntaxKind::PipePipe,
                SyntaxKind::PipeGt,
            ]
        );
    }

    #[test]
    fn arrow_and_fat_arrow() {
        assert_eq!(
            lex_kinds("-> =>"),
            vec![SyntaxKind::Arrow, SyntaxKind::FatArrow]
        );
    }

    #[test]
    fn delimiters() {
        assert_eq!(
            lex_kinds("(){}[]"),
            vec![
                SyntaxKind::LParen,
                SyntaxKind::RParen,
                SyntaxKind::LBrace,
                SyntaxKind::RBrace,
                SyntaxKind::LBracket,
                SyntaxKind::RBracket,
            ]
        );
    }

    #[test]
    fn integer_literal() {
        assert_eq!(lex_kinds("42"), vec![SyntaxKind::IntLit]);
    }

    #[test]
    fn float_literal() {
        assert_eq!(lex_kinds("3.14"), vec![SyntaxKind::FloatLit]);
    }

    #[test]
    fn length_literals() {
        assert_eq!(
            lex_kinds("10mm 5cm 2.5in 1m"),
            vec![
                SyntaxKind::LengthLit,
                SyntaxKind::LengthLit,
                SyntaxKind::LengthLit,
                SyntaxKind::LengthLit,
            ]
        );
    }

    #[test]
    fn angle_literals() {
        assert_eq!(
            lex_kinds("45deg 0.5rad"),
            vec![SyntaxKind::AngleLit, SyntaxKind::AngleLit]
        );
    }

    #[test]
    fn string_literal() {
        assert_eq!(lex_kinds("\"hello\""), vec![SyntaxKind::StringLit]);
    }

    #[test]
    fn string_with_escape() {
        assert_eq!(lex_kinds("\"he\\\"llo\""), vec![SyntaxKind::StringLit]);
    }

    #[test]
    fn keywords() {
        assert_eq!(
            lex_kinds("let data fn enum if else match with"),
            vec![
                SyntaxKind::Let,
                SyntaxKind::Data,
                SyntaxKind::Fn,
                SyntaxKind::Enum,
                SyntaxKind::If,
                SyntaxKind::Else,
                SyntaxKind::Match,
                SyntaxKind::With,
            ]
        );
    }

    #[test]
    fn booleans() {
        assert_eq!(
            lex_kinds("true false"),
            vec![SyntaxKind::True, SyntaxKind::False]
        );
    }

    #[test]
    fn identifiers() {
        assert_eq!(
            lex_kinds("foo bar ISO_METRIC M3"),
            vec![
                SyntaxKind::Ident,
                SyntaxKind::Ident,
                SyntaxKind::Ident,
                SyntaxKind::Ident,
            ]
        );
    }

    #[test]
    fn line_comment() {
        assert_eq!(
            lex_kinds_with_newlines("42 // comment\n10"),
            vec![SyntaxKind::IntLit, SyntaxKind::Newline, SyntaxKind::IntLit]
        );
    }

    #[test]
    fn block_comment() {
        assert_eq!(
            lex_kinds("42 /* comment */ 10"),
            vec![SyntaxKind::IntLit, SyntaxKind::IntLit]
        );
    }

    #[test]
    fn nested_block_comment() {
        assert_eq!(lex_kinds("/* outer /* inner */ still */"), vec![]);
    }

    #[test]
    fn let_statement() {
        assert_eq!(
            lex_kinds("let x = 10mm"),
            vec![
                SyntaxKind::Let,
                SyntaxKind::Ident,
                SyntaxKind::Eq,
                SyntaxKind::LengthLit,
            ]
        );
    }

    #[test]
    fn function_call() {
        assert_eq!(
            lex_kinds("box(vec3(80mm, 50mm, 5mm))"),
            vec![
                SyntaxKind::Ident,
                SyntaxKind::LParen,
                SyntaxKind::Ident,
                SyntaxKind::LParen,
                SyntaxKind::LengthLit,
                SyntaxKind::Comma,
                SyntaxKind::LengthLit,
                SyntaxKind::Comma,
                SyntaxKind::LengthLit,
                SyntaxKind::RParen,
                SyntaxKind::RParen,
            ]
        );
    }

    #[test]
    fn pipe_chain() {
        assert_eq!(
            lex_kinds("x |> f |> g"),
            vec![
                SyntaxKind::Ident,
                SyntaxKind::PipeGt,
                SyntaxKind::Ident,
                SyntaxKind::PipeGt,
                SyntaxKind::Ident,
            ]
        );
    }

    #[test]
    fn lambda_delimiter() {
        assert_eq!(
            lex_kinds("|x| x + 1"),
            vec![
                SyntaxKind::Pipe,
                SyntaxKind::Ident,
                SyntaxKind::Pipe,
                SyntaxKind::Ident,
                SyntaxKind::Plus,
                SyntaxKind::IntLit,
            ]
        );
    }

    #[test]
    fn error_unterminated_string() {
        let (_, errors) = lex("\"unterminated");
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].kind, ErrorKind::UnterminatedString);
    }

    #[test]
    fn error_unexpected_char() {
        let (_, errors) = lex("@");
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].kind, ErrorKind::UnexpectedChar);
    }

    #[test]
    fn error_unterminated_block_comment() {
        let (_, errors) = lex("/* no end");
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].kind, ErrorKind::UnterminatedBlockComment);
    }

    #[test]
    fn number_followed_by_unknown_suffix() {
        // "10min" should be parsed as IntLit(10) + Ident(min)
        assert_eq!(
            lex_kinds("10min"),
            vec![SyntaxKind::IntLit, SyntaxKind::Ident]
        );
    }

    #[test]
    fn semicolons() {
        assert_eq!(
            lex_kinds("a; b"),
            vec![SyntaxKind::Ident, SyntaxKind::Semicolon, SyntaxKind::Ident]
        );
    }

    #[test]
    fn eof_token_always_present() {
        let (tokens, _) = lex("");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, SyntaxKind::Eof);
    }
}
