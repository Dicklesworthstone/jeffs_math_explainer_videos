//! The owned YAML-subset parser: the actual shipped config-file shapes,
//! exactly — not YAML-the-standard.
//!
//! The Reference ships two YAML documents (`default_config.yml`,
//! `tex_templates.yml`) and users write `custom_config.yml` in the same
//! dialect. Everything those files actually use is supported precisely:
//!
//! - **Block mappings** nested by space indentation (tabs are a precise
//!   error), with consistent sibling indentation enforced.
//! - **Plain scalars** resolved with PyYAML 1.1's rules for the lexemes the
//!   files use: `True`/`False`-family booleans, decimal integers and floats,
//!   `~`/`null` and empty values, everything else a string — including the
//!   Reference's **tuple-strings** (`(1920, 1080)`), which are plain strings
//!   here and typed by the config layer, exactly as the Reference
//!   `literal_eval`s them.
//! - **Quoted scalars** (double with the common escapes, single with `''`),
//!   single-line.
//! - **Literal block scalars** `|` with all three chomping indicators
//!   (`|`, `|-`, `|+`) — the `tex_templates.yml` preamble shape.
//! - **Comments** (full-line at any indent; trailing after a space).
//! - **Duplicate keys**: defined as last-wins **with a warning** — PyYAML's
//!   silent behavior, made visible.
//!
//! Everything else — flow collections, block sequences, folded scalars,
//! anchors/aliases, tags, directives, multi-document streams, multi-line
//! flow scalars, complex keys — is a **named, positioned diagnostic**, never
//! a silent misparse. This is an owned parser under the governed closure (no
//! yaml crates), with parser budgets from day one (§16.5): input size and
//! nesting depth are limited, hostile input yields errors, and
//! [`fuzz_probe`] is the registered fuzz entry point (it must never panic).

use core::fmt;

/// Parser resource budgets (§16.5). Defaults are far above any real config
/// file yet bounded enough that hostile input cannot amplify.
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    /// Maximum input size in bytes.
    pub max_bytes: usize,
    /// Maximum mapping nesting depth.
    pub max_depth: usize,
}

impl Limits {
    /// 1 MiB and depth 32: the shipped files are ~6 KiB and depth 3.
    pub const DEFAULT: Self = Self {
        max_bytes: 1024 * 1024,
        max_depth: 32,
    };
}

impl Default for Limits {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// A parsed YAML-subset value. Maps preserve insertion order (the Reference
/// iterates `directories.subdirs` in file order, so order is semantics).
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    /// An empty value, `~`, or a `null` lexeme.
    Null,
    /// A YAML 1.1 boolean lexeme (`True`, `false`, `yes`, `OFF`, …).
    Bool(bool),
    /// A decimal integer.
    Int(i64),
    /// A decimal float.
    Float(f64),
    /// Any other scalar, including tuple-strings like `(1920, 1080)`.
    Str(String),
    /// A block mapping, in insertion order.
    Map(Vec<(String, Value)>),
}

impl Value {
    /// Child of a map by key.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&Value> {
        match self {
            Self::Map(entries) => entries.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    /// Descend a dotted path (`"camera.resolution"`).
    #[must_use]
    pub fn get_path(&self, path: &str) -> Option<&Value> {
        let mut cur = self;
        for part in path.split('.') {
            cur = cur.get(part)?;
        }
        Some(cur)
    }

    /// The map entries, if this is a map.
    #[must_use]
    pub fn as_map(&self) -> Option<&[(String, Value)]> {
        match self {
            Self::Map(entries) => Some(entries),
            _ => None,
        }
    }

    /// A short name for diagnostics ("string", "integer", …).
    #[must_use]
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Null => "null",
            Self::Bool(_) => "boolean",
            Self::Int(_) => "integer",
            Self::Float(_) => "float",
            Self::Str(_) => "string",
            Self::Map(_) => "mapping",
        }
    }
}

/// A non-fatal condition surfaced to the user (the defined duplicate-key
/// policy: last-wins with a warning).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Warning {
    /// 1-based line of the condition.
    pub line: u32,
    /// Human-readable description.
    pub message: String,
}

impl fmt::Display for Warning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

/// A parse failure: position plus a precise, expected-vs-found description.
/// The parser never panics and never silently misparses — everything outside
/// the subset lands here with its name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseError {
    /// 1-based line.
    pub line: u32,
    /// 1-based column (byte offset within the line).
    pub col: u32,
    /// What went wrong.
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}, col {}: {}", self.line, self.col, self.message)
    }
}

impl std::error::Error for ParseError {}

