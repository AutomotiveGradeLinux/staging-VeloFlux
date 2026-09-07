// Copyright 2026 EMQ Technologies Co., Ltd.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Section-based LIN Description File parser.
//!
//! Unknown top-level sections are skipped so OEM files and LIN 2.x extras
//! (`Event_triggered_frames`, `Sporadic_frames`, diagnostics, schedule tables)
//! still load. Only `Signals`, `Frames`, encodings, and representations are
//! interpreted for decode.

use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub(super) struct Position {
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone)]
pub(super) struct SignalDef {
    pub pos: Position,
    pub name: String,
    pub length: u16,
}

#[derive(Debug, Clone)]
pub(super) struct FrameDef {
    pub pos: Position,
    pub name: String,
    pub id: u8,
    pub length: u8,
    pub length_explicit: bool,
    pub placements: Vec<SignalPlacement>,
}

#[derive(Debug, Clone)]
pub(super) struct SignalPlacement {
    pub pos: Position,
    pub name: String,
    pub start: u16,
}

#[derive(Debug, Clone)]
pub(super) struct EncodingDef {
    pub pos: Position,
    pub physical: Vec<PhysicalEncoding>,
    pub has_ascii: bool,
    pub has_bcd: bool,
}

#[derive(Debug, Clone)]
pub(super) struct PhysicalEncoding {
    pub pos: Position,
    pub scale: f64,
    pub offset: f64,
}

#[derive(Debug)]
pub(super) struct ParsedLdf {
    pub source_file: String,
    pub protocol_version: String,
    pub language_version: String,
    pub signals: HashMap<String, SignalDef>,
    pub frames: Vec<FrameDef>,
    pub encodings: HashMap<String, EncodingDef>,
    pub representations: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TokenKind {
    Eof,
    Identifier,
    Number,
    String,
    Symbol,
}

#[derive(Debug)]
struct Token {
    kind: TokenKind,
    text: String,
    pos: Position,
}

struct Lexer<'a> {
    filename: &'a str,
    data: &'a [u8],
    off: usize,
    line: u32,
    column: u32,
}

impl<'a> Lexer<'a> {
    fn new(filename: &'a str, data: &'a [u8]) -> Self {
        Self {
            filename,
            data,
            off: 0,
            line: 1,
            column: 1,
        }
    }

    fn next(&mut self) -> Result<Token, String> {
        self.skip_whitespace_and_comments()?;
        let pos = self.position();
        if self.off >= self.data.len() {
            return Ok(Token {
                kind: TokenKind::Eof,
                text: String::new(),
                pos,
            });
        }
        let c = self.data[self.off];
        if is_identifier_start(c) {
            let start = self.off;
            while self.off < self.data.len() && is_identifier_part(self.data[self.off]) {
                self.advance();
            }
            return Ok(Token {
                kind: TokenKind::Identifier,
                text: bytes_to_string(&self.data[start..self.off]),
                pos,
            });
        }
        if c == b'"' {
            return self.scan_string(pos);
        }
        if is_number_start(self.data, self.off) {
            return self.scan_number(pos);
        }
        if c == b'{'
            || c == b'}'
            || c == b':'
            || c == b','
            || c == b';'
            || c == b'='
            || (0x20..0x7f).contains(&c)
        {
            self.advance();
            return Ok(Token {
                kind: TokenKind::Symbol,
                text: bytes_to_string(&[c]),
                pos,
            });
        }
        Err(error(
            self.filename,
            pos,
            format!("unexpected character {c:#x}"),
        ))
    }

    fn skip_whitespace_and_comments(&mut self) -> Result<(), String> {
        while self.off < self.data.len() {
            let c = self.data[self.off];
            if is_ascii_space(c) {
                self.advance();
                continue;
            }
            if c != b'/' || self.off + 1 >= self.data.len() {
                return Ok(());
            }
            match self.data[self.off + 1] {
                b'/' => {
                    self.advance();
                    self.advance();
                    while self.off < self.data.len() && self.data[self.off] != b'\n' {
                        self.advance();
                    }
                }
                b'*' => {
                    let pos = self.position();
                    self.advance();
                    self.advance();
                    loop {
                        if self.off >= self.data.len() {
                            return Err(error(self.filename, pos, "unterminated block comment"));
                        }
                        if self.data[self.off] == b'*'
                            && self.off + 1 < self.data.len()
                            && self.data[self.off + 1] == b'/'
                        {
                            self.advance();
                            self.advance();
                            break;
                        }
                        self.advance();
                    }
                }
                _ => return Ok(()),
            }
        }
        Ok(())
    }

