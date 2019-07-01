use std::cmp::{Ord, Ordering, PartialOrd};

use rnix::{SyntaxElement, SyntaxNode, TextUnit};

use crate::{
    dsl::IndentRule,
    engine::{BlockPosition, FmtModel, SpaceBlock, SpaceBlockOrToken},
    pattern::{Pattern, PatternSet},
};

const INDENT_SIZE: u32 = 2;

/// Indentation level (number of leading spaces).
///
/// It consists of two bits:
///   * `level`: the usual nesting level
///   * `alignment`: additional space required to align the node.
///
/// For example, in something like
///
/// ```nix
/// | foo.bar {
/// |   x = z;
/// | }
/// ```
///
/// `x = z` has alignment of one space, and level of one "  ".
#[derive(Default, Debug, Clone, Copy)]
pub(super) struct IndentLevel {
    level: u32,
    alignment: u32,
}

impl std::ops::AddAssign for IndentLevel {
    fn add_assign(&mut self, rhs: IndentLevel) {
        self.level += rhs.level;
        self.alignment += rhs.alignment;
    }
}

impl PartialEq for IndentLevel {
    fn eq(&self, other: &IndentLevel) -> bool {
        self.len_as_spaces() == other.len_as_spaces()
    }
}

impl Eq for IndentLevel {}

impl PartialOrd for IndentLevel {
    fn partial_cmp(&self, other: &IndentLevel) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for IndentLevel {
    fn cmp(&self, other: &IndentLevel) -> Ordering {
        self.len_as_spaces().cmp(&other.len_as_spaces())
    }
}

impl IndentLevel {
    pub(super) fn from_str(s: &str) -> IndentLevel {
        let len = len_for_indent(s);
        IndentLevel { level: len / INDENT_SIZE, alignment: len % INDENT_SIZE }
    }

    pub(super) fn from_len(len: TextUnit) -> IndentLevel {
        let len: u32 = len.into();
        IndentLevel { level: len / INDENT_SIZE, alignment: len % INDENT_SIZE }
    }

    pub(super) fn indent(self) -> IndentLevel {
        IndentLevel { level: self.level + 1, alignment: self.alignment }
    }

    pub(super) fn as_str(&self) -> &str {
        #[rustfmt::skip]
        const SPACES: &str =
"                                                                                                ";
        let len = self.len_as_spaces();
        let len = len as usize;
        assert!(len <= SPACES.len(), "don't support indent this large");
        &SPACES[..len]
    }

    fn len_as_spaces(&self) -> u32 {
        self.level * INDENT_SIZE + self.alignment
    }
}

impl IndentRule {
    pub(super) fn apply<'a>(
        &self,
        element: SyntaxElement<'a>,
        model: &mut FmtModel<'a>,
        anchor_set: &PatternSet<&Pattern>,
    ) {
        assert!(self.pattern.matches(element));
        let anchor_indent = match indent_anchor(element, model, anchor_set) {
            Some((anchor, indent)) => {
                if let Some(p) = &self.anchor_pattern {
                    if !p.matches(anchor.into()) {
                        default_indent(element, model, anchor_set);
                        return;
                    }
                }
                indent
            }
            _ => IndentLevel::default(),
        };
        let block = model.block_for(element, BlockPosition::Before);
        block.set_indent(anchor_indent.indent());
    }
}

impl SpaceBlock<'_> {
    fn set_indent(&mut self, indent: IndentLevel) {
        let newlines: String = self.text().chars().filter(|&it| it == '\n').collect();
        self.set_text(&format!("{}{}", newlines, indent.as_str()));
    }

    fn indent(&self) -> IndentLevel {
        let text = self.text();
        match text.rfind('\n') {
            None => IndentLevel::default(),
            Some(idx) => IndentLevel::from_str(&text[idx + 1..]),
        }
    }
}

pub(super) fn default_indent<'a>(
    element: SyntaxElement<'a>,
    model: &mut FmtModel<'a>,
    anchor_set: &PatternSet<&Pattern>,
) {
    let anchor_indent = match indent_anchor(element, model, anchor_set) {
        Some((_anchor, indent)) => indent,
        _ => IndentLevel::default(),
    };
    let block = model.block_for(element, BlockPosition::Before);
    block.set_indent(anchor_indent);
}

/// Computes an anchoring element, together with its indent.
///
/// By default, the anchor is an ancestor of `element` which itself is the first
/// element on the line.
///
/// Elements from `anchor_set` are considered anchors even if they don't begin
/// the line.
pub(super) fn indent_anchor<'a>(
    element: SyntaxElement<'a>,
    model: &mut FmtModel<'a>,
    anchor_set: &PatternSet<&Pattern>,
) -> Option<(&'a SyntaxNode, IndentLevel)> {
    let parent = element.parent()?;
    for node in parent.ancestors() {
        let block = model.block_for(node.into(), BlockPosition::Before);
        if block.has_newline() {
            return Some((node, block.indent()));
        }
        if anchor_set.matching(node.into()).next().is_some() {
            let indent = model.indent_of(node);
            return Some((node, indent));
        }
    }
    None
}

impl<'a> FmtModel<'a> {
    /// Calculates current indent level for node.
    fn indent_of(&mut self, node: &'a SyntaxNode) -> IndentLevel {
        // The impl is tricky: we need to account for whitespace in `model`, which
        // might be different from original whitespace in the syntax tree
        let mut indent = IndentLevel::default();
        self.with_preceding_elements(node, &mut |element| match element {
            SpaceBlockOrToken::Token(it) => {
                let (len, has_newline) = len_of_last_line(it.text());
                indent.alignment += len;
                has_newline
            }
            SpaceBlockOrToken::SpaceBlock(it) => {
                let (len, has_newline) = len_of_last_line(it.text());
                if has_newline {
                    indent += it.indent();
                } else {
                    indent.alignment += len;
                }
                has_newline
            }
        });

        return indent;

        fn len_of_last_line(s: &str) -> (u32, bool) {
            if let Some(idx) = s.rfind('\n') {
                return (len_for_indent(&s[idx + 1..]), true);
            }
            (len_for_indent(s), false)
        }
    }
}

fn len_for_indent(s: &str) -> u32 {
    s.chars().count() as u32
}
