use crate::span::Span;

/// Every kind of token the lexer can produce.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntaxKind {
    // === Literals ===
    /// Integer literal: `42`
    IntLit,
    /// Float literal: `3.14`
    FloatLit,
    /// Length with unit: `10mm`, `5cm`, `2.5in`, `1m`
    LengthLit,
    /// Angle with unit: `45deg`, `0.5rad`
    AngleLit,
    /// String literal: `"hello"`
    StringLit,
    /// `true`
    True,
    /// `false`
    False,

    // === Identifiers ===
    /// Variable or function name
    Ident,

    // === Keywords ===
    /// `let`
    Let,
    /// `data`
    Data,
    /// `fn`
    Fn,
    /// `enum`
    Enum,
    /// `if`
    If,
    /// `else`
    Else,
    /// `match`
    Match,
    /// `with`
    With,

    // === Operators ===
    /// `+`
    Plus,
    /// `-`
    Minus,
    /// `*`
    Star,
    /// `/`
    Slash,
    /// `=`
    Eq,
    /// `==`
    EqEq,
    /// `!=`
    BangEq,
    /// `<`
    Lt,
    /// `<=`
    LtEq,
    /// `>`
    Gt,
    /// `>=`
    GtEq,
    /// `&&`
    AmpAmp,
    /// `||`
    PipePipe,
    /// `!`
    Bang,
    /// `|>`
    PipeGt,
    /// `|` (lambda parameter delimiter)
    Pipe,
    /// `.`
    Dot,
    /// `->`
    Arrow,
    /// `:`
    Colon,
    /// `,`
    Comma,
    /// `=>`
    FatArrow,
    /// `;`
    Semicolon,

    // === Delimiters ===
    /// `(`
    LParen,
    /// `)`
    RParen,
    /// `{`
    LBrace,
    /// `}`
    RBrace,
    /// `[`
    LBracket,
    /// `]`
    RBracket,

    // === Special ===
    /// Newline (may be significant for statement separation)
    Newline,
    /// End of file
    Eof,
    /// Invalid token (lexer error recovery)
    Error,
}

impl SyntaxKind {
    /// Look up a keyword by its string. Returns `None` for non-keywords.
    pub fn keyword(s: &str) -> Option<SyntaxKind> {
        match s {
            "let" => Some(SyntaxKind::Let),
            "data" => Some(SyntaxKind::Data),
            "fn" => Some(SyntaxKind::Fn),
            "enum" => Some(SyntaxKind::Enum),
            "if" => Some(SyntaxKind::If),
            "else" => Some(SyntaxKind::Else),
            "match" => Some(SyntaxKind::Match),
            "with" => Some(SyntaxKind::With),
            "true" => Some(SyntaxKind::True),
            "false" => Some(SyntaxKind::False),
            _ => None,
        }
    }

    /// Human-readable name for this token kind.
    pub fn name(self) -> &'static str {
        match self {
            Self::IntLit => "integer literal",
            Self::FloatLit => "float literal",
            Self::LengthLit => "length literal",
            Self::AngleLit => "angle literal",
            Self::StringLit => "string literal",
            Self::True => "'true'",
            Self::False => "'false'",
            Self::Ident => "identifier",
            Self::Let => "'let'",
            Self::Data => "'data'",
            Self::Fn => "'fn'",
            Self::Enum => "'enum'",
            Self::If => "'if'",
            Self::Else => "'else'",
            Self::Match => "'match'",
            Self::With => "'with'",
            Self::Plus => "'+'",
            Self::Minus => "'-'",
            Self::Star => "'*'",
            Self::Slash => "'/'",
            Self::Eq => "'='",
            Self::EqEq => "'=='",
            Self::BangEq => "'!='",
            Self::Lt => "'<'",
            Self::LtEq => "'<='",
            Self::Gt => "'>'",
            Self::GtEq => "'>='",
            Self::AmpAmp => "'&&'",
            Self::PipePipe => "'||'",
            Self::Bang => "'!'",
            Self::PipeGt => "'|>'",
            Self::Pipe => "'|'",
            Self::Dot => "'.'",
            Self::Arrow => "'->'",
            Self::Colon => "':'",
            Self::Comma => "','",
            Self::FatArrow => "'=>'",
            Self::Semicolon => "';'",
            Self::LParen => "'('",
            Self::RParen => "')'",
            Self::LBrace => "'{'",
            Self::RBrace => "'}'",
            Self::LBracket => "'['",
            Self::RBracket => "']'",
            Self::Newline => "newline",
            Self::Eof => "end of file",
            Self::Error => "error",
        }
    }
}

/// A token produced by the lexer.
///
/// The actual text is recovered from the source via the span.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: SyntaxKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: SyntaxKind, span: Span) -> Self {
        Self { kind, span }
    }

    /// Extract the text this token covers from the original source.
    pub fn text<'a>(&self, source: &'a str) -> &'a str {
        &source[self.span.start as usize..self.span.end as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyword_lookup() {
        assert_eq!(SyntaxKind::keyword("let"), Some(SyntaxKind::Let));
        assert_eq!(SyntaxKind::keyword("fn"), Some(SyntaxKind::Fn));
        assert_eq!(SyntaxKind::keyword("true"), Some(SyntaxKind::True));
        assert_eq!(SyntaxKind::keyword("false"), Some(SyntaxKind::False));
        assert_eq!(SyntaxKind::keyword("foo"), None);
        assert_eq!(SyntaxKind::keyword("ISO_METRIC"), None);
    }

    #[test]
    fn token_text_extraction() {
        let source = "let x = 42";
        let token = Token::new(SyntaxKind::Let, Span::new(0, 3));
        assert_eq!(token.text(source), "let");
    }
}