    fn scan_string(&mut self, pos: Position) -> Result<Token, String> {
        let start = self.off;
        self.advance();
        let mut escaped = false;
        while self.off < self.data.len() {
            let c = self.data[self.off];
            self.advance();
            if escaped {
                escaped = false;
                continue;
            }
            if c == b'\\' {
                escaped = true;
                continue;
            }
            if c == b'"' {
                let raw = bytes_to_string(&self.data[start..self.off]);
                let value = unquote(&raw, self.filename, pos)?;
                return Ok(Token {
                    kind: TokenKind::String,
                    text: value,
                    pos,
                });
            }
        }
        Err(error(self.filename, pos, "unterminated string"))
    }

    fn scan_number(&mut self, pos: Position) -> Result<Token, String> {
        let start = self.off;
        if self.data[self.off] == b'+' || self.data[self.off] == b'-' {
            self.advance();
        }
        if self.off + 1 < self.data.len()
            && self.data[self.off] == b'0'
            && (self.data[self.off + 1] == b'x' || self.data[self.off + 1] == b'X')
        {
            self.advance();
            self.advance();
            let digit_start = self.off;
            while self.off < self.data.len() && is_hex(self.data[self.off]) {
                self.advance();
            }
            if self.off == digit_start {
                return Err(error(self.filename, pos, "invalid hexadecimal number"));
            }
            return Ok(Token {
                kind: TokenKind::Number,
                text: bytes_to_string(&self.data[start..self.off]),
                pos,
            });
        }
        while self.off < self.data.len() && is_digit(self.data[self.off]) {
            self.advance();
        }
        if self.off < self.data.len() && self.data[self.off] == b'.' {
            self.advance();
            while self.off < self.data.len() && is_digit(self.data[self.off]) {
                self.advance();
            }
        }
        if self.off < self.data.len()
            && (self.data[self.off] == b'e' || self.data[self.off] == b'E')
        {
            self.advance();
            if self.off < self.data.len()
                && (self.data[self.off] == b'+' || self.data[self.off] == b'-')
            {
                self.advance();
            }
            let digit_start = self.off;
            while self.off < self.data.len() && is_digit(self.data[self.off]) {
                self.advance();
            }
            if self.off == digit_start {
                return Err(error(self.filename, pos, "invalid exponent"));
            }
        }
        Ok(Token {
            kind: TokenKind::Number,
            text: bytes_to_string(&self.data[start..self.off]),
            pos,
        })
    }

    fn advance(&mut self) {
        if self.data[self.off] == b'\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        self.off += 1;
    }

    fn position(&self) -> Position {
        Position {
            line: self.line,
            column: self.column,
        }
    }
}

struct Parser<'a> {
    filename: &'a str,
    lex: Lexer<'a>,
    lookahead: Option<Token>,
    file: ParsedLdf,
}

impl<'a> Parser<'a> {
    fn parse(filename: &'a str, data: &'a [u8]) -> Result<ParsedLdf, String> {
        let mut parser = Self {
            filename,
            lex: Lexer::new(filename, data),
            lookahead: None,
            file: ParsedLdf {
                source_file: filename.to_string(),
                protocol_version: String::new(),
                language_version: String::new(),
                signals: HashMap::new(),
                frames: Vec::new(),
                encodings: HashMap::new(),
                representations: HashMap::new(),
            },
        };
        parser.parse_file()?;
        Ok(parser.file)
    }

