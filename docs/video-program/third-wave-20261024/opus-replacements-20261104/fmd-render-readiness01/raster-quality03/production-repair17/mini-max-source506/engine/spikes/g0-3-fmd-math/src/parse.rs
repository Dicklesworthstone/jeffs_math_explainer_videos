//! The spike's TeX-math reader: control sequences, groups, scripts, and
//! the constructs under proof. Every node carries its source byte span —
//! the §11.3 provenance that later drives `isolate`/`t2c`/
//! `TransformMatchingTex` without the Reference's render-twice hack.
//!
//! An unsupported construct is a precise, named error (§11.5: never
//! silence, never garbage) — the spike enforces the error *shape* the
//! coverage ratchet depends on.

use core::fmt;
use core::ops::Range;

/// Why a source string failed to parse or lay out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MathError {
    /// A control sequence the engine does not (yet) support, with its
    /// span — the ratchet's unit of accounting.
    UnsupportedCommand {
        /// The command name, without the backslash.
        name: String,
        /// Byte range in the source.
        span: Range<usize>,
    },
    /// Structural errors: unbalanced groups, missing arguments, a
    /// `\right` without `\left`, an `&`/`\\` outside a matrix body.
    Malformed {
        /// Human-readable description.
        what: &'static str,
        /// Byte position where the problem was detected.
        at: usize,
    },
    /// A character no bundled face maps (the coverage error for glyphs).
    UnmappedChar {
        /// The character.
        ch: char,
        /// Byte range in the source.
        span: Range<usize>,
    },
}

impl fmt::Display for MathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedCommand { name, span } => write!(
                f,
                "\\{name} is not supported by the G0-3 spike (bytes {}..{})",
                span.start, span.end
            ),
            Self::Malformed { what, at } => write!(f, "malformed input at byte {at}: {what}"),
            Self::UnmappedChar { ch, span } => write!(
                f,
                "no bundled face maps {ch:?} (bytes {}..{})",
                span.start, span.end
            ),
        }
    }
}

impl std::error::Error for MathError {}

/// A parsed math node. Every variant carries its source span.
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    /// A single character atom (letter, digit, symbol).
    Char {
        /// The character.
        ch: char,
        /// Source span.
        span: Range<usize>,
    },
    /// A brace group / argument list.
    List {
        /// Children in order.
        items: Vec<Node>,
        /// Source span of the whole group.
        span: Range<usize>,
    },
    /// `\frac{num}{den}`.
    Frac {
        /// Numerator.
        num: Box<Node>,
        /// Denominator.
        den: Box<Node>,
        /// Source span.
        span: Range<usize>,
    },
    /// `\sqrt[index]{radicand}` (index optional).
    Radical {
        /// The optional index (degree).
        index: Option<Box<Node>>,
        /// The radicand.
        radicand: Box<Node>,
        /// Source span.
        span: Range<usize>,
    },
    /// A base with optional sub/superscript (either may be absent, not both).
    Script {
        /// The base.
        base: Box<Node>,
        /// Subscript.
        sub: Option<Box<Node>>,
        /// Superscript.
        sup: Option<Box<Node>>,
        /// Source span.
        span: Range<usize>,
    },
    /// `\left⟨delim⟩ body \right⟨delim⟩`.
    LeftRight {
        /// Opening delimiter character.
        open: char,
        /// Closing delimiter character.
        close: char,
        /// The enclosed body.
        body: Box<Node>,
        /// Source span.
        span: Range<usize>,
    },
    /// A big operator (`\sum`, `\int`, `\prod`) — limits attach via `Script`.
    BigOp {
        /// The operator's Unicode character.
        ch: char,
        /// Source span.
        span: Range<usize>,
    },
    /// `\begin{matrix} … \end{matrix}`: rows of cells.
    Matrix {
        /// Rows, each a list of cell nodes.
        rows: Vec<Vec<Node>>,
        /// Source span.
        span: Range<usize>,
    },
}

