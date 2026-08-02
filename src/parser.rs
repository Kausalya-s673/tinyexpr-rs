//! Recursive-descent parser for tinyexpr-rs.
//!
//! Grammar (unchanged from upstream TinyExpr):
//!
//! ```text
//! list   = expr {"," expr}
//! expr   = term {("+" | "-") term}
//! term   = factor {("*" | "/" | "%") factor}
//! factor = power {"^" power}
//! power  = {("-" | "+")} base
//! base   = number
//!        | variable
//!        | fn0
//!        | fn1 power
//!        | fnX "(" list ")"
//!        | "(" list ")"
//! ```

use crate::ast::{BinaryOp, Builtin, Expr, UnaryOp};
use crate::builtins;
use crate::errors::ParseError;
use crate::lexer::Lexer;
use crate::token::Token;

/// Parses a complete expression from source text. Fails if trailing tokens
/// remain after a full `list` production is consumed.
pub fn parse(input: &str) -> Result<Expr, ParseError> {
    Parser::new(input)?.parse()
}

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current: Token,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Result<Self, ParseError> {
        let mut lexer = Lexer::new(input);
        let current = lexer.next_token()?;
        Ok(Parser { lexer, current })
    }

    pub fn parse(mut self) -> Result<Expr, ParseError> {
        let expr = self.parse_list()?;
        if self.current != Token::End {
            return Err(ParseError::UnexpectedToken);
        }
        Ok(expr)
    }

    // ---- token stream helpers --------------------------------------------

    /// Consumes the current token, pulling the next one from the lexer.
    fn advance(&mut self) -> Result<Token, ParseError> {
        let next = self.lexer.next_token()?;
        Ok(std::mem::replace(&mut self.current, next))
    }

    /// Consumes `tok` if it's current, returning whether it matched.
    fn eat(&mut self, tok: &Token) -> Result<bool, ParseError> {
        if &self.current == tok {
            self.advance()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Consumes `tok` or fails; use for tokens that are grammatically
    /// mandatory at this point (e.g. a closing paren).
    fn expect(&mut self, tok: &Token) -> Result<(), ParseError> {
        if self.eat(tok)? {
            Ok(())
        } else {
            Err(ParseError::UnexpectedToken)
        }
    }

    // ---- grammar productions -----------------------------------------------

    /// `list = expr {"," expr}`
    ///
    /// TinyExpr's comma operator evaluates every operand in order — so
    /// `foo(), bar()` must run both calls — but the value of the whole
    /// list is only its last element. A single `expr` (the common case,
    /// e.g. a lone function argument) is returned as-is; two or more are
    /// collected into `Expr::Sequence` so `eval.rs` can walk and evaluate
    /// every operand instead of just the final one.
    pub fn parse_list(&mut self) -> Result<Expr, ParseError> {
        let mut exprs = vec![self.parse_expr()?];

        while self.eat(&Token::Comma)? {
            exprs.push(self.parse_expr()?);
        }

        if exprs.len() == 1 {
            Ok(exprs.pop().unwrap())
        } else {
            Ok(Expr::Sequence(exprs))
        }
    }

    /// `expr = term {("+" | "-") term}`
    pub fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_term()?;

        loop {
            let op = if self.eat(&Token::Plus)? {
                BinaryOp::Add
            } else if self.eat(&Token::Minus)? {
                BinaryOp::Sub
            } else {
                break;
            };

            let right = self.parse_term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    /// `term = factor {("*" | "/" | "%") factor}`
    pub fn parse_term(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_factor()?;

        loop {
            let op = if self.eat(&Token::Star)? {
                BinaryOp::Mul
            } else if self.eat(&Token::Slash)? {
                BinaryOp::Div
            } else if self.eat(&Token::Percent)? {
                BinaryOp::Mod
            } else {
                break;
            };

            let right = self.parse_factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    /// `factor = power {"^" power}`
    ///
    /// TinyExpr's default build (no `TE_POW_FROM_RIGHT`) makes `^`
    /// **left-associative** — `2^3^2 == (2^3)^2 == 64` — matching
    /// spreadsheet behavior (Excel, Google Sheets), not the
    /// right-associative convention math notation usually uses. This is a
    /// plain `while` fold, same shape as `parse_term`/`parse_expr`, not a
    /// recursive call.
    pub fn parse_factor(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_power()?;

        while self.eat(&Token::Caret)? {
            let right = self.parse_power()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op: BinaryOp::Pow,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    /// `power = {("-" | "+")} base`
    ///
    /// Recurses on itself (not `parse_base`) so chained signs like `--x` or
    /// `+-x` compose correctly.
    pub fn parse_power(&mut self) -> Result<Expr, ParseError> {
        if self.eat(&Token::Minus)? {
            let expr = self.parse_power()?;
            Ok(Expr::Unary {
                op: UnaryOp::Minus,
                expr: Box::new(expr),
            })
        } else if self.eat(&Token::Plus)? {
            let expr = self.parse_power()?;
            Ok(Expr::Unary {
                op: UnaryOp::Plus,
                expr: Box::new(expr),
            })
        } else {
            self.parse_base()
        }
    }

    /// `base = number | variable | fn0 | fn1 power | fnX "(" list ")" | "(" list ")"`
    ///
    /// Peeks at `self.current` (cloned) rather than consuming it up front
    /// via `advance()`. This means the two error arms (`End` / wildcard)
    /// never consume the offending token — it's left sitting in
    /// `self.current` for whatever diagnostics `ParseError` grows later
    /// (position info, "found token X" messages, error recovery, etc.)
    /// instead of already being past it by the time the error is raised.
    pub fn parse_base(&mut self) -> Result<Expr, ParseError> {
        match self.current.clone() {
            Token::Number(n) => {
                self.advance()?;
                Ok(Expr::Number(n))
            }

            Token::OpenParen => {
                self.advance()?;
                let inner = self.parse_list()?;
                self.expect(&Token::CloseParen)?;
                Ok(inner)
            }

            Token::Identifier(name) => {
                self.advance()?;
                match builtins::lookup(&name) {
                    Some(function) => self.parse_call(function),
                    // Not a known builtin -> treat as a variable reference.
                    // Whether it's actually bound is an eval-time concern,
                    // not a parse-time one, so this never fails here.
                    None => Ok(Expr::Variable(name)),
                }
            }

            Token::End => Err(ParseError::UnexpectedEof),

            // Plus/Minus/Star/Slash/Percent/Caret/CloseParen/Comma can't
            // start a `base`. Left unconsumed in `self.current`.
            _ => Err(ParseError::UnexpectedToken),
        }
    }

    /// Parses the argument list for `function`, dispatched on its declared
    /// arity:
    ///   - 0-ary (`pi`, `e`): no parens required, `pi()` also legal.
    ///   - 1-ary (`sin`, `sqrt`, ...): either `fn(x)` or the bare-`power`
    ///     form `fn x` (so `sin -x`, `sqrt 4` both parse).
    ///   - N-ary (`atan2`, `pow`, `ncr`, `npr`): parens and a comma list of
    ///     exactly `n` arguments are mandatory.
    fn parse_call(&mut self, function: Builtin) -> Result<Expr, ParseError> {
        match builtins::arity(function.clone()) {
            0 => {
                if self.eat(&Token::OpenParen)? {
                    self.expect(&Token::CloseParen)?;
                }
                Ok(Expr::Function {
                    function,
                    args: Vec::new(),
                })
            }

            1 => {
                let arg = if self.eat(&Token::OpenParen)? {
                    let arg = self.parse_expr()?;
                    self.expect(&Token::CloseParen)?;
                    arg
                } else {
                    self.parse_power()?
                };
                Ok(Expr::Function {
                    function,
                    args: vec![arg],
                })
            }

            n => {
                self.expect(&Token::OpenParen)?;
                let mut args = vec![self.parse_expr()?];
                while self.eat(&Token::Comma)? {
                    args.push(self.parse_expr()?);
                }
                self.expect(&Token::CloseParen)?;

                if args.len() != n {
                    return Err(ParseError::ArityMismatch {
                        expected: n,
                        found: args.len(),
                    });
                }

                Ok(Expr::Function { function, args })
            }
        }
    }
}