    fn parse_file(&mut self) -> Result<(), String> {
        loop {
            let token = self.peek()?;
            if token.kind == TokenKind::Eof {
                return Ok(());
            }
            let name = self.expect_kind(TokenKind::Identifier, "top-level section or property")?;
            match name.text.as_str() {
                "LIN_protocol_version" => {
                    self.file.protocol_version = self.parse_string_assignment()?;
                }
                "LIN_language_version" => {
                    self.file.language_version = self.parse_string_assignment()?;
                }
                "Signals" => self.parse_signals()?,
                "Frames" => self.parse_frames()?,
                "Signal_encoding_types" => self.parse_encoding_types()?,
                "Signal_representation" => self.parse_representations()?,
                _ => self.skip_top_level(name)?,
            }
        }
    }

    fn parse_string_assignment(&mut self) -> Result<String, String> {
        self.expect_symbol("=")?;
        let value = self.expect_kind(TokenKind::String, "quoted version")?;
        self.expect_symbol(";")?;
        Ok(value.text)
    }

    fn parse_signals(&mut self) -> Result<(), String> {
        self.expect_symbol("{")?;
        loop {
            if self.accept_symbol("}")? {
                return Ok(());
            }
            let name = self.expect_kind(TokenKind::Identifier, "signal name")?;
            if self.file.signals.contains_key(&name.text) {
                return Err(error(
                    self.filename,
                    name.pos,
                    format!("duplicate signal `{}`", name.text),
                ));
            }
            self.expect_symbol(":")?;
            let (length, _) = self.parse_uint(16)?;
            if length == 0 {
                return Err(error(
                    self.filename,
                    name.pos,
                    format!("signal `{}` has zero length", name.text),
                ));
            }
            self.expect_symbol(",")?;
            self.skip_initial_value()?;
            self.expect_symbol(",")?;
            self.expect_kind(TokenKind::Identifier, "signal publisher")?;
            loop {
                if self.accept_symbol(";")? {
                    break;
                }
                self.expect_symbol(",")?;
                self.expect_kind(TokenKind::Identifier, "signal subscriber")?;
            }
            self.file.signals.insert(
                name.text.clone(),
                SignalDef {
                    pos: name.pos,
                    name: name.text,
                    length: length as u16,
                },
            );
        }
    }

    fn skip_initial_value(&mut self) -> Result<(), String> {
        if !self.accept_symbol("{")? {
            self.parse_number()?;
            return Ok(());
        }
        if self.accept_symbol("}")? {
            return Ok(());
        }
        loop {
            self.parse_number()?;
            if self.accept_symbol("}")? {
                return Ok(());
            }
            self.expect_symbol(",")?;
        }
    }

    fn parse_frames(&mut self) -> Result<(), String> {
        self.expect_symbol("{")?;
        loop {
            if self.accept_symbol("}")? {
                return Ok(());
            }
            let name = self.expect_kind(TokenKind::Identifier, "frame name")?;
            self.expect_symbol(":")?;
            let (id, id_pos) = self.parse_uint(8)?;
            if id > 0x3f {
                return Err(error(
                    self.filename,
                    id_pos,
                    format!(
                        "frame `{}` ID {id:#x} is outside 6-bit LIN range",
                        name.text
                    ),
                ));
            }
            self.expect_symbol(",")?;
            self.expect_kind(TokenKind::Identifier, "frame publisher")?;
            let mut length = u64::from(legacy_frame_length(id as u8));
            let mut length_explicit = false;
            if self.accept_symbol(",")? {
                length_explicit = true;
                let (parsed, length_pos) = self.parse_uint(8)?;
                length = parsed;
                if length == 0 || length > 8 {
                    return Err(error(
                        self.filename,
                        length_pos,
                        format!("frame `{}` length {length} is outside 1..8", name.text),
                    ));
                }
            }
            self.expect_symbol("{")?;
            let mut frame = FrameDef {
                pos: name.pos,
                name: name.text,
                id: id as u8,
                length: length as u8,
                length_explicit,
                placements: Vec::new(),
            };
            loop {
                if self.accept_symbol("}")? {
                    break;
                }
                let signal = self.expect_kind(TokenKind::Identifier, "frame signal name")?;
                self.expect_symbol(",")?;
                let (start, _) = self.parse_uint(16)?;
                self.expect_symbol(";")?;
                frame.placements.push(SignalPlacement {
                    pos: signal.pos,
                    name: signal.text,
                    start: start as u16,
                });
            }
            self.file.frames.push(frame);
        }
    }