fn err(line: u32, col: u32, message: impl Into<String>) -> ParseError {
    ParseError {
        line,
        col,
        message: message.into(),
    }
}

/// One physical line, pre-split.
struct Line<'a> {
    /// 1-based line number.
    no: u32,
    /// Leading-space count (indentation).
    indent: usize,
    /// Content after the indentation (may be empty).
    content: &'a str,
}

impl Line<'_> {
    fn is_blank(&self) -> bool {
        self.content.is_empty()
    }
    fn is_comment(&self) -> bool {
        self.content.starts_with('#')
    }
}

/// Parse a document with default [`Limits`].
///
/// # Errors
/// [`ParseError`] with position and an expected-vs-found message.
pub fn parse(src: &str) -> Result<(Value, Vec<Warning>), ParseError> {
    parse_with_limits(src, Limits::DEFAULT)
}

/// Parse a document under explicit resource budgets.
///
/// # Errors
/// [`ParseError`]; hostile or over-budget input is an error, never a panic.
pub fn parse_with_limits(src: &str, limits: Limits) -> Result<(Value, Vec<Warning>), ParseError> {
    if src.len() > limits.max_bytes {
        return Err(err(
            1,
            1,
            format!(
                "input is {} bytes, over the {}-byte budget",
                src.len(),
                limits.max_bytes
            ),
        ));
    }

    let mut lines = Vec::new();
    for (i, raw) in src.split('\n').enumerate() {
        let no = u32::try_from(i + 1).unwrap_or(u32::MAX);
        let raw = raw.strip_suffix('\r').unwrap_or(raw);
        // Indentation must be spaces; a tab here is the classic YAML trap
        // and gets a precise refusal.
        let mut indent = 0usize;
        for b in raw.bytes() {
            match b {
                b' ' => indent += 1,
                b'\t' => {
                    return Err(err(
                        no,
                        u32::try_from(indent + 1).unwrap_or(u32::MAX),
                        "tab character in indentation; the subset indents with spaces only",
                    ));
                }
                _ => break,
            }
        }
        let content = &raw[indent..];
        // Control characters (other than tab inside content) are hostile
        // input, not configuration.
        if let Some(pos) = content.bytes().position(|b| b < 0x20 && b != b'\t') {
            return Err(err(
                no,
                u32::try_from(indent + pos + 1).unwrap_or(u32::MAX),
                format!(
                    "control character 0x{:02x} in content; not valid in the subset",
                    content.as_bytes()[pos]
                ),
            ));
        }
        lines.push(Line {
            no,
            indent,
            content,
        });
    }

    let mut parser = Parser {
        lines,
        idx: 0,
        limits,
        warnings: Vec::new(),
    };
    parser.check_top_level_markers()?;
    // The root mapping's indentation is wherever its first entry sits
    // (the shipped files use column 1, but a uniformly indented document
    // is unambiguous and accepted).
    let root_indent = parser.peek_meaningful().map_or(0, |l| l.indent);
    let root = parser.parse_mapping(root_indent, 1)?;
    // Anything left is a line the root mapping could not own.
    if let Some(line) = parser.peek_meaningful() {
        return Err(err(
            line.no,
            u32::try_from(line.indent + 1).unwrap_or(u32::MAX),
            format!(
                "indentation of {} does not match any open mapping level",
                line.indent
            ),
        ));
    }
    Ok((Value::Map(root), parser.warnings))
}

struct Parser<'a> {
    lines: Vec<Line<'a>>,
    idx: usize,
    limits: Limits,
    warnings: Vec<Warning>,
}

impl<'a> Parser<'a> {
    /// Reject document/stream syntax up front with named diagnostics.
    fn check_top_level_markers(&self) -> Result<(), ParseError> {
        for line in &self.lines {
            if line.indent == 0
                && (line.content == "---"
                    || line.content.starts_with("--- ")
                    || line.content == "...")
            {
                return Err(err(
                    line.no,
                    1,
                    "document markers (---, ...) are outside the subset: config files are single documents",
                ));
            }
            if line.indent == 0 && line.content.starts_with('%') {
                return Err(err(
                    line.no,
                    1,
                    "YAML directives (%…) are outside the subset",
                ));
            }
        }
        Ok(())
    }

