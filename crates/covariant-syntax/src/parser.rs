use crate::ast::*;
use crate::error::{ErrorKind, SyntaxError};
use crate::span::{Span, Spanned};
use crate::token::{SyntaxKind, Token};

/// Parse source code and tokens into an AST.
pub fn parse(source: &str, tokens: Vec<Token>) -> (SourceFile, Vec<SyntaxError>) {
    let mut parser = Parser::new(source, tokens);
    let file = parser.parse_file();
    (file, parser.errors)
}

struct Parser<'src> {
    source: &'src str,
    tokens: Vec<Token>,
    pos: usize,
    errors: Vec<SyntaxError>,
}

// ======== Utilities ========

impl<'src> Parser<'src> {
    fn new(source: &'src str, tokens: Vec<Token>) -> Self {
        Self {
            source,
            tokens,
            pos: 0,
            errors: Vec::new(),
        }
    }

    fn peek(&self) -> SyntaxKind {
        self.current_token().kind
    }

    fn current_token(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or_else(|| {
            self.tokens
                .last()
                .expect("token stream should contain at least Eof")
        })
    }

    fn advance(&mut self) -> Token {
        let tok = self.current_token().clone();
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        tok
    }

    fn at(&self, kind: SyntaxKind) -> bool {
        self.peek() == kind
    }

    fn at_end(&self) -> bool {
        self.peek() == SyntaxKind::Eof
    }

    fn expect(&mut self, kind: SyntaxKind) -> Result<Token, ()> {
        if self.at(kind) {
            Ok(self.advance())
        } else {
            let tok = self.current_token().clone();
            self.push_error(
                format!("expected {}, found {}", kind.name(), tok.kind.name()),
                tok.span,
                ErrorKind::ExpectedToken,
            );
            Err(())
        }
    }

    fn text(&self, token: &Token) -> &'src str {
        token.text(self.source)
    }

    fn skip_newlines(&mut self) {
        while self.at(SyntaxKind::Newline) {
            self.advance();
        }
    }

    fn skip_trivia(&mut self) {
        while matches!(self.peek(), SyntaxKind::Newline | SyntaxKind::Error) {
            self.advance();
        }
    }

    fn push_error(&mut self, message: impl Into<String>, span: Span, kind: ErrorKind) {
        self.errors.push(SyntaxError::new(message, span, kind));
    }

    /// Skip tokens until we reach a likely statement boundary.
    #[allow(dead_code)]
    fn synchronize(&mut self) {
        loop {
            match self.peek() {
                SyntaxKind::Eof => break,
                SyntaxKind::Let
                | SyntaxKind::Fn
                | SyntaxKind::Data
                | SyntaxKind::Enum
                | SyntaxKind::RBrace => break,
                SyntaxKind::Newline | SyntaxKind::Semicolon => {
                    self.advance();
                    break;
                }
                _ => {
                    self.advance();
                }
            }
        }
    }
}

// ======== Top-level ========

impl Parser<'_> {
    fn parse_file(&mut self) -> SourceFile {
        let start = self.current_token().span;
        let mut stmts = Vec::new();

        self.skip_trivia();
        while !self.at_end() {
            if let Some(stmt) = self.parse_stmt() {
                stmts.push(stmt);
            }
            self.skip_trivia();
            // Skip optional semicolons/newlines between statements
            while matches!(self.peek(), SyntaxKind::Semicolon | SyntaxKind::Newline) {
                self.advance();
            }
        }

        let end = self.current_token().span;
        SourceFile {
            stmts,
            span: start.merge(end),
        }
    }
}

// ======== Statements ========