    fn parse_encoding_types(&mut self) -> Result<(), String> {
        self.expect_symbol("{")?;
        loop {
            if self.accept_symbol("}")? {
                return Ok(());
            }
            let name = self.expect_kind(TokenKind::Identifier, "encoding name")?;
            if self.file.encodings.contains_key(&name.text) {
                return Err(error(
                    self.filename,
                    name.pos,
                    format!("duplicate encoding `{}`", name.text),
                ));
            }
            self.expect_symbol("{")?;
            let mut encoding = EncodingDef {
                pos: name.pos,
                physical: Vec::new(),
                has_ascii: false,
                has_bcd: false,
            };
            loop {
                if self.accept_symbol("}")? {
                    break;
                }
                let kind = self.expect_kind(TokenKind::Identifier, "encoding value type")?;
                match kind.text.as_str() {
                    "logical_value" => self.parse_logical_encoding()?,
                    "physical_value" => {
                        encoding
                            .physical
                            .push(self.parse_physical_encoding(kind.pos)?);
                    }
                    "ascii_value" => {
                        encoding.has_ascii = true;
                        self.expect_symbol(";")?;
                    }
                    "bcd_value" => {
                        encoding.has_bcd = true;
                        self.expect_symbol(";")?;
                    }
                    other => {
                        return Err(error(
                            self.filename,
                            kind.pos,
                            format!("unsupported encoding value type `{other}`"),
                        ));
                    }
                }
            }
            self.file.encodings.insert(name.text, encoding);
        }
    }

    fn parse_logical_encoding(&mut self) -> Result<(), String> {
        self.expect_symbol(",")?;
        self.parse_number()?;
        if self.accept_symbol(",")? {
            self.expect_kind(TokenKind::String, "logical value text")?;
        }
        self.expect_symbol(";")?;
        Ok(())
    }

    fn parse_physical_encoding(&mut self, pos: Position) -> Result<PhysicalEncoding, String> {
        self.expect_symbol(",")?;
        self.parse_number()?;
        self.expect_symbol(",")?;
        self.parse_number()?;
        self.expect_symbol(",")?;
        let (scale, _) = self.parse_number()?;
        self.expect_symbol(",")?;
        let (offset, _) = self.parse_number()?;
        if self.accept_symbol(",")? {
            self.expect_kind(TokenKind::String, "physical value unit")?;
        }
        self.expect_symbol(";")?;
        Ok(PhysicalEncoding { pos, scale, offset })
    }

    fn parse_representations(&mut self) -> Result<(), String> {
        self.expect_symbol("{")?;
        loop {
            if self.accept_symbol("}")? {
                return Ok(());
            }
            let encoding = self.expect_kind(TokenKind::Identifier, "encoding name")?;
            self.expect_symbol(":")?;
            loop {
                let signal = self.expect_kind(TokenKind::Identifier, "represented signal name")?;
                if let Some(previous) = self.file.representations.get(&signal.text)
                    && previous != &encoding.text
                {
                    return Err(error(
                        self.filename,
                        signal.pos,
                        format!(
                            "signal `{}` has conflicting encodings `{previous}` and `{}`",
                            signal.text, encoding.text
                        ),
                    ));
                }
                self.file
                    .representations
                    .insert(signal.text, encoding.text.clone());
                if self.accept_symbol(";")? {
                    break;
                }
                self.expect_symbol(",")?;
            }
        }
    }

    fn skip_top_level(&mut self, name: Token) -> Result<(), String> {
        let next = self.peek()?;
        let text = next.text.clone();
        let pos = next.pos;
        match text.as_str() {
            ";" => {
                self.consume()?;
                Ok(())
            }
            "=" => {
                self.consume()?;
                self.skip_until_semicolon()
            }
            "{" => {
                self.consume()?;
                self.skip_balanced_block(name.pos)
            }
            other => Err(error(
                self.filename,
                pos,
                format!(
                    "cannot skip top-level item `{}` before `{other}`",
                    name.text
                ),
            )),
        }
    }