    /// The next non-blank, non-comment line without consuming it.
    fn peek_meaningful(&self) -> Option<&Line<'a>> {
        self.lines[self.idx..]
            .iter()
            .find(|l| !l.is_blank() && !l.is_comment())
    }

    /// Parse one block mapping whose entries sit at exactly `indent`.
    fn parse_mapping(
        &mut self,
        indent: usize,
        depth: usize,
    ) -> Result<Vec<(String, Value)>, ParseError> {
        if depth > self.limits.max_depth {
            let line = self.peek_meaningful().map_or(0, |l| l.no);
            return Err(err(
                line,
                1,
                format!(
                    "nesting deeper than the {}-level budget",
                    self.limits.max_depth
                ),
            ));
        }
        let mut entries: Vec<(String, Value)> = Vec::new();
        let mut first_lines: Vec<(String, u32)> = Vec::new();

        loop {
            // Skip blanks and comments (any indentation).
            while self
                .lines
                .get(self.idx)
                .is_some_and(|l| l.is_blank() || l.is_comment())
            {
                self.idx += 1;
            }
            let Some(line) = self.lines.get(self.idx) else {
                break;
            };
            if line.indent < indent {
                break; // Pops back to an outer mapping.
            }
            let (no, line_indent) = (line.no, line.indent);
            let col = |offset: usize| u32::try_from(line_indent + offset + 1).unwrap_or(u32::MAX);
            if line_indent > indent {
                return Err(err(
                    no,
                    col(0),
                    format!(
                        "unexpected indentation: this mapping's entries start at column {}, found column {}",
                        indent + 1,
                        line_indent + 1
                    ),
                ));
            }

            let content = line.content;
            self.reject_unsupported_entry(content, no, line_indent)?;

            // ---- the key ----
            let (key, after_key) = self.parse_key(content, no, line_indent)?;

            // ---- the value ----
            let rest = after_key.trim_start_matches(' ');
            let rest_offset = content.len() - rest.len();
            self.idx += 1;
            let value = self.parse_value(rest, no, line_indent, rest_offset, depth)?;

            // The defined duplicate-key policy: last wins, with a warning.
            if let Some(slot) = entries.iter_mut().find(|(k, _)| *k == key) {
                let first = first_lines
                    .iter()
                    .find(|(k, _)| *k == key)
                    .map_or(0, |(_, n)| *n);
                self.warnings.push(Warning {
                    line: no,
                    message: format!(
                        "duplicate key {key:?} overrides the value from line {first} (last-wins)"
                    ),
                });
                slot.1 = value;
            } else {
                first_lines.push((key.clone(), no));
                entries.push((key, value));
            }
        }
        Ok(entries)
    }

    /// Name the out-of-subset constructs that can begin a mapping entry.
    fn reject_unsupported_entry(
        &self,
        content: &str,
        no: u32,
        indent: usize,
    ) -> Result<(), ParseError> {
        let col = u32::try_from(indent + 1).unwrap_or(u32::MAX);
        if content == "-" || content.starts_with("- ") {
            return Err(err(
                no,
                col,
                "block sequences (- item) are outside the subset: the shipped config shapes are nested mappings only",
            ));
        }
        if content.starts_with("? ") {
            return Err(err(
                no,
                col,
                "complex mapping keys (? …) are outside the subset",
            ));
        }
        Ok(())
    }

    /// Split `key: rest` (plain or quoted key), returning the key and the
    /// remainder after the colon.
    fn parse_key(
        &self,
        content: &'a str,
        no: u32,
        indent: usize,
    ) -> Result<(String, &'a str), ParseError> {
        let col = |offset: usize| u32::try_from(indent + offset + 1).unwrap_or(u32::MAX);
        if content.starts_with('"') || content.starts_with('\'') {
            let (key, consumed) = parse_quoted(content, no, indent)?;
            let rest = &content[consumed..];
            let rest = rest.trim_start_matches(' ');
            let Some(after) = rest.strip_prefix(':') else {
                return Err(err(
                    no,
                    col(consumed),
                    format!("expected ':' after quoted key, found {rest:?}"),
                ));
            };
            if !after.is_empty() && !after.starts_with(' ') {
                return Err(err(
                    no,
                    col(content.len() - after.len()),
                    "expected a space or end of line after ':'",
                ));
            }
            return Ok((key, after));
        }
        // Plain key: up to the first ':' that ends it (followed by a space
        // or the end of the line).
        let bytes = content.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b':' && (i + 1 == bytes.len() || bytes[i + 1] == b' ') {
                let key = content[..i].trim_end_matches(' ');
                if key.is_empty() {
                    return Err(err(no, col(0), "empty mapping key"));
                }
                return Ok((key.to_owned(), &content[i + 1..]));
            }
            i += 1;
        }
        Err(err(
            no,
            col(content.len()),
            format!(
                "expected \"key: value\" or \"key:\", found no ':' in {:?}",
                truncate_for_message(content)
            ),
        ))
    }

    /// Parse the value part of an entry (after `key:`).
    fn parse_value(
        &mut self,
        rest: &str,
        no: u32,
        key_indent: usize,
        rest_offset: usize,
        depth: usize,
    ) -> Result<Value, ParseError> {
        let col =
            |extra: usize| u32::try_from(key_indent + rest_offset + extra + 1).unwrap_or(u32::MAX);

        // Empty (or comment-only) after the colon: either a nested mapping
        // or an explicit null.
        if rest.is_empty() || rest.starts_with('#') {
            if let Some(next) = self.peek_meaningful()
                && next.indent > key_indent
            {
                let child_indent = next.indent;
                return Ok(Value::Map(self.parse_mapping(child_indent, depth + 1)?));
            }
            return Ok(Value::Null);
        }

        match rest.as_bytes()[0] {
            b'|' => self.parse_literal_block(rest, no, key_indent, col(0)),
            b'>' => Err(err(
                no,
                col(0),
                "folded block scalars (>) are outside the subset; use the literal style (|)",
            )),
            b'[' | b'{' => Err(err(
                no,
                col(0),
                "flow collections ([…], {…}) are outside the subset: the shipped config shapes use block mappings and tuple-strings",
            )),
            b'&' | b'*' => Err(err(
                no,
                col(0),
                "anchors and aliases (&, *) are outside the subset",
            )),
            b'!' => Err(err(no, col(0), "tags (!…) are outside the subset")),
            b'"' | b'\'' => {
                let (value, consumed) = parse_quoted(rest, no, key_indent + rest_offset)?;
                let tail = rest[consumed..].trim_start_matches(' ');
                if !tail.is_empty() && !tail.starts_with('#') {
                    return Err(err(
                        no,
                        col(consumed),
                        format!(
                            "unexpected content after quoted scalar: {:?}",
                            truncate_for_message(tail)
                        ),
                    ));
                }
                Ok(Value::Str(value))
            }
            _ => {
                // Plain scalar: runs to a trailing comment (space before '#')
                // or the end of the line.
                let cut = find_trailing_comment(rest);
                let text = rest[..cut].trim_end_matches(' ');
                Ok(resolve_plain_scalar(text, no, col(0))?)
            }
        }
    }

    /// Literal block scalar (`|`, `|-`, `|+`), the `tex_templates.yml`
    /// preamble shape.
    fn parse_literal_block(
        &mut self,
        header: &str,
        no: u32,
        key_indent: usize,
        header_col: u32,
    ) -> Result<Value, ParseError> {
        // Header: '|' + optional chomping indicator + optional comment.
        let after = &header[1..];
        let (chomp, after) = match after.as_bytes().first() {
            Some(b'-') => (Chomp::Strip, &after[1..]),
            Some(b'+') => (Chomp::Keep, &after[1..]),
            _ => (Chomp::Clip, after),
        };
        let tail = after.trim_start_matches(' ');
        if !tail.is_empty() && !tail.starts_with('#') {
            if tail.bytes().next().is_some_and(|b| b.is_ascii_digit()) {
                return Err(err(
                    no,
                    header_col,
                    "explicit block-scalar indentation indicators (|2) are outside the subset",
                ));
            }
            return Err(err(
                no,
                header_col,
                format!(
                    "unexpected content after block-scalar header: {:?}",
                    truncate_for_message(tail)
                ),
            ));
        }

        // Collect the block: blank lines and lines indented deeper than the
        // key. The first non-blank line fixes the content indentation.
        let mut content_indent: Option<usize> = None;
        let mut collected: Vec<String> = Vec::new();
        while let Some(line) = self.lines.get(self.idx) {
            if line.is_blank() {
                collected.push(String::new());
                self.idx += 1;
                continue;
            }
            if line.indent <= key_indent {
                break;
            }
            let ci = match content_indent {
                Some(ci) => {
                    if line.indent < ci {
                        return Err(err(
                            line.no,
                            u32::try_from(line.indent + 1).unwrap_or(u32::MAX),
                            format!(
                                "block-scalar line indented at column {} under a block indented at column {}",
                                line.indent + 1,
                                ci + 1
                            ),
                        ));
                    }
                    ci
                }
                None => {
                    content_indent = Some(line.indent);
                    line.indent
                }
            };
            // Deeper-indented lines keep their extra spaces, literally.
            let extra = line.indent - ci;
            collected.push(format!("{}{}", " ".repeat(extra), line.content));
            self.idx += 1;
        }

        // Trailing blank lines are subject to chomping; blanks before the
        // first content line are content only if content exists at all.
        while collected.last().is_some_and(String::is_empty) && chomp != Chomp::Keep {
            collected.pop();
        }
        let mut text = collected.join("\n");
        match chomp {
            Chomp::Strip => {}
            Chomp::Clip => {
                if !text.is_empty() {
                    text.push('\n');
                }
            }
            Chomp::Keep => {
                text.push('\n');
            }
        }
        Ok(Value::Str(text))
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Chomp {
    /// `|-`: drop every trailing newline.
    Strip,
    /// `|`: exactly one trailing newline.
    Clip,
    /// `|+`: keep all trailing newlines.
    Keep,
}

/// The byte offset in `rest` where a trailing comment begins (a `#` at the
/// start or preceded by a space), or `rest.len()`.
fn find_trailing_comment(rest: &str) -> usize {
    let bytes = rest.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        if b == b'#' && (i == 0 || bytes[i - 1] == b' ') {
            return i;
        }
    }
    rest.len()
}

