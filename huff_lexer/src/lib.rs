//! ## Huff Lexer
//!
//! Lexical analyzer for the huff language.
//!
//! The Huff Lexer is instantiable with a string representing the source code.
//!
//! Once instantiated, the lexer can be used to iterate over the tokens in the source code.
//! It also exposes a number of practical methods for accessing information about the source code
//! throughout lexing.
//!
//! #### Usage
//!
//! The following example steps through the lexing of a simple, single-line source code macro
//! definition.
//!
//! ```rust
//! use huff_utils::{token::*, span::*};
//! use huff_lexer::{Lexer};
//!
//! // Instantiate a new lexer
//! let source = "#define macro HELLO_WORLD()";
//! let mut lexer = Lexer::new(source);
//! assert_eq!(lexer.source, source);
//!
//! // This token should be a Define identifier
//! let tok = lexer.next().unwrap().unwrap();
//! assert_eq!(tok, Token::new(TokenKind::Define, Span::new(0..7)));
//! assert_eq!(lexer.span, Span::new(0..7));
//!
//! // The next token should be the whitespace
//! let tok = lexer.next().unwrap().unwrap();
//! assert_eq!(tok, Token::new(TokenKind::Whitespace, Span::new(7..8)));
//! assert_eq!(lexer.span, Span::new(7..8));
//!
//! // Then we should parse the macro keyword
//! let tok = lexer.next().unwrap().unwrap();
//! assert_eq!(tok, Token::new(TokenKind::Macro, Span::new(8..13)));
//! assert_eq!(lexer.span, Span::new(8..13));
//!
//! // The next token should be another whitespace
//! let tok = lexer.next().unwrap().unwrap();
//! assert_eq!(tok, Token::new(TokenKind::Whitespace, Span::new(13..14)));
//! assert_eq!(lexer.span, Span::new(13..14));
//!
//! // Then we should get the function name
//! let tok = lexer.next().unwrap().unwrap();
//! assert_eq!(tok, Token::new(TokenKind::Ident("HELLO_WORLD"), Span::new(14..25)));
//! assert_eq!(lexer.span, Span::new(14..25));
//!
//! // Then we should have an open paren
//! let tok = lexer.next().unwrap().unwrap();
//! assert_eq!(tok, Token::new(TokenKind::OpenParen, Span::new(25..26)));
//! assert_eq!(lexer.span, Span::new(25..26));
//!
//! // Lastly, we should have a closing parenthesis
//! let tok = lexer.next().unwrap().unwrap();
//! assert_eq!(tok, Token::new(TokenKind::CloseParen, Span::new(26..27)));
//! assert_eq!(lexer.span, Span::new(26..27));
//!
//! // We covered the whole source
//! assert_eq!(lexer.span.end, source.len());
//! assert!(lexer.eof);
//! ```

#![deny(missing_docs)]
#![allow(dead_code)]

use huff_utils::{error::*, span::*, token::*};
use std::{iter::Peekable, str::Chars};

/// ## Lexer
///
/// The lexer encapsulated in a struct.
pub struct Lexer<'a> {
    /// The source code as peekable chars.
    pub chars: Peekable<Chars<'a>>,
    /// The raw source code.
    pub source: &'a str,
    /// The current lexing span.
    pub span: Span,
    /// If the lexer has reached the end of file.
    pub eof: bool,
    /// EOF Token has been returned.
    pub eof_returned: bool,
}

impl<'a> Lexer<'a> {
    /// Public associated function that instantiates a new lexer.
    pub fn new(source: &'a str) -> Self {
        Self {
            chars: source.chars().peekable(),
            source,
            span: Span::default(),
            eof: false,
            eof_returned: false,
        }
    }

    /// Public associated function that returns the current lexing span.
    pub fn current_span(&self) -> Span {
        if self.eof {
            Span::EOF
        } else {
            self.span
        }
    }

    /// Try to peek at the next character from the source
    pub fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    /// Try to peek at the nth character from the source
    pub fn nthpeek(&mut self, n: usize) -> Option<char> {
        self.chars.clone().nth(n)
    }

    /// Try to peek at next n characters from the source
    pub fn peeknchars(&mut self, n: usize) -> String {
        let mut newspan: Span = self.span;
        newspan.end += n;
        // Break with an empty string if the bounds are exceeded
        if newspan.end > self.source.len() {
            return String::default()
        }
        self.source[newspan.range().unwrap()].to_string()
    }