    fn skip_until_semicolon(&mut self) -> Result<(), String> {
        let mut depth = 0usize;
        loop {
            let token = self.consume()?;
            if token.kind == TokenKind::Eof {
                return Err(error(
                    self.filename,
                    token.pos,
                    "unexpected EOF while skipping assignment",
                ));
            }
            match token.text.as_str() {
                "{" => depth += 1,
                "}" => {
                    if depth == 0 {
                        return Err(error(
                            self.filename,
                            token.pos,
                            "unexpected } while skipping assignment",
                        ));
                    }
                    depth -= 1;
                }
                ";" if depth == 0 => return Ok(()),
                _ => {}
            }
        }
    }

    fn skip_balanced_block(&mut self, start: Position) -> Result<(), String> {
        let mut depth = 1usize;
        while depth > 0 {
            let token = self.consume()?;
            if token.kind == TokenKind::Eof {
                return Err(error(self.filename, start, "unterminated top-level block"));
            }
            match token.text.as_str() {
                "{" => depth += 1,
                "}" => depth -= 1,
                _ => {}
            }
        }
        Ok(())
    }

    fn parse_uint(&mut self, bits: u32) -> Result<(u64, Position), String> {
        let token = self.expect_kind(TokenKind::Number, "unsigned integer")?;
        let is_hex = token.text.starts_with("0x") || token.text.starts_with("0X");
        if !is_hex && token.text.contains(['.', 'e', 'E']) {
            return Err(error(
                self.filename,
                token.pos,
                format!("expected integer, got `{}`", token.text),
            ));
        }
        let value =
            u64::from_str_radix(strip_int_prefix(&token.text), if is_hex { 16 } else { 10 })
                .map_err(|_| {
                    error(
                        self.filename,
                        token.pos,
                        format!("invalid {bits}-bit unsigned integer `{}`", token.text),
                    )
                })?;
        let max = if bits >= 64 {
            u64::MAX
        } else {
            (1u64 << bits) - 1
        };
        if value > max {
            return Err(error(
                self.filename,
                token.pos,
                format!("invalid {bits}-bit unsigned integer `{}`", token.text),
            ));
        }
        Ok((value, token.pos))
    }

    fn parse_number(&mut self) -> Result<(f64, Position), String> {
        let token = self.expect_kind(TokenKind::Number, "number")?;
        let value = parse_float(&token.text).map_err(|_| {
            error(
                self.filename,
                token.pos,
                format!("invalid number `{}`", token.text),
            )
        })?;
        if !value.is_finite() {
            return Err(error(
                self.filename,
                token.pos,
                format!("non-finite number `{}`", token.text),
            ));
        }
        Ok((value, token.pos))
    }

    fn expect_symbol(&mut self, symbol: &str) -> Result<Token, String> {
        let token = self.consume()?;
        if token.kind != TokenKind::Symbol || token.text != symbol {
            return Err(error(
                self.filename,
                token.pos,
                format!("expected `{symbol}`, got `{}`", token.text),
            ));
        }
        Ok(token)
    }

    fn accept_symbol(&mut self, symbol: &str) -> Result<bool, String> {
        let token = self.peek()?;
        if token.kind != TokenKind::Symbol || token.text != symbol {
            return Ok(false);
        }
        self.consume()?;
        Ok(true)
    }

    fn expect_kind(&mut self, kind: TokenKind, description: &str) -> Result<Token, String> {
        let token = self.consume()?;
        if token.kind != kind {
            return Err(error(
                self.filename,
                token.pos,
                format!("expected {description}, got `{}`", token.text),
            ));
        }
        Ok(token)
    }

    fn peek(&mut self) -> Result<&Token, String> {
        if self.lookahead.is_none() {
            self.lookahead = Some(self.lex.next()?);
        }
        self.lookahead.as_ref().ok_or_else(|| {
            error(
                self.filename,
                self.lex.position(),
                "internal LDF parser error",
            )
        })
    }