/// Parse a quoted scalar starting at `content[0]` (`"` or `'`). Returns the
/// decoded string and the bytes consumed (including quotes). Single-line
/// only: an unterminated quote is a precise error, not a continuation.
fn parse_quoted(content: &str, no: u32, indent: usize) -> Result<(String, usize), ParseError> {
    let col = |offset: usize| u32::try_from(indent + offset + 1).unwrap_or(u32::MAX);
    let bytes = content.as_bytes();
    let quote = bytes[0];
    let mut out = String::new();
    let mut i = 1;
    while i < bytes.len() {
        let b = bytes[i];
        if b == quote {
            if quote == b'\'' && bytes.get(i + 1) == Some(&b'\'') {
                out.push('\''); // '' is the single-quote escape.
                i += 2;
                continue;
            }
            return Ok((out, i + 1));
        }
        if quote == b'"' && b == b'\\' {
            let Some(&esc) = bytes.get(i + 1) else {
                return Err(err(no, col(i), "dangling escape at end of line"));
            };
            match esc {
                b'"' => out.push('"'),
                b'\\' => out.push('\\'),
                b'n' => out.push('\n'),
                b't' => out.push('\t'),
                b'r' => out.push('\r'),
                b'0' => out.push('\0'),
                b'u' => {
                    let hex = content
                        .get(i + 2..i + 6)
                        .ok_or_else(|| err(no, col(i), "\\u escape needs four hex digits"))?;
                    let cp = u32::from_str_radix(hex, 16)
                        .map_err(|_| err(no, col(i), format!("bad \\u escape {hex:?}")))?;
                    let ch = char::from_u32(cp).ok_or_else(|| {
                        err(no, col(i), format!("\\u{hex} is not a valid character"))
                    })?;
                    out.push(ch);
                    i += 6;
                    continue;
                }
                other => {
                    return Err(err(
                        no,
                        col(i),
                        format!("unsupported escape \\{}", other as char),
                    ));
                }
            }
            i += 2;
            continue;
        }
        // Multi-byte UTF-8 passes through untouched.
        let ch_len = content[i..].chars().next().map_or(1, char::len_utf8);
        out.push_str(&content[i..i + ch_len]);
        i += ch_len;
    }
    Err(err(
        no,
        col(content.len()),
        format!(
            "unterminated {} quote (multi-line quoted scalars are outside the subset)",
            if quote == b'"' { "double" } else { "single" }
        ),
    ))
}