impl Parser<'_> {
    fn parse_stmt(&mut self) -> Option<Spanned<Stmt>> {
        self.skip_newlines();
        if self.at_end() {
            return None;
        }

        match self.peek() {
            SyntaxKind::Let => Some(self.parse_let_stmt()),
            SyntaxKind::Fn => Some(self.parse_fn_def()),
            SyntaxKind::Data => Some(self.parse_data_def()),
            SyntaxKind::Enum => Some(self.parse_enum_def()),
            _ => {
                let expr = self.parse_expr();
                let span = expr.span;
                Some(Spanned::new(Stmt::Expr(expr), span))
            }
        }
    }

    fn parse_let_stmt(&mut self) -> Spanned<Stmt> {
        let let_tok = self.advance(); // consume 'let'
        self.skip_newlines();

        let name_tok = self.advance();
        let name = Spanned::new(self.text(&name_tok).to_string(), name_tok.span);

        self.skip_newlines();

        // Optional type annotation
        let ty = if self.at(SyntaxKind::Colon) {
            self.advance();
            self.skip_newlines();
            Some(self.parse_type())
        } else {
            None
        };

        self.skip_newlines();
        let _ = self.expect(SyntaxKind::Eq);
        self.skip_newlines();

        let value = self.parse_expr();
        let span = let_tok.span.merge(value.span);

        Spanned::new(Stmt::Let(LetStmt { name, ty, value }), span)
    }

    fn parse_fn_def(&mut self) -> Spanned<Stmt> {
        let fn_tok = self.advance(); // consume 'fn'
        self.skip_newlines();

        let name_tok = self.advance();
        let name = Spanned::new(self.text(&name_tok).to_string(), name_tok.span);

        self.skip_newlines();
        let _ = self.expect(SyntaxKind::LParen);
        let params = self.parse_param_list();
        let _ = self.expect(SyntaxKind::RParen);

        self.skip_newlines();

        // Optional return type
        let return_ty = if self.at(SyntaxKind::Arrow) {
            self.advance();
            self.skip_newlines();
            Some(self.parse_type())
        } else {
            None
        };

        self.skip_newlines();
        let body = self.parse_block_expr();
        let span = fn_tok.span.merge(body.span);

        Spanned::new(
            Stmt::FnDef(FnDef {
                name,
                params,
                return_ty,
                body,
            }),
            span,
        )
    }

    fn parse_data_def(&mut self) -> Spanned<Stmt> {
        let data_tok = self.advance(); // consume 'data'
        self.skip_newlines();

        let name_tok = self.advance();
        let name = Spanned::new(self.text(&name_tok).to_string(), name_tok.span);

        self.skip_newlines();
        let _ = self.expect(SyntaxKind::LBrace);
        let fields = self.parse_field_list();
        let rbrace = self.expect(SyntaxKind::RBrace);
        let end_span = rbrace.map_or(name.span, |t| t.span);

        Spanned::new(
            Stmt::DataDef(DataDef { name, fields }),
            data_tok.span.merge(end_span),
        )
    }

    fn parse_enum_def(&mut self) -> Spanned<Stmt> {
        let enum_tok = self.advance(); // consume 'enum'
        self.skip_newlines();

        let name_tok = self.advance();
        let name = Spanned::new(self.text(&name_tok).to_string(), name_tok.span);

        self.skip_newlines();
        let _ = self.expect(SyntaxKind::LBrace);

        let mut variants = Vec::new();
        self.skip_newlines();
        while !self.at(SyntaxKind::RBrace) && !self.at_end() {
            let var_tok = self.advance();
            variants.push(Spanned::new(self.text(&var_tok).to_string(), var_tok.span));
            self.skip_newlines();
            if self.at(SyntaxKind::Comma) {
                self.advance();
            }
            self.skip_newlines();
        }

        let rbrace = self.expect(SyntaxKind::RBrace);
        let end_span = rbrace.map_or(name.span, |t| t.span);

        Spanned::new(
            Stmt::EnumDef(EnumDef { name, variants }),
            enum_tok.span.merge(end_span),
        )
    }
}

// ======== Expressions ========