impl Node {
    /// The node's source span.
    #[must_use]
    pub fn span(&self) -> Range<usize> {
        match self {
            Self::Char { span, .. }
            | Self::List { span, .. }
            | Self::Frac { span, .. }
            | Self::Radical { span, .. }
            | Self::Script { span, .. }
            | Self::LeftRight { span, .. }
            | Self::BigOp { span, .. }
            | Self::Matrix { span, .. } => span.clone(),
        }
    }
}

struct Reader<'a> {
    src: &'a str,
    pos: usize,
}

impl<'a> Reader<'a> {
    fn peek(&self) -> Option<char> {
        self.src[self.pos..].chars().next()
    }

    fn bump(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.pos += ch.len_utf8();
        Some(ch)
    }

    fn skip_ws(&mut self) {
        while self.peek().is_some_and(char::is_whitespace) {
            self.bump();
        }
    }

    /// Read a control-sequence name after the backslash.
    fn control_name(&mut self) -> String {
        let mut name = String::new();
        while let Some(c) = self.peek() {
            if c.is_ascii_alphabetic() {
                name.push(c);
                self.bump();
            } else {
                if name.is_empty() {
                    // single-char control sequence (\\, \{, …)
                    name.push(c);
                    self.bump();
                }
                break;
            }
        }
        name
    }

    /// Parse one required `{…}` argument (or a single token as TeX allows).
    fn argument(&mut self) -> Result<Node, MathError> {
        self.skip_ws();
        match self.peek() {
            Some('{') => self.group(),
            Some(_) => self.single_token(),
            None => Err(MathError::Malformed {
                what: "missing argument",
                at: self.pos,
            }),
        }
    }

    fn single_token(&mut self) -> Result<Node, MathError> {
        let start = self.pos;
        let ch = self.bump().ok_or(MathError::Malformed {
            what: "missing token",
            at: start,
        })?;
        if ch == '\\' {
            let name = self.control_name();
            return self.command(&name, start);
        }
        Ok(Node::Char {
            ch,
            span: start..self.pos,
        })
    }

    /// Parse a `{ … }` group.
    fn group(&mut self) -> Result<Node, MathError> {
        let start = self.pos;
        self.bump(); // consume '{'
        let items = self.list_until(|r| r.peek() == Some('}'))?;
        if self.peek() != Some('}') {
            return Err(MathError::Malformed {
                what: "unbalanced group: missing }",
                at: self.pos,
            });
        }
        self.bump();
        Ok(Node::List {
            items,
            span: start..self.pos,
        })
    }

    /// Parse list items until `stop` (which is not consumed) or exhaustion.
    fn list_until(&mut self, stop: impl Fn(&Self) -> bool) -> Result<Vec<Node>, MathError> {
        let mut items: Vec<Node> = Vec::new();
        loop {
            self.skip_ws();
            if self.peek().is_none() || stop(self) {
                return Ok(items);
            }
            let start = self.pos;
            match self.peek() {
                Some('^') | Some('_') => {
                    let is_sup = self.bump() == Some('^');
                    let script = self.argument()?;
                    let base = items.pop().ok_or(MathError::Malformed {
                        what: "script with no base",
                        at: start,
                    })?;
                    items.push(attach_script(base, script, is_sup, self.pos)?);
                }
                Some('{') => items.push(self.group()?),
                Some('}') => {
                    return Err(MathError::Malformed {
                        what: "unbalanced group: stray }",
                        at: self.pos,
                    });
                }
                Some('&') | Some('\\') if self.peek() == Some('&') => {
                    return Err(MathError::Malformed {
                        what: "& outside a matrix body",
                        at: self.pos,
                    });
                }
                Some('\\') => {
                    self.bump();
                    let name = self.control_name();
                    if name == "\\" {
                        return Err(MathError::Malformed {
                            what: "\\\\ outside a matrix body",
                            at: start,
                        });
                    }
                    items.push(self.command(&name, start)?);
                }
                Some(_) => items.push(self.single_token()?),
                None => return Ok(items),
            }
        }
    }