/// Type a plain scalar with PyYAML 1.1's resolution for the lexeme families
/// the shipped files use.
fn resolve_plain_scalar(text: &str, no: u32, col: u32) -> Result<Value, ParseError> {
    match text {
        "" | "~" | "null" | "Null" | "NULL" => return Ok(Value::Null),
        "true" | "True" | "TRUE" | "yes" | "Yes" | "YES" | "on" | "On" | "ON" => {
            return Ok(Value::Bool(true));
        }
        "false" | "False" | "FALSE" | "no" | "No" | "NO" | "off" | "Off" | "OFF" => {
            return Ok(Value::Bool(false));
        }
        _ => {}
    }
    if is_int_lexeme(text) {
        return match text.parse::<i64>() {
            Ok(v) => Ok(Value::Int(v)),
            Err(_) => Err(err(
                no,
                col,
                format!("integer {text:?} does not fit in 64 bits"),
            )),
        };
    }
    if is_float_lexeme(text) {
        // The lexeme shape guarantees `parse` succeeds.
        if let Ok(v) = text.parse::<f64>() {
            return Ok(Value::Float(v));
        }
    }
    Ok(Value::Str(text.to_owned()))
}

/// `[-+]?[0-9]+`
fn is_int_lexeme(text: &str) -> bool {
    let digits = text.strip_prefix(['-', '+']).unwrap_or(text);
    !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit())
}