impl Parser<'_> {
    fn parse_expr(&mut self) -> Spanned<Expr> {
        self.parse_expr_bp(0)
    }

    /// Pratt parser / precedence climbing.
    fn parse_expr_bp(&mut self, min_bp: u8) -> Spanned<Expr> {
        self.skip_newlines();

        // Prefix operators
        let mut lhs = if let Some(bp) = prefix_bp(self.peek()) {
            let op_tok = self.advance();
            let op_kind = match op_tok.kind {
                SyntaxKind::Minus => UnaryOpKind::Neg,
                SyntaxKind::Bang => UnaryOpKind::Not,
                _ => unreachable!(),
            };
            let operand = self.parse_expr_bp(bp);
            let span = op_tok.span.merge(operand.span);
            Spanned::new(
                Expr::UnaryOp {
                    op: Spanned::new(op_kind, op_tok.span),
                    operand: Box::new(operand),
                },
                span,
            )
        } else {
            self.parse_primary()
        };

        loop {
            self.skip_newlines();

            // Postfix: function calls and field access
            lhs = self.parse_postfix(lhs);

            self.skip_newlines();

            // with-update: `expr with { field = value }`
            if self.at(SyntaxKind::With) {
                self.advance();
                self.skip_newlines();
                let _ = self.expect(SyntaxKind::LBrace);
                let updates = self.parse_field_init_list();
                let rbrace = self.expect(SyntaxKind::RBrace);
                let end_span = rbrace.map_or(lhs.span, |t| t.span);
                let span = lhs.span.merge(end_span);
                lhs = Spanned::new(
                    Expr::WithUpdate {
                        base: Box::new(lhs),
                        updates,
                    },
                    span,
                );
                continue;
            }

            // Infix operators
            let Some((l_bp, r_bp)) = infix_bp(self.peek()) else {
                break;
            };

            if l_bp < min_bp {
                break;
            }

            let op_tok = self.advance();
            let op_kind = match op_tok.kind {
                SyntaxKind::Plus => BinOpKind::Add,
                SyntaxKind::Minus => BinOpKind::Sub,
                SyntaxKind::Star => BinOpKind::Mul,
                SyntaxKind::Slash => BinOpKind::Div,
                SyntaxKind::EqEq => BinOpKind::Eq,
                SyntaxKind::BangEq => BinOpKind::Neq,
                SyntaxKind::Lt => BinOpKind::Lt,
                SyntaxKind::LtEq => BinOpKind::Leq,
                SyntaxKind::Gt => BinOpKind::Gt,
                SyntaxKind::GtEq => BinOpKind::Geq,
                SyntaxKind::AmpAmp => BinOpKind::And,
                SyntaxKind::PipePipe => BinOpKind::Or,
                SyntaxKind::PipeGt => BinOpKind::Pipe,
                _ => unreachable!(),
            };

            self.skip_newlines();
            let rhs = self.parse_expr_bp(r_bp);
            let span = lhs.span.merge(rhs.span);
            lhs = Spanned::new(
                Expr::BinOp {
                    lhs: Box::new(lhs),
                    op: Spanned::new(op_kind, op_tok.span),
                    rhs: Box::new(rhs),
                },
                span,
            );
        }

        lhs
    }

    fn parse_primary(&mut self) -> Spanned<Expr> {
        match self.peek() {
            SyntaxKind::IntLit
            | SyntaxKind::FloatLit
            | SyntaxKind::LengthLit
            | SyntaxKind::AngleLit => self.parse_number_lit(),
            SyntaxKind::StringLit => self.parse_string_lit(),
            SyntaxKind::True => {
                let t = self.advance();
                Spanned::new(Expr::BoolLit(true), t.span)
            }
            SyntaxKind::False => {
                let t = self.advance();
                Spanned::new(Expr::BoolLit(false), t.span)
            }
            SyntaxKind::Ident => self.parse_ident_or_constructor(),
            SyntaxKind::LParen => self.parse_grouped_expr(),
            SyntaxKind::LBracket => self.parse_list_expr(),
            SyntaxKind::LBrace => self.parse_block_expr(),
            SyntaxKind::If => self.parse_if_expr(),
            SyntaxKind::Match => self.parse_match_expr(),
            SyntaxKind::Pipe => self.parse_lambda_expr(),
            _ => {
                let tok = self.advance();
                self.push_error(
                    format!("expected expression, found {}", tok.kind.name()),
                    tok.span,
                    ErrorKind::ExpectedExpr,
                );
                Spanned::new(Expr::IntLit(0), tok.span) // placeholder
            }
        }
    }

    fn parse_postfix(&mut self, mut lhs: Spanned<Expr>) -> Spanned<Expr> {
        loop {
            match self.peek() {
                SyntaxKind::LParen => {
                    self.advance(); // consume '('
                    let args = self.parse_arg_list();
                    let rparen = self.expect(SyntaxKind::RParen);
                    let end_span = rparen.map_or(lhs.span, |t| t.span);
                    let span = lhs.span.merge(end_span);
                    lhs = Spanned::new(
                        Expr::FnCall {
                            func: Box::new(lhs),
                            args,
                        },
                        span,
                    );
                }
                SyntaxKind::Dot => {
                    self.advance(); // consume '.'
                    let field_tok = self.advance();
                    let field = Spanned::new(self.text(&field_tok).to_string(), field_tok.span);
                    let span = lhs.span.merge(field_tok.span);
                    lhs = Spanned::new(
                        Expr::FieldAccess {
                            object: Box::new(lhs),
                            field,
                        },
                        span,
                    );
                }
                _ => break,
            }
        }
        lhs
    }

    fn parse_ident_or_constructor(&mut self) -> Spanned<Expr> {
        let name_tok = self.advance();
        let name = self.text(&name_tok).to_string();
        let name_span = name_tok.span;

        // Data constructor: UpperCaseName { field = value, ... }
        if self.at(SyntaxKind::LBrace) && name.starts_with(|c: char| c.is_uppercase()) {
            self.advance(); // consume '{'
            let fields = self.parse_field_init_list();
            let rbrace = self.expect(SyntaxKind::RBrace);
            let end_span = rbrace.map_or(name_span, |t| t.span);
            return Spanned::new(
                Expr::DataConstructor {
                    name: Spanned::new(name, name_span),
                    fields,
                },
                name_span.merge(end_span),
            );
        }

        Spanned::new(Expr::Ident(name), name_span)
    }

    fn parse_number_lit(&mut self) -> Spanned<Expr> {
        let token = self.advance();
        let text = self.text(&token);
        let span = token.span;

        match token.kind {
            SyntaxKind::IntLit => {
                let val = text.parse::<i64>().unwrap_or(0);
                Spanned::new(Expr::IntLit(val), span)
            }
            SyntaxKind::FloatLit => {
                let val = text.parse::<f64>().unwrap_or(0.0);
                Spanned::new(Expr::FloatLit(val), span)
            }
            SyntaxKind::LengthLit => {
                let (num_str, unit_str) = split_number_unit(text);
                let val = num_str.parse::<f64>().unwrap_or(0.0);
                let unit = match unit_str {
                    "mm" => LengthUnit::Mm,
                    "cm" => LengthUnit::Cm,
                    "m" => LengthUnit::M,
                    "in" => LengthUnit::In,
                    _ => LengthUnit::Mm, // fallback
                };
                Spanned::new(Expr::LengthLit(val, unit), span)
            }
            SyntaxKind::AngleLit => {
                let (num_str, unit_str) = split_number_unit(text);
                let val = num_str.parse::<f64>().unwrap_or(0.0);
                let unit = match unit_str {
                    "deg" => AngleUnit::Deg,
                    "rad" => AngleUnit::Rad,
                    _ => AngleUnit::Deg, // fallback
                };
                Spanned::new(Expr::AngleLit(val, unit), span)
            }
            _ => unreachable!(),
        }
    }

    fn parse_string_lit(&mut self) -> Spanned<Expr> {
        let token = self.advance();
        let text = self.text(&token);
        // Strip quotes
        let inner = &text[1..text.len() - 1];
        // Basic escape handling
        let unescaped = unescape(inner);
        Spanned::new(Expr::StringLit(unescaped), token.span)
    }

    fn parse_grouped_expr(&mut self) -> Spanned<Expr> {
        let lparen = self.advance(); // consume '('
        self.skip_newlines();
        let inner = self.parse_expr();
        self.skip_newlines();
        let rparen = self.expect(SyntaxKind::RParen);
        let end_span = rparen.map_or(inner.span, |t| t.span);
        Spanned::new(Expr::Grouped(Box::new(inner)), lparen.span.merge(end_span))
    }

    fn parse_list_expr(&mut self) -> Spanned<Expr> {
        let lbracket = self.advance(); // consume '['
        self.skip_newlines();

        let mut items = Vec::new();
        while !self.at(SyntaxKind::RBracket) && !self.at_end() {
            items.push(self.parse_expr());
            self.skip_newlines();
            if self.at(SyntaxKind::Comma) {
                self.advance();
            }
            self.skip_newlines();
        }

        let rbracket = self.expect(SyntaxKind::RBracket);
        let end_span = rbracket.map_or(lbracket.span, |t| t.span);
        Spanned::new(Expr::List(items), lbracket.span.merge(end_span))
    }

    fn parse_block_expr(&mut self) -> Spanned<Expr> {
        let lbrace = self.advance(); // consume '{'
        self.skip_newlines();

        let mut stmts = Vec::new();
        let mut tail = None;

        while !self.at(SyntaxKind::RBrace) && !self.at_end() {
            self.skip_newlines();
            if self.at(SyntaxKind::RBrace) || self.at_end() {
                break;
            }

            // Try to parse a statement
            if let Some(stmt) = self.parse_stmt() {
                stmts.push(stmt);
            }

            self.skip_newlines();
            // Skip optional semicolons
            while self.at(SyntaxKind::Semicolon) || self.at(SyntaxKind::Newline) {
                self.advance();
            }
        }

        // The last expression statement becomes the tail if there's no semicolon after it
        if stmts
            .last()
            .is_some_and(|s| matches!(s.node, Stmt::Expr(_)))
        {
            let last = stmts.pop().unwrap();
            if let Stmt::Expr(expr) = last.node {
                tail = Some(Box::new(expr));
            }
        }

        let rbrace = self.expect(SyntaxKind::RBrace);
        let end_span = rbrace.map_or(lbrace.span, |t| t.span);

        Spanned::new(Expr::Block { stmts, tail }, lbrace.span.merge(end_span))
    }

    fn parse_if_expr(&mut self) -> Spanned<Expr> {
        let if_tok = self.advance(); // consume 'if'
        self.skip_newlines();

        let cond = self.parse_expr();
        self.skip_newlines();
        let then_branch = self.parse_block_expr();

        self.skip_newlines();
        let else_branch = if self.at(SyntaxKind::Else) {
            self.advance();
            self.skip_newlines();
            Some(Box::new(if self.at(SyntaxKind::If) {
                self.parse_if_expr()
            } else {
                self.parse_block_expr()
            }))
        } else {
            None
        };

        let end_span = else_branch.as_ref().map_or(then_branch.span, |e| e.span);

        Spanned::new(
            Expr::If {
                cond: Box::new(cond),
                then_branch: Box::new(then_branch),
                else_branch,
            },
            if_tok.span.merge(end_span),
        )
    }

    fn parse_match_expr(&mut self) -> Spanned<Expr> {
        let match_tok = self.advance(); // consume 'match'
        self.skip_newlines();

        let subject = self.parse_expr();
        self.skip_newlines();
        let _ = self.expect(SyntaxKind::LBrace);
        self.skip_newlines();

        let mut arms = Vec::new();
        while !self.at(SyntaxKind::RBrace) && !self.at_end() {
            let pattern = self.parse_pattern();
            self.skip_newlines();
            let _ = self.expect(SyntaxKind::FatArrow);
            self.skip_newlines();
            let body = self.parse_expr();
            let arm_span = pattern.span.merge(body.span);
            arms.push(MatchArm {
                pattern,
                body,
                span: arm_span,
            });
            self.skip_newlines();
            if self.at(SyntaxKind::Comma) {
                self.advance();
            }
            self.skip_newlines();
        }

        let rbrace = self.expect(SyntaxKind::RBrace);
        let end_span = rbrace.map_or(match_tok.span, |t| t.span);

        Spanned::new(
            Expr::Match {
                subject: Box::new(subject),
                arms,
            },
            match_tok.span.merge(end_span),
        )
    }

    fn parse_lambda_expr(&mut self) -> Spanned<Expr> {
        let pipe_tok = self.advance(); // consume '|'
        self.skip_newlines();

        let mut params = Vec::new();
        while !self.at(SyntaxKind::Pipe) && !self.at_end() {
            let name_tok = self.advance();
            let name = Spanned::new(self.text(&name_tok).to_string(), name_tok.span);

            self.skip_newlines();
            let ty = if self.at(SyntaxKind::Colon) {
                self.advance();
                self.skip_newlines();
                Some(self.parse_type())
            } else {
                None
            };

            let param_span = name.span.merge(ty.as_ref().map_or(name.span, |t| t.span));
            params.push(Param {
                name,
                ty,
                default: None,
                span: param_span,
            });

            self.skip_newlines();
            if self.at(SyntaxKind::Comma) {
                self.advance();
                self.skip_newlines();
            }
        }

        let _ = self.expect(SyntaxKind::Pipe); // closing '|'
        self.skip_newlines();
        let body = self.parse_expr();
        let span = pipe_tok.span.merge(body.span);

        Spanned::new(
            Expr::Lambda {
                params,
                body: Box::new(body),
            },
            span,
        )
    }

    fn parse_pattern(&mut self) -> Spanned<Pattern> {
        match self.peek() {
            SyntaxKind::Ident => {
                let tok = self.advance();
                let text = self.text(&tok).to_string();
                let pattern = if text == "_" {
                    Pattern::Wildcard
                } else {
                    Pattern::Ident(text)
                };
                Spanned::new(pattern, tok.span)
            }
            SyntaxKind::IntLit
            | SyntaxKind::FloatLit
            | SyntaxKind::LengthLit
            | SyntaxKind::AngleLit
            | SyntaxKind::StringLit
            | SyntaxKind::True
            | SyntaxKind::False => {
                let expr = self.parse_primary();
                let span = expr.span;
                Spanned::new(Pattern::Literal(Box::new(expr)), span)
            }
            _ => {
                let tok = self.advance();
                self.push_error(
                    format!("expected pattern, found {}", tok.kind.name()),
                    tok.span,
                    ErrorKind::ExpectedExpr,
                );
                Spanned::new(Pattern::Wildcard, tok.span)
            }
        }
    }
}