    /// Dispatch a parsed control sequence.
    fn command(&mut self, name: &str, start: usize) -> Result<Node, MathError> {
        match name {
            "frac" => {
                let num = self.argument()?;
                let den = self.argument()?;
                Ok(Node::Frac {
                    num: Box::new(num),
                    den: Box::new(den),
                    span: start..self.pos,
                })
            }
            "sqrt" => {
                self.skip_ws();
                let index = if self.peek() == Some('[') {
                    self.bump();
                    let items = self.list_until(|r| r.peek() == Some(']'))?;
                    if self.peek() != Some(']') {
                        return Err(MathError::Malformed {
                            what: "unterminated \\sqrt index",
                            at: self.pos,
                        });
                    }
                    let end = self.pos;
                    self.bump();
                    Some(Box::new(Node::List {
                        items,
                        span: start..end,
                    }))
                } else {
                    None
                };
                let radicand = self.argument()?;
                Ok(Node::Radical {
                    index,
                    radicand: Box::new(radicand),
                    span: start..self.pos,
                })
            }
            "left" => {
                self.skip_ws();
                let open = self.delimiter_char()?;
                let items = self.list_until(|r| r.src[r.pos..].starts_with("\\right"))?;
                if !self.src[self.pos..].starts_with("\\right") {
                    return Err(MathError::Malformed {
                        what: "\\left without matching \\right",
                        at: self.pos,
                    });
                }
                self.pos += "\\right".len();
                self.skip_ws();
                let close = self.delimiter_char()?;
                let span = start..self.pos;
                Ok(Node::LeftRight {
                    open,
                    close,
                    body: Box::new(Node::List {
                        items,
                        span: span.clone(),
                    }),
                    span,
                })
            }
            "right" => Err(MathError::Malformed {
                what: "\\right without \\left",
                at: start,
            }),
            "sum" => Ok(Node::BigOp {
                ch: '∑',
                span: start..self.pos,
            }),
            "int" => Ok(Node::BigOp {
                ch: '∫',
                span: start..self.pos,
            }),
            "prod" => Ok(Node::BigOp {
                ch: '∏',
                span: start..self.pos,
            }),
            "begin" => self.environment(start),
            "end" => Err(MathError::Malformed {
                what: "\\end without \\begin",
                at: start,
            }),
            other => Err(MathError::UnsupportedCommand {
                name: other.to_string(),
                span: start..self.pos,
            }),
        }
    }

    fn delimiter_char(&mut self) -> Result<char, MathError> {
        let at = self.pos;
        match self.bump() {
            Some(c @ ('(' | ')' | '[' | ']' | '{' | '}' | '|')) => Ok(c),
            Some('\\') => {
                let name = self.control_name();
                match name.as_str() {
                    "{" => Ok('{'),
                    "}" => Ok('}'),
                    "langle" => Ok('⟨'),
                    "rangle" => Ok('⟩'),
                    other => Err(MathError::UnsupportedCommand {
                        name: other.to_string(),
                        span: at..self.pos,
                    }),
                }
            }
            _ => Err(MathError::Malformed {
                what: "expected a delimiter",
                at,
            }),
        }
    }

    /// `\begin{matrix} … \end{matrix}`.
    fn environment(&mut self, start: usize) -> Result<Node, MathError> {
        self.skip_ws();
        if self.peek() != Some('{') {
            return Err(MathError::Malformed {
                what: "\\begin without {environment}",
                at: self.pos,
            });
        }
        self.bump();
        let mut env = String::new();
        while let Some(c) = self.peek() {
            if c == '}' {
                break;
            }
            env.push(c);
            self.bump();
        }
        self.bump(); // '}'
        if env != "matrix" {
            return Err(MathError::UnsupportedCommand {
                name: format!("begin{{{env}}}"),
                span: start..self.pos,
            });
        }
        let end_marker = "\\end{matrix}";
        let mut rows: Vec<Vec<Node>> = Vec::new();
        let mut row: Vec<Node> = Vec::new();
        loop {
            self.skip_ws();
            if self.src[self.pos..].starts_with(end_marker) {
                self.pos += end_marker.len();
                break;
            }
            if self.peek() == Some('&') {
                self.bump();
                continue; // cell boundary: the list_until below stopped here
            }
            if self.src[self.pos..].starts_with("\\\\") {
                self.pos += 2;
                rows.push(core::mem::take(&mut row));
                continue;
            }
            if self.peek().is_none() {
                return Err(MathError::Malformed {
                    what: "unterminated matrix environment",
                    at: self.pos,
                });
            }
            let items = self.list_until(|r| {
                r.peek() == Some('&')
                    || r.src[r.pos..].starts_with("\\\\")
                    || r.src[r.pos..].starts_with(end_marker)
            })?;
            let span = start..self.pos;
            row.push(Node::List { items, span });
        }
        if !row.is_empty() {
            rows.push(row);
        }
        Ok(Node::Matrix {
            rows,
            span: start..self.pos,
        })
    }
}