/// `[-+]? ( [0-9]+ '.' [0-9]* | '.' [0-9]+ ) ( [eE] [-+]? [0-9]+ )?`
///
/// A dot is required, matching PyYAML 1.1 (where `1e3` resolves as a plain
/// string, a known quirk kept deliberately: same input, same type).
fn is_float_lexeme(text: &str) -> bool {
    let rest = text.strip_prefix(['-', '+']).unwrap_or(text);
    let (mantissa, exponent) = match rest.find(['e', 'E']) {
        Some(pos) => (&rest[..pos], Some(&rest[pos + 1..])),
        None => (rest, None),
    };
    let Some(dot) = mantissa.find('.') else {
        return false;
    };
    let (int_part, frac_part) = (&mantissa[..dot], &mantissa[dot + 1..]);
    let int_ok = int_part.bytes().all(|b| b.is_ascii_digit());
    let frac_ok = frac_part.bytes().all(|b| b.is_ascii_digit());
    if !(int_ok && frac_ok && (!int_part.is_empty() || !frac_part.is_empty())) {
        return false;
    }
    match exponent {
        None => true,
        Some(exp) => {
            let exp = exp.strip_prefix(['-', '+']).unwrap_or(exp);
            !exp.is_empty() && exp.bytes().all(|b| b.is_ascii_digit())
        }
    }
}

fn truncate_for_message(s: &str) -> String {
    const MAX: usize = 40;
    if s.len() <= MAX {
        s.to_owned()
    } else {
        let mut end = MAX;
        while !s.is_char_boundary(end) {
            end -= 1;
        }
        format!("{}…", &s[..end])
    }
}

/// The Reference's `merge_dicts_recursively`, exactly: later wins; when both
/// sides are mappings the merge recurses; key order is the union in
/// first-appearance order (base first, new keys appended).
#[must_use]
pub fn merge(base: Value, over: Value) -> Value {
    match (base, over) {
        (Value::Map(mut base_entries), Value::Map(over_entries)) => {
            for (key, over_value) in over_entries {
                if let Some(slot) = base_entries.iter_mut().find(|(k, _)| *k == key) {
                    let current = std::mem::replace(&mut slot.1, Value::Null);
                    slot.1 = merge(current, over_value);
                } else {
                    base_entries.push((key, over_value));
                }
            }
            Value::Map(base_entries)
        }
        (_, over) => over,
    }
}