// ======== Argument and field lists ========

impl Parser<'_> {
    fn parse_arg_list(&mut self) -> Vec<Arg> {
        let mut args = Vec::new();
        self.skip_newlines();

        while !self.at(SyntaxKind::RParen) && !self.at_end() {
            let start_span = self.current_token().span;

            // Check for named argument: `name = value`
            if self.peek() == SyntaxKind::Ident && self.is_named_arg() {
                let name_tok = self.advance();
                let name = Spanned::new(self.text(&name_tok).to_string(), name_tok.span);
                let _ = self.expect(SyntaxKind::Eq);
                self.skip_newlines();
                let value = self.parse_expr();
                let span = start_span.merge(value.span);
                args.push(Arg {
                    name: Some(name),
                    value,
                    span,
                });
            } else {
                let value = self.parse_expr();
                let span = start_span.merge(value.span);
                args.push(Arg {
                    name: None,
                    value,
                    span,
                });
            }

            self.skip_newlines();
            if self.at(SyntaxKind::Comma) {
                self.advance();
            }
            self.skip_newlines();
        }

        args
    }

    /// Look ahead to determine if current Ident is a named argument (ident = expr).
    fn is_named_arg(&self) -> bool {
        // Peek at pos+1 to see if it's '=' (but not '==')
        let next_pos = self.pos + 1;
        if let Some(tok) = self.tokens.get(next_pos) {
            tok.kind == SyntaxKind::Eq
        } else {
            false
        }
    }

    fn parse_field_init_list(&mut self) -> Vec<FieldInit> {
        let mut fields = Vec::new();
        self.skip_newlines();

        while !self.at(SyntaxKind::RBrace) && !self.at_end() {
            let name_tok = self.advance();
            let name = Spanned::new(self.text(&name_tok).to_string(), name_tok.span);
            self.skip_newlines();
            let _ = self.expect(SyntaxKind::Eq);
            self.skip_newlines();
            let value = self.parse_expr();
            let span = name.span.merge(value.span);

            fields.push(FieldInit { name, value, span });

            self.skip_newlines();
            if self.at(SyntaxKind::Comma) {
                self.advance();
            }
            self.skip_newlines();
        }

        fields
    }

    fn parse_param_list(&mut self) -> Vec<Param> {
        let mut params = Vec::new();
        self.skip_newlines();

        while !self.at(SyntaxKind::RParen) && !self.at_end() {
            let name_tok = self.advance();
            let name = Spanned::new(self.text(&name_tok).to_string(), name_tok.span);

            self.skip_newlines();
            let ty = if self.at(SyntaxKind::Colon) {
                self.advance();
                self.skip_newlines();
                Some(self.parse_type())
            } else {
                None
            };

            self.skip_newlines();
            let default = if self.at(SyntaxKind::Eq) {
                self.advance();
                self.skip_newlines();
                Some(self.parse_expr())
            } else {
                None
            };

            let end = default
                .as_ref()
                .map(|d| d.span)
                .or(ty.as_ref().map(|t| t.span))
                .unwrap_or(name.span);
            let span = name.span.merge(end);

            params.push(Param {
                name,
                ty,
                default,
                span,
            });

            self.skip_newlines();
            if self.at(SyntaxKind::Comma) {
                self.advance();
            }
            self.skip_newlines();
        }

        params
    }

    fn parse_field_list(&mut self) -> Vec<Field> {
        let mut fields = Vec::new();
        self.skip_newlines();

        while !self.at(SyntaxKind::RBrace) && !self.at_end() {
            let name_tok = self.advance();
            let name = Spanned::new(self.text(&name_tok).to_string(), name_tok.span);

            self.skip_newlines();
            let _ = self.expect(SyntaxKind::Colon);
            self.skip_newlines();
            let ty = self.parse_type();

            self.skip_newlines();
            let default = if self.at(SyntaxKind::Eq) {
                self.advance();
                self.skip_newlines();
                Some(self.parse_expr())
            } else {
                None
            };

            let end = default.as_ref().map(|d| d.span).unwrap_or(ty.span);
            let span = name.span.merge(end);

            fields.push(Field {
                name,
                ty,
                default,
                span,
            });

            self.skip_newlines();
            if self.at(SyntaxKind::Comma) {
                self.advance();
            }
            self.skip_newlines();
        }

        fields
    }
}