    fn consume(&mut self) -> Result<Token, String> {
        if let Some(token) = self.lookahead.take() {
            Ok(token)
        } else {
            self.lex.next()
        }
    }
}

pub(super) fn parse_ldf(source_file: &str, data: &[u8]) -> Result<ParsedLdf, String> {
    Parser::parse(source_file, data)
}

fn legacy_frame_length(id: u8) -> u8 {
    match id {
        0..=31 => 2,
        32..=47 => 4,
        _ => 8,
    }
}

pub(super) fn error(filename: &str, pos: Position, message: impl std::fmt::Display) -> String {
    format!("{}:{}:{}: {message}", filename, pos.line, pos.column)
}

fn bytes_to_string(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn unquote(raw: &str, filename: &str, pos: Position) -> Result<String, String> {
    let inner = raw
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .ok_or_else(|| error(filename, pos, "invalid string"))?;
    if !inner.contains('\\') {
        return Ok(inner.to_string());
    }
    let mut out = String::new();
    let mut chars = inner.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('\\') => out.push('\\'),
            Some('"') => out.push('"'),
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some('t') => out.push('\t'),
            Some(other) => {
                return Err(error(
                    filename,
                    pos,
                    format!("invalid string escape \\{other}"),
                ));
            }
            None => return Err(error(filename, pos, "invalid string")),
        }
    }
    Ok(out)
}

fn strip_int_prefix(text: &str) -> &str {
    text.strip_prefix("0x")
        .or_else(|| text.strip_prefix("0X"))
        .unwrap_or(text)
}

fn parse_float(text: &str) -> Result<f64, ()> {
    let (sign, raw) = if let Some(rest) = text.strip_prefix('-') {
        (-1.0, rest)
    } else if let Some(rest) = text.strip_prefix('+') {
        (1.0, rest)
    } else {
        (1.0, text)
    };
    if let Some(hex) = raw.strip_prefix("0x").or_else(|| raw.strip_prefix("0X")) {
        let value = u64::from_str_radix(hex, 16).map_err(|_| ())?;
        return Ok(sign * value as f64);
    }
    raw.parse::<f64>().map(|value| sign * value).map_err(|_| ())
}

fn is_identifier_start(c: u8) -> bool {
    c == b'_' || c.is_ascii_alphabetic()
}

fn is_identifier_part(c: u8) -> bool {
    is_identifier_start(c) || is_digit(c)
}

fn is_digit(c: u8) -> bool {
    c.is_ascii_digit()
}

fn is_hex(c: u8) -> bool {
    c.is_ascii_hexdigit()
}

fn is_number_start(data: &[u8], off: usize) -> bool {
    is_digit(data[off])
        || ((data[off] == b'+' || data[off] == b'-')
            && off + 1 < data.len()
            && is_digit(data[off + 1]))
}

fn is_ascii_space(c: u8) -> bool {
    matches!(c, b'\t' | b'\n' | 0x0b | b'\x0c' | b'\r' | b' ')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lexer_comments_numbers_and_strings() {
        let mut lexer = Lexer::new(
            "test.ldf",
            b"// line\nA /* block */ = -1.25e+2; B=0x2a; C=\"{ // text\";",
        );
        let want = [
            "A",
            "=",
            "-1.25e+2",
            ";",
            "B",
            "=",
            "0x2a",
            ";",
            "C",
            "=",
            "{ // text",
            ";",
        ];
        for expected in want {
            let token = lexer.next().expect("token");
            assert_eq!(token.text, expected);
        }
        assert_eq!(lexer.next().expect("eof").kind, TokenKind::Eof);
    }

    #[test]
    fn lexer_reports_position() {
        let mut lexer = Lexer::new("test.ldf", b"A {\n  \x00");
        lexer.next().expect("ident");
        lexer.next().expect("brace");
        let err = lexer.next().expect_err("nul");
        assert!(err.contains("test.ldf:2:3"), "{err}");
    }

    #[test]
    fn error_formats_position() {
        assert_eq!(
            error(
                "body.ldf",
                Position { line: 4, column: 2 },
                "duplicate signal `Temp`"
            ),
            "body.ldf:4:2: duplicate signal `Temp`"
        );
    }
}