/// Fuzz-facing entry point (registered for the W10 fuzzing campaign; see
/// also fm-ntp's harness infrastructure): parse arbitrary bytes under the
/// default budgets. Must **never panic** — every failure path is a
/// [`ParseError`]. Returns `true` iff the bytes parsed as a document.
#[must_use]
pub fn fuzz_probe(bytes: &[u8]) -> bool {
    let Ok(src) = core::str::from_utf8(bytes) else {
        return false;
    };
    parse(src).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_ok(src: &str) -> Value {
        let (value, _warnings) = parse(src).expect("parse");
        value
    }

    #[test]
    fn scalars_resolve_like_pyyaml() {
        let v = parse_ok(
            "a: True\nb: False\nc: 144\nd: 1.0\ne: UR\nf: \"#333333\"\ng: (1920, 1080)\nh:\ni: ~\nj: 1e3\nk: -0.5\nl: yes\n",
        );
        assert_eq!(v.get("a"), Some(&Value::Bool(true)));
        assert_eq!(v.get("b"), Some(&Value::Bool(false)));
        assert_eq!(v.get("c"), Some(&Value::Int(144)));
        assert_eq!(v.get("d"), Some(&Value::Float(1.0)));
        assert_eq!(v.get("e"), Some(&Value::Str("UR".into())));
        assert_eq!(v.get("f"), Some(&Value::Str("#333333".into())));
        assert_eq!(v.get("g"), Some(&Value::Str("(1920, 1080)".into())));
        assert_eq!(v.get("h"), Some(&Value::Null));
        assert_eq!(v.get("i"), Some(&Value::Null));
        // PyYAML 1.1 quirk kept: no dot means no float.
        assert_eq!(v.get("j"), Some(&Value::Str("1e3".into())));
        assert_eq!(v.get("k"), Some(&Value::Float(-0.5)));
        assert_eq!(v.get("l"), Some(&Value::Bool(true)));
    }

    #[test]
    fn nested_mappings_and_comments() {
        let v = parse_ok(
            "# full-line comment\ndirectories:\n  base: \"\"\n  subdirs:\n    output: \"videos\"  # trailing comment\n    # interior comment\n    data: \"data\"\nwindow:\n  full_screen: False\n",
        );
        assert_eq!(
            v.get_path("directories.subdirs.output"),
            Some(&Value::Str("videos".into()))
        );
        assert_eq!(
            v.get_path("directories.subdirs.data"),
            Some(&Value::Str("data".into()))
        );
        assert_eq!(v.get_path("window.full_screen"), Some(&Value::Bool(false)));
    }

    #[test]
    fn literal_blocks_with_all_chomping_modes() {
        let strip = parse_ok("p: |-\n  line one\n  line two\n\n\nq: 1\n");
        assert_eq!(
            strip.get("p"),
            Some(&Value::Str("line one\nline two".into()))
        );

        let clip = parse_ok("p: |\n  line one\n  line two\n\n\nq: 1\n");
        assert_eq!(
            clip.get("p"),
            Some(&Value::Str("line one\nline two\n".into()))
        );

        let keep = parse_ok("p: |+\n  line one\n\n\nq: 1\n");
        assert_eq!(keep.get("p"), Some(&Value::Str("line one\n\n\n".into())));
    }

    #[test]
    fn literal_block_preserves_interior_blanks_and_deeper_indent() {
        let v = parse_ok("p: |-\n  first\n\n    indented\n  last\n");
        assert_eq!(
            v.get("p"),
            Some(&Value::Str("first\n\n  indented\nlast".into()))
        );
    }

    #[test]
    fn tex_templates_shape_parses() {
        // The exact shape of the shipped tex_templates.yml.
        let v = parse_ok(
            "default:\n  description: \"\"\n  compiler: latex\n  preamble: |-\n    \\usepackage{amsmath}\n    %% comment inside block is literal\n    \\DeclareMathSymbol{\\minus}{\\mathbin}{AMSa}{\"39}\nempty:\n  description: \"\"\n  preamble: \"\"\n",
        );
        assert_eq!(
            v.get_path("default.preamble"),
            Some(&Value::Str(
                "\\usepackage{amsmath}\n%% comment inside block is literal\n\\DeclareMathSymbol{\\minus}{\\mathbin}{AMSa}{\"39}".into()
            ))
        );
        assert_eq!(
            v.get_path("empty.preamble"),
            Some(&Value::Str(String::new()))
        );
    }

    #[test]
    fn duplicate_keys_are_last_wins_with_a_warning() {
        let (v, warnings) = parse("a: 1\nb: 2\na: 3\n").expect("parse");
        assert_eq!(v.get("a"), Some(&Value::Int(3)));
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].line, 3);
        assert!(warnings[0].message.contains("duplicate key \"a\""));
        assert!(warnings[0].message.contains("line 1"));
        // Position is preserved (Python dict update semantics).
        assert_eq!(
            v.as_map()
                .unwrap()
                .iter()
                .map(|(k, _)| k.as_str())
                .collect::<Vec<_>>(),
            vec!["a", "b"]
        );
    }

    #[test]
    fn quoted_scalars_and_escapes() {
        let v = parse_ok(
            "a: \"with \\\"quotes\\\" and \\n newline\"\nb: 'single ''escaped'''\nc: \"caf\\u00e9\"\nd: \"直接\"\n",
        );
        assert_eq!(
            v.get("a"),
            Some(&Value::Str("with \"quotes\" and \n newline".into()))
        );
        assert_eq!(v.get("b"), Some(&Value::Str("single 'escaped'".into())));
        assert_eq!(v.get("c"), Some(&Value::Str("café".into())));
        assert_eq!(v.get("d"), Some(&Value::Str("直接".into())));
    }

    #[test]
    fn keys_can_start_with_digits() {
        // resolution_options has a literal `4k:` key.
        let v = parse_ok("4k: (3840, 2160)\n");
        assert_eq!(v.get("4k"), Some(&Value::Str("(3840, 2160)".into())));
    }

    #[test]
    fn plain_scalars_keep_colons_without_spaces() {
        // URLs and Windows paths survive as plain scalars.
        let v = parse_ok("url: https://example.com/x\n");
        assert_eq!(
            v.get("url"),
            Some(&Value::Str("https://example.com/x".into()))
        );
    }

    #[test]
    fn out_of_subset_features_are_named_diagnostics() {
        for (src, needle) in [
            ("a: [1, 2]\n", "flow collections"),
            ("a: {b: 1}\n", "flow collections"),
            ("a:\n  - one\n", "block sequences"),
            ("a: >-\n  folded\n", "folded block scalars"),
            ("a: &anchor 1\n", "anchors and aliases"),
            ("a: *alias\n", "anchors and aliases"),
            ("a: !!str 1\n", "tags"),
            ("--- \na: 1\n", "document markers"),
            ("%YAML 1.2\na: 1\n", "directives"),
            ("? complex\n: 1\n", "complex mapping keys"),
            ("a: |2\n  x\n", "indentation indicators"),
            ("a: \"unterminated\n", "unterminated double quote"),
        ] {
            let e = parse(src).expect_err(src);
            assert!(
                e.message.contains(needle),
                "for {src:?}: expected {needle:?} in {:?}",
                e.message
            );
        }
    }

    #[test]
    fn tabs_and_control_chars_are_refused_precisely() {
        let e = parse("a:\n\tb: 1\n").expect_err("tab");
        assert_eq!((e.line, e.col), (2, 1));
        assert!(e.message.contains("tab character in indentation"));

        let e = parse("a: b\u{0001}c\n").expect_err("control");
        assert!(e.message.contains("control character 0x01"));
    }

    #[test]
    fn indentation_errors_carry_expected_vs_found() {
        // A deeper line where a sibling was expected.
        let e = parse("a: 1\n   b: 2\n").expect_err("deeper");
        assert!(e.message.contains("entries start at column 1"));
        assert!(e.message.contains("found column 4"));

        // An unindent between two open levels: the outer mapping refuses it.
        let e = parse("a:\n    b: 1\n  c: 2\n").expect_err("mismatch");
        assert!(
            e.message.contains("entries start at column 1"),
            "{:?}",
            e.message
        );
        assert!(e.message.contains("found column 3"), "{:?}", e.message);

        // An unindent below the (indented) root level matches nothing.
        let e = parse("  a: 1\nb: 2\n").expect_err("below root");
        assert!(
            e.message.contains("does not match any open mapping level"),
            "{:?}",
            e.message
        );
    }

    #[test]
    fn missing_colon_is_a_precise_error() {
        let e = parse("just some words\n").expect_err("no colon");
        assert!(e.message.contains("found no ':'"), "{:?}", e.message);
    }

    #[test]
    fn depth_and_size_budgets_hold() {
        // Depth: 40 nested mappings against a 32 budget.
        let mut src = String::new();
        for i in 0..40 {
            src.push_str(&" ".repeat(i));
            src.push_str("k:\n");
        }
        src.push_str(&" ".repeat(40));
        src.push_str("leaf: 1\n");
        let e = parse(&src).expect_err("depth");
        assert!(
            e.message
                .contains("nesting deeper than the 32-level budget")
        );

        // Size.
        let big = "a: 1\n".repeat(300_000);
        let e = parse(&big).expect_err("size");
        assert!(e.message.contains("byte budget"));
    }

    #[test]
    fn merge_is_reference_exact() {
        let (base, _) = parse("a:\n  x: 1\n  y: 2\nb: keep\n").unwrap();
        let (over, _) = parse("a:\n  y: 20\n  z: 30\nc: new\n").unwrap();
        let merged = merge(base, over);
        assert_eq!(merged.get_path("a.x"), Some(&Value::Int(1)));
        assert_eq!(merged.get_path("a.y"), Some(&Value::Int(20)));
        assert_eq!(merged.get_path("a.z"), Some(&Value::Int(30)));
        assert_eq!(merged.get("b"), Some(&Value::Str("keep".into())));
        assert_eq!(merged.get("c"), Some(&Value::Str("new".into())));
        // A scalar replaces a map wholesale (no recursion).
        let (base, _) = parse("a:\n  x: 1\n").unwrap();
        let (over, _) = parse("a: flat\n").unwrap();
        assert_eq!(merge(base, over).get("a"), Some(&Value::Str("flat".into())));
    }

    #[test]
    fn fuzz_probe_never_panics_on_structured_or_random_bytes() {
        assert!(fuzz_probe(b"a: 1\n"));
        assert!(!fuzz_probe(&[0xff, 0xfe, 0x00]));
        // Deterministic pseudo-random smoke via a small LCG (no external deps).
        let mut state: u64 = 0x9e37_79b9_7f4a_7c15;
        for len in 0..512usize {
            let mut buf = vec![0u8; len];
            for b in &mut buf {
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                *b = (state >> 33) as u8;
            }
            let _ = fuzz_probe(&buf); // must not panic
        }
        // Structured hostility: deep indentation ladders, quote storms.
        let ladder: String = (0..64).map(|i| format!("{}k:\n", " ".repeat(i))).collect();
        let _ = fuzz_probe(ladder.as_bytes());
        let _ = fuzz_probe("a: \"\\u00\n".as_bytes());
        let _ = fuzz_probe("a: \"\\q\"\n".as_bytes());
    }
}