// ======== Types ========

impl Parser<'_> {
    fn parse_type(&mut self) -> Spanned<Type> {
        let tok = self.advance();
        let name = self.text(&tok).to_string();
        let span = tok.span;

        // Check for List[T]
        if name == "List" && self.at(SyntaxKind::LBracket) {
            self.advance();
            self.skip_newlines();
            let inner = self.parse_type();
            self.skip_newlines();
            let rbracket = self.expect(SyntaxKind::RBracket);
            let end_span = rbracket.map_or(inner.span, |t| t.span);
            return Spanned::new(Type::List(Box::new(inner)), span.merge(end_span));
        }

        // Check for Fn(A, B) -> C
        if name == "Fn" && self.at(SyntaxKind::LParen) {
            self.advance();
            self.skip_newlines();
            let mut params = Vec::new();
            while !self.at(SyntaxKind::RParen) && !self.at_end() {
                params.push(self.parse_type());
                self.skip_newlines();
                if self.at(SyntaxKind::Comma) {
                    self.advance();
                    self.skip_newlines();
                }
            }
            let _ = self.expect(SyntaxKind::RParen);
            self.skip_newlines();
            let _ = self.expect(SyntaxKind::Arrow);
            self.skip_newlines();
            let ret = self.parse_type();
            let end_span = ret.span;
            return Spanned::new(
                Type::Fn {
                    params,
                    ret: Box::new(ret),
                },
                span.merge(end_span),
            );
        }

        Spanned::new(Type::Named(name), span)
    }
}