/// Merge a new script into a base (combining `x^a_b` into one Script node,
/// rejecting double superscripts the way TeX does).
fn attach_script(base: Node, script: Node, is_sup: bool, at: usize) -> Result<Node, MathError> {
    let span = base.span().start..script.span().end;
    if let Node::Script {
        base: inner,
        mut sub,
        mut sup,
        ..
    } = base
    {
        let slot = if is_sup { &mut sup } else { &mut sub };
        if slot.is_some() {
            return Err(MathError::Malformed {
                what: "double script",
                at,
            });
        }
        *slot = Some(Box::new(script));
        return Ok(Node::Script {
            base: inner,
            sub,
            sup,
            span,
        });
    }
    let (sub, sup) = if is_sup {
        (None, Some(Box::new(script)))
    } else {
        (Some(Box::new(script)), None)
    };
    Ok(Node::Script {
        base: Box::new(base),
        sub,
        sup,
        span,
    })
}

/// Parse a complete source string into a root list node.
pub fn parse(src: &str) -> Result<Node, MathError> {
    let mut reader = Reader { src, pos: 0 };
    let items = reader.list_until(|_| false)?;
    if reader.pos < src.len() {
        return Err(MathError::Malformed {
            what: "trailing input",
            at: reader.pos,
        });
    }
    Ok(Node::List {
        items,
        span: 0..src.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_nested_frac() {
        let node = parse(r"\frac{1}{\frac{2}{3}}").expect("parses");
        let Node::List { items, .. } = node else {
            panic!("root is a list")
        };
        assert!(matches!(&items[0], Node::Frac { .. }));
    }

    #[test]
    fn combines_sub_and_sup_on_one_base() {
        let node = parse(r"x_i^2").expect("parses");
        let Node::List { items, .. } = node else {
            panic!("root is a list")
        };
        let Node::Script { sub, sup, .. } = &items[0] else {
            panic!("expected script node")
        };
        assert!(sub.is_some() && sup.is_some());
    }

    #[test]
    fn double_superscript_is_malformed() {
        assert!(matches!(
            parse(r"x^a^b"),
            Err(MathError::Malformed { what, .. }) if what == "double script"
        ));
    }

    #[test]
    fn unsupported_command_names_itself_with_span() {
        let err = parse(r"a + \substack{x}").unwrap_err();
        let MathError::UnsupportedCommand { name, span } = err else {
            panic!("expected the ratchet error, got {err:?}")
        };
        assert_eq!(name, "substack");
        assert_eq!(&r"a + \substack{x}"[span], r"\substack");
    }

    #[test]
    fn left_right_requires_both_ends() {
        assert!(matches!(
            parse(r"\left( x"),
            Err(MathError::Malformed { what, .. }) if what.contains("\\left")
        ));
        assert!(matches!(
            parse(r"x \right)"),
            Err(MathError::Malformed { what, .. }) if what.contains("\\right")
        ));
    }

    #[test]
    fn matrix_rows_and_cells() {
        let node = parse(r"\begin{matrix} a & b \\ c & d \end{matrix}").expect("parses");
        let Node::List { items, .. } = node else {
            panic!("root is a list")
        };
        let Node::Matrix { rows, .. } = &items[0] else {
            panic!("expected matrix")
        };
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].len(), 2);
        assert_eq!(rows[1].len(), 2);
    }

    #[test]
    fn spans_are_faithful() {
        let src = r"\frac{a}{b}";
        let node = parse(src).expect("parses");
        let Node::List { items, .. } = node else {
            panic!("root is a list")
        };
        assert_eq!(&src[items[0].span()], src);
    }
}