    /// Peek n chars from a given start point in the source
    pub fn peekncharsfrom(&mut self, n: usize, from: usize) -> String {
        self.source[Span::new(from..(from + n)).range().unwrap()].to_string()
    }

    /// Try to look back `dist` chars from `span.start`, but return an empty string if
    /// `self.span.start - dist` will underflow.
    pub fn try_look_back(&mut self, dist: usize) -> String {
        match self.span.start.checked_sub(dist) {
            Some(n) => self.peekncharsfrom(dist - 1, n),
            None => String::default(),
        }
    }

    // pub fn check_keyword_rule(&mut self, )

    /// Gets the current slice of the source code covered by span
    pub fn slice(&self) -> &'a str {
        &self.source[self.span.range().unwrap()]
    }

    /// Consumes the characters
    pub fn consume(&mut self) -> Option<char> {
        self.chars.next().map(|x| {
            self.span.end += 1;
            x
        })
    }

    /// Consumes n characters
    pub fn nconsume(&mut self, count: usize) {
        for _ in 0..count {
            let _ = self.consume();
        }
    }

    /// Consume characters until a sequence matches
    pub fn seq_consume(&mut self, word: &str) {
        let mut current_pos = self.span.start;
        while self.peek() != None {
            let peeked = self.peekncharsfrom(word.len(), current_pos);
            if word == peeked {
                break
            }
            self.consume();
            current_pos += 1;
        }
    }

    /// Dynamically consumes characters based on filters
    pub fn dyn_consume(&mut self, f: impl Fn(&char) -> bool + Copy) {
        while self.peek().map(|x| f(&x)).unwrap_or(false) {
            self.consume();
        }
    }

    /// Resets the Lexer's span
    pub fn reset(&mut self) {
        self.span.start = self.span.end;
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<Token<'a>, LexicalError>;

    /// Iterates over the source code
    fn next(&mut self) -> Option<Self::Item> {
        self.reset();
        if let Some(ch) = self.consume() {
            let kind = match ch {
                // Comments
                '/' => {
                    if let Some(ch2) = self.peek() {
                        match ch2 {
                            '/' => {
                                self.consume();
                                // Consume until newline
                                self.dyn_consume(|c| *c != '\n');
                                TokenKind::Comment(self.slice())
                            }
                            '*' => {
                                self.consume();
                                // Consume until next '*/' occurance
                                self.seq_consume("*/");
                                TokenKind::Comment(self.slice())
                            }
                            _ => TokenKind::Div,
                        }
                    } else {
                        TokenKind::Div
                    }
                }
                // # keywords
                '#' => {
                    let mut found_kind: Option<TokenKind> = None;

                    // Match exactly on define keyword
                    let define_keyword = "#define";
                    let peeked = self.peeknchars(define_keyword.len() - 1);
                    if define_keyword == peeked {
                        self.dyn_consume(|c| c.is_alphabetic());
                        found_kind = Some(TokenKind::Define);
                    }

                    if found_kind == None {
                        // Match on the include keyword
                        let include_keyword = "#include";
                        let peeked = self.peeknchars(include_keyword.len() - 1);
                        if include_keyword == peeked {
                            self.dyn_consume(|c| c.is_alphabetic());
                            found_kind = Some(TokenKind::Include);
                        }
                    }

                    if let Some(kind) = found_kind {
                        kind
                    } else {
                        // Otherwise we don't support # prefixed indentifiers
                        return Some(Err(LexicalError::new(
                            LexicalErrorKind::InvalidCharacter('#'),
                            self.current_span(),
                        )))
                    }
                }
                // Alphabetical characters
                ch if ch.is_alphabetic() => {
                    let mut found_kind: Option<TokenKind> = None;

                    let keys = [
                        ("macro", TokenKind::Macro),
                        ("function", TokenKind::Function),
                        ("constant", TokenKind::Constant),
                        ("takes", TokenKind::Takes),
                        ("returns", TokenKind::Returns),
                    ];
                    let mut active_key = ""; // Initialize blank string to prevent "possibly-uninitialized" compiler error.
                    for (key, kind) in &keys {
                        active_key = *key;
                        let peeked = self.peeknchars(active_key.len() - 1);

                        if active_key == peeked {
                            self.dyn_consume(|c| c.is_alphabetic());
                            found_kind = Some(*kind);
                            break
                        }
                    }

                    // Check to see if the found kind is, in fact, a keyword and not the name of
                    // a function. If it is, set `found_kind` to `None` so that it is set to a
                    // `TokenKind::Ident` in the following control flow.
                    //
                    // TODO: Add some extra checks for other cases.
                    // e.g. "dup1 0x7c09063f eq takes jumpi" still registers "takes" as a
                    // `TokenKind::Takes`
                    let function_keyword = keys[1].0;
                    if self.try_look_back(function_keyword.len() + 1) == function_keyword ||
                        self.peekncharsfrom(1, active_key.len()) == ":"
                    {
                        found_kind = None;
                    }

                    // Check for macro keyword
                    let fsp = "FREE_STORAGE_POINTER";
                    let peeked = self.peeknchars(fsp.len() - 1);
                    if fsp == peeked {
                        self.dyn_consume(|c| c.is_alphabetic() || c.eq(&'_'));
                        // Consume the parenthesis following the FREE_STORAGE_POINTER
                        if let Some('(') = self.peek() {
                            self.consume();
                        }
                        if let Some(')') = self.peek() {
                            self.consume();
                        }
                        found_kind = Some(TokenKind::FreeStoragePointer);
                    }

                    if let Some(kind) = found_kind {
                        kind
                    } else {
                        self.dyn_consume(|c| c.is_alphanumeric() || c.eq(&'_'));
                        TokenKind::Ident(self.slice())
                    }
                }
                '=' => TokenKind::Assign,
                '(' => TokenKind::OpenParen,
                ')' => TokenKind::CloseParen,
                '[' => TokenKind::OpenBracket,
                ']' => TokenKind::CloseBracket,
                '{' => TokenKind::OpenBrace,
                '}' => TokenKind::CloseBrace,
                '+' => TokenKind::Add,
                '-' => TokenKind::Sub,
                '*' => TokenKind::Mul,
                // NOTE: TokenKind::Div is lexed further up since it overlaps with comment
                // identifiers
                ',' => TokenKind::Comma,
                '0'..='9' => {
                    self.dyn_consume(char::is_ascii_digit);
                    TokenKind::Num(self.slice().parse().unwrap())
                }
                // Lexes Spaces and Newlines as Whitespace
                ch if ch.is_ascii_whitespace() => {
                    self.dyn_consume(char::is_ascii_whitespace);
                    TokenKind::Whitespace
                }
                // String literals
                '"' => loop {
                    match self.peek() {
                        Some('"') => {
                            self.consume();
                            let str = self.slice();
                            break TokenKind::Str(&str[1..str.len() - 1])
                        }
                        Some('\\') if matches!(self.nthpeek(1), Some('\\') | Some('"')) => {
                            self.consume();
                        }
                        Some(_) => {}
                        None => {
                            self.eof = true;
                            return Some(Err(LexicalError::new(
                                LexicalErrorKind::UnexpectedEof,
                                self.span,
                            )))
                        }
                    }
                    self.consume();
                },
                // Allow string literals to be wrapped by single quotes
                '\'' => loop {
                    match self.peek() {
                        Some('\'') => {
                            self.consume();
                            let str = self.slice();
                            break TokenKind::Str(&str[1..str.len() - 1])
                        }
                        Some('\\') if matches!(self.nthpeek(1), Some('\\') | Some('\'')) => {
                            self.consume();
                        }
                        Some(_) => {}
                        None => {
                            self.eof = true;
                            return Some(Err(LexicalError::new(
                                LexicalErrorKind::UnexpectedEof,
                                self.span,
                            )))
                        }
                    }
                    self.consume();
                },
                // At this point, the source code has an invalid or unsupported token
                ch => {
                    return Some(Err(LexicalError::new(
                        LexicalErrorKind::InvalidCharacter(ch),
                        self.span,
                    )))
                }
            };

            if self.peek().is_none() {
                self.eof = true;
            }

            let token = Token { kind, span: self.span };

            return Some(Ok(token))
        }

        // Mark EOF
        self.eof = true;

        // If we haven't returned an eof token, return one
        if !self.eof_returned {
            self.eof_returned = true;
            return Some(Ok(Token { kind: TokenKind::Eof, span: self.span }))
        }

        None
    }
}