// ======== Helpers ========

fn prefix_bp(kind: SyntaxKind) -> Option<u8> {
    match kind {
        SyntaxKind::Minus | SyntaxKind::Bang => Some(15),
        _ => None,
    }
}

fn infix_bp(kind: SyntaxKind) -> Option<(u8, u8)> {
    // (left_bp, right_bp): left < right for left-associative
    match kind {
        SyntaxKind::PipeGt => Some((1, 2)),
        SyntaxKind::PipePipe => Some((3, 4)),
        SyntaxKind::AmpAmp => Some((5, 6)),
        SyntaxKind::EqEq | SyntaxKind::BangEq => Some((7, 8)),
        SyntaxKind::Lt | SyntaxKind::LtEq | SyntaxKind::Gt | SyntaxKind::GtEq => Some((9, 10)),
        SyntaxKind::Plus | SyntaxKind::Minus => Some((11, 12)),
        SyntaxKind::Star | SyntaxKind::Slash => Some((13, 14)),
        _ => None,
    }
}

fn split_number_unit(text: &str) -> (&str, &str) {
    let split_pos = text
        .find(|c: char| c.is_ascii_alphabetic())
        .unwrap_or(text.len());
    (&text[..split_pos], &text[split_pos..])
}

fn unescape(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('t') => result.push('\t'),
                Some('\\') => result.push('\\'),
                Some('"') => result.push('"'),
                Some(other) => {
                    result.push('\\');
                    result.push(other);
                }
                None => result.push('\\'),
            }
        } else {
            result.push(c);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer;

    fn parse_expr_str(source: &str) -> Spanned<Expr> {
        let (tokens, lex_errors) = lexer::lex(source);
        assert!(lex_errors.is_empty(), "lex errors: {lex_errors:?}");
        let mut parser = Parser::new(source, tokens);
        let expr = parser.parse_expr();
        assert!(
            parser.errors.is_empty(),
            "parse errors: {:?}",
            parser.errors
        );
        expr
    }

    fn parse_file_str(source: &str) -> SourceFile {
        let (tokens, lex_errors) = lexer::lex(source);
        assert!(lex_errors.is_empty(), "lex errors: {lex_errors:?}");
        let (file, parse_errors) = parse(source, tokens);
        assert!(parse_errors.is_empty(), "parse errors: {parse_errors:?}");
        file
    }

    #[test]
    fn integer_literal() {
        let expr = parse_expr_str("42");
        assert!(matches!(expr.node, Expr::IntLit(42)));
    }

    #[test]
    fn float_literal() {
        let expr = parse_expr_str("1.23");
        assert!(matches!(expr.node, Expr::FloatLit(v) if (v - 1.23).abs() < f64::EPSILON));
    }

    #[test]
    fn length_literal() {
        let expr = parse_expr_str("10mm");
        assert!(
            matches!(expr.node, Expr::LengthLit(v, LengthUnit::Mm) if (v - 10.0).abs() < f64::EPSILON)
        );
    }

    #[test]
    fn angle_literal() {
        let expr = parse_expr_str("45deg");
        assert!(
            matches!(expr.node, Expr::AngleLit(v, AngleUnit::Deg) if (v - 45.0).abs() < f64::EPSILON)
        );
    }

    #[test]
    fn bool_literal() {
        assert!(matches!(parse_expr_str("true").node, Expr::BoolLit(true)));
        assert!(matches!(parse_expr_str("false").node, Expr::BoolLit(false)));
    }

    #[test]
    fn string_literal() {
        let expr = parse_expr_str("\"hello\"");
        assert!(matches!(expr.node, Expr::StringLit(ref s) if s == "hello"));
    }

    #[test]
    fn binary_add() {
        let expr = parse_expr_str("1 + 2");
        match &expr.node {
            Expr::BinOp { op, .. } => assert_eq!(op.node, BinOpKind::Add),
            other => panic!("expected BinOp, got {other:?}"),
        }
    }

    #[test]
    fn precedence_mul_over_add() {
        // 1 + 2 * 3 should parse as 1 + (2 * 3)
        let expr = parse_expr_str("1 + 2 * 3");
        match &expr.node {
            Expr::BinOp { op, rhs, .. } => {
                assert_eq!(op.node, BinOpKind::Add);
                assert!(matches!(rhs.node, Expr::BinOp { .. }));
            }
            other => panic!("expected BinOp(Add), got {other:?}"),
        }
    }

    #[test]
    fn precedence_paren() {
        // (1 + 2) * 3
        let expr = parse_expr_str("(1 + 2) * 3");
        match &expr.node {
            Expr::BinOp { op, lhs, .. } => {
                assert_eq!(op.node, BinOpKind::Mul);
                assert!(matches!(lhs.node, Expr::Grouped(_)));
            }
            other => panic!("expected BinOp(Mul), got {other:?}"),
        }
    }

    #[test]
    fn unary_neg() {
        let expr = parse_expr_str("-x");
        match &expr.node {
            Expr::UnaryOp { op, operand } => {
                assert_eq!(op.node, UnaryOpKind::Neg);
                assert!(matches!(operand.node, Expr::Ident(_)));
            }
            other => panic!("expected UnaryOp, got {other:?}"),
        }
    }

    #[test]
    fn unary_not() {
        let expr = parse_expr_str("!flag");
        match &expr.node {
            Expr::UnaryOp { op, .. } => assert_eq!(op.node, UnaryOpKind::Not),
            other => panic!("expected UnaryOp, got {other:?}"),
        }
    }

    #[test]
    fn function_call() {
        let expr = parse_expr_str("foo(1, 2)");
        match &expr.node {
            Expr::FnCall { func, args } => {
                assert!(matches!(func.node, Expr::Ident(ref n) if n == "foo"));
                assert_eq!(args.len(), 2);
            }
            other => panic!("expected FnCall, got {other:?}"),
        }
    }

    #[test]
    fn named_args() {
        let expr = parse_expr_str("foo(depth = 10mm, chamfer = 0.5mm)");
        match &expr.node {
            Expr::FnCall { args, .. } => {
                assert!(args[0].name.is_some());
                assert_eq!(args[0].name.as_ref().unwrap().node, "depth");
                assert_eq!(args[1].name.as_ref().unwrap().node, "chamfer");
            }
            other => panic!("expected FnCall, got {other:?}"),
        }
    }

    #[test]
    fn mixed_args() {
        let expr = parse_expr_str("f(a, b = 1)");
        match &expr.node {
            Expr::FnCall { args, .. } => {
                assert!(args[0].name.is_none());
                assert!(args[1].name.is_some());
            }
            other => panic!("expected FnCall, got {other:?}"),
        }
    }

    #[test]
    fn field_access() {
        let expr = parse_expr_str("plate.width");
        match &expr.node {
            Expr::FieldAccess { field, .. } => assert_eq!(field.node, "width"),
            other => panic!("expected FieldAccess, got {other:?}"),
        }
    }

    #[test]
    fn chained_field_and_call() {
        let expr = parse_expr_str("a.b(c)");
        assert!(matches!(expr.node, Expr::FnCall { .. }));
    }

    #[test]
    fn pipe_operator() {
        let expr = parse_expr_str("x |> f |> g");
        // |> is left-assoc: ((x |> f) |> g)
        match &expr.node {
            Expr::BinOp { op, lhs, .. } => {
                assert_eq!(op.node, BinOpKind::Pipe);
                assert!(matches!(lhs.node, Expr::BinOp { .. }));
            }
            other => panic!("expected nested pipe, got {other:?}"),
        }
    }

    #[test]
    fn let_statement() {
        let file = parse_file_str("let x = 42");
        assert_eq!(file.stmts.len(), 1);
        match &file.stmts[0].node {
            Stmt::Let(ls) => assert_eq!(ls.name.node, "x"),
            other => panic!("expected Let, got {other:?}"),
        }
    }

    #[test]
    fn let_with_type() {
        let file = parse_file_str("let x: Int = 42");
        match &file.stmts[0].node {
            Stmt::Let(ls) => {
                assert!(ls.ty.is_some());
                assert!(matches!(ls.ty.as_ref().unwrap().node, Type::Named(ref n) if n == "Int"));
            }
            other => panic!("expected Let, got {other:?}"),
        }
    }

    #[test]
    fn fn_def() {
        let file = parse_file_str("fn add(a: Int, b: Int) -> Int { a + b }");
        assert_eq!(file.stmts.len(), 1);
        match &file.stmts[0].node {
            Stmt::FnDef(fndef) => {
                assert_eq!(fndef.name.node, "add");
                assert_eq!(fndef.params.len(), 2);
                assert!(fndef.return_ty.is_some());
            }
            other => panic!("expected FnDef, got {other:?}"),
        }
    }

    #[test]
    fn data_def() {
        let file = parse_file_str("data Rectangle { width: Length, height: Length }");
        match &file.stmts[0].node {
            Stmt::DataDef(dd) => {
                assert_eq!(dd.name.node, "Rectangle");
                assert_eq!(dd.fields.len(), 2);
            }
            other => panic!("expected DataDef, got {other:?}"),
        }
    }

    #[test]
    fn data_def_with_defaults() {
        let file = parse_file_str("data Rect { w: Length = 10mm, h: Length = 20mm }");
        match &file.stmts[0].node {
            Stmt::DataDef(dd) => {
                assert!(dd.fields[0].default.is_some());
                assert!(dd.fields[1].default.is_some());
            }
            other => panic!("expected DataDef, got {other:?}"),
        }
    }

    #[test]
    fn enum_def() {
        let file = parse_file_str("enum Color { Red, Green, Blue }");
        match &file.stmts[0].node {
            Stmt::EnumDef(ed) => {
                assert_eq!(ed.name.node, "Color");
                assert_eq!(ed.variants.len(), 3);
            }
            other => panic!("expected EnumDef, got {other:?}"),
        }
    }

    #[test]
    fn if_expression() {
        let expr = parse_expr_str("if true { 1 } else { 2 }");
        assert!(matches!(expr.node, Expr::If { .. }));
    }

    #[test]
    fn if_without_else() {
        let expr = parse_expr_str("if flag { 1 }");
        match &expr.node {
            Expr::If { else_branch, .. } => assert!(else_branch.is_none()),
            other => panic!("expected If, got {other:?}"),
        }
    }

    #[test]
    fn match_expression() {
        let expr = parse_expr_str("match x { 1 => \"one\", _ => \"other\" }");
        match &expr.node {
            Expr::Match { arms, .. } => assert_eq!(arms.len(), 2),
            other => panic!("expected Match, got {other:?}"),
        }
    }

    #[test]
    fn lambda() {
        let expr = parse_expr_str("|x| x + 1");
        match &expr.node {
            Expr::Lambda { params, .. } => assert_eq!(params.len(), 1),
            other => panic!("expected Lambda, got {other:?}"),
        }
    }

    #[test]
    fn multi_param_lambda() {
        let expr = parse_expr_str("|x, y| x + y");
        match &expr.node {
            Expr::Lambda { params, .. } => assert_eq!(params.len(), 2),
            other => panic!("expected Lambda, got {other:?}"),
        }
    }

    #[test]
    fn list_literal() {
        let expr = parse_expr_str("[1, 2, 3]");
        match &expr.node {
            Expr::List(items) => assert_eq!(items.len(), 3),
            other => panic!("expected List, got {other:?}"),
        }
    }

    #[test]
    fn empty_list() {
        let expr = parse_expr_str("[]");
        match &expr.node {
            Expr::List(items) => assert_eq!(items.len(), 0),
            other => panic!("expected List, got {other:?}"),
        }
    }

    #[test]
    fn data_constructor() {
        let expr = parse_expr_str("Rectangle { width = 50mm, height = 100mm }");
        match &expr.node {
            Expr::DataConstructor { name, fields } => {
                assert_eq!(name.node, "Rectangle");
                assert_eq!(fields.len(), 2);
            }
            other => panic!("expected DataConstructor, got {other:?}"),
        }
    }

    #[test]
    fn with_update() {
        let expr = parse_expr_str("r with { height = 200mm }");
        assert!(matches!(expr.node, Expr::WithUpdate { .. }));
    }

    #[test]
    fn block_expression() {
        let file = parse_file_str("{ let x = 1; x + 2 }");
        assert_eq!(file.stmts.len(), 1);
    }

    #[test]
    fn nested_calls() {
        let expr = parse_expr_str("f(g(x), h(y))");
        match &expr.node {
            Expr::FnCall { args, .. } => {
                assert_eq!(args.len(), 2);
                assert!(matches!(args[0].value.node, Expr::FnCall { .. }));
                assert!(matches!(args[1].value.node, Expr::FnCall { .. }));
            }
            other => panic!("expected FnCall, got {other:?}"),
        }
    }

    #[test]
    fn mounting_plate_example() {
        let source = r#"
let plate = box(vec3(80mm, 50mm, 5mm))

let hole = threaded_hole(
  ISO_METRIC, M3, TAP,
  depth = 8mm,
  chamfer = 0.5mm
)

let holes = union_many([
  move(hole, vec3(10mm, 10mm, 0)),
  move(hole, vec3(70mm, 10mm, 0)),
  move(hole, vec3(70mm, 40mm, 0)),
  move(hole, vec3(10mm, 40mm, 0))
])

let model = difference(plate, holes)

export_stl("mounting_plate.stl", model)
"#;
        let file = parse_file_str(source);
        assert_eq!(file.stmts.len(), 5);
    }
}
