use std::{
    cmp::{Ord, Ordering, PartialOrd},
    fmt,
};

use rnix::{
    NodeOrToken, SmolStr, SyntaxElement, SyntaxKind,
    SyntaxKind::{NODE_ROOT, TOKEN_ELSE, TOKEN_IN, TOKEN_THEN},
    SyntaxNode, SyntaxToken, TextUnit,
};

use crate::{
    dsl::{IndentRule, Modality, RuleName},
    engine::{BlockPosition, FmtModel, SpaceBlock, SpaceBlockOrToken},
    pattern::{Pattern, PatternSet},
    tree_utils::prev_non_whitespace_token_sibling,
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
        self.len() == other.len()
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
        self.len().cmp(&other.len())
    }
}

impl fmt::Display for IndentLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.as_short_str() {
            Some(s) => f.write_str(s),
            None => write!(f, "{:width$}", "", width = u32::from(self.len()) as usize),
        }
    }
}

impl From<IndentLevel> for SmolStr {
    fn from(indent: IndentLevel) -> SmolStr {
        match indent.as_short_str() {
            Some(s) => s.into(),
            None => indent.to_string().into(),
        }
    }
}

impl IndentLevel {
    /// Constructs `IndentLevel` from indent string (without \n)
    pub(super) fn from_str(s: &str) -> IndentLevel {
        let len = len_for_indent(s);
        IndentLevel { level: len / INDENT_SIZE, alignment: len % INDENT_SIZE }
    }

    pub(super) fn from_whitespace_block(s: &str) -> IndentLevel {
        match s.rfind('\n') {
            None => IndentLevel::default(),
            Some(idx) => IndentLevel::from_str(&s[idx + 1..]),
        }
    }

    pub(super) fn from_len(len: TextUnit) -> IndentLevel {
        let len: u32 = len.into();
        IndentLevel { level: len / INDENT_SIZE, alignment: len % INDENT_SIZE }
    }

    pub(super) fn indent(self) -> IndentLevel {
        IndentLevel { level: self.level + 1, alignment: self.alignment }
    }

    pub(super) fn len(self) -> TextUnit {
        (self.level * INDENT_SIZE + self.alignment).into()
    }

    fn as_short_str(self) -> Option<&'static str> {
        #[rustfmt::skip]
        const SPACES: &str =
"                                                                                                ";
        let len = u32::from(self.len()) as usize;
        if len <= SPACES.len() {
            Some(&SPACES[..len])
        } else {
            None
        }
    }
}

impl IndentRule {
    pub(super) fn matches(&self, element: &SyntaxElement) -> bool {
        let parent = match element.parent() {
            None => return false,
            Some(it) => it,
        };
        if !self.parent.matches(&parent.into()) {
            return false;
        }
        if let Some(child) = &self.child {
            child.matches(element) == (self.child_modality == Modality::Positive)
        } else {
            true
        }
    }

    pub(super) fn apply(
        &self,
        element: &SyntaxElement,
        model: &mut FmtModel,
        anchor_set: &PatternSet<&Pattern>,
    ) {
        debug_assert!(self.matches(element));
        let anchor_indent = match indent_anchor(element, model, anchor_set) {
            Some((anchor, indent)) => {
                if let Some(p) = &self.anchor_pattern {
                    if !p.matches(&anchor.into()) {
                        default_indent(element, model, anchor_set);
                        return;
                    }
                }
                indent
            }
            _ => IndentLevel::default(),
        };
        let block = model.block_for(element, BlockPosition::Before);
        block.set_indent(anchor_indent.indent(), self.name);
    }
}

impl SpaceBlock {
    fn set_indent(&mut self, indent: IndentLevel, rule: RuleName) {
        let newlines: String = self.text().chars().filter(|&it| it == '\n').collect();
        self.set_text(&format!("{}{}", newlines, indent), Some(rule));
    }

    fn indent(&self) -> IndentLevel {
        IndentLevel::from_whitespace_block(self.text())
    }
}

pub(super) fn default_indent(
    element: &SyntaxElement,
    model: &mut FmtModel,
    anchor_set: &PatternSet<&Pattern>,
) {
    let anchor_indent = match indent_anchor(element, model, anchor_set) {
        Some((_anchor, indent)) => indent,
        _ => IndentLevel::default(),
    };
    let block = model.block_for(element, BlockPosition::Before);
    block.set_indent(anchor_indent, RuleName::new("Preserve indentation"));
}

pub(super) fn string_interpol_indent(
    element: &SyntaxElement,
    model: &mut FmtModel,
    anchor_set: &PatternSet<&Pattern>,
) {
    let anchor_indent = match indent_anchor(&element, model, anchor_set) {
        Some((_anchor, indent)) => indent,
        _ => IndentLevel::default(),
    };
    let block = model.block_for(&element, BlockPosition::Before);
    if !block.has_newline() {
        // No need to indent an element if it doesn't start a line
        return;
    }

    if element.parent().map(|it| it.text_range().start()) == Some(element.text_range().start()) {
        match element {
            NodeOrToken::Token(_token) => {
                block.set_indent(anchor_indent, RuleName::new("String Interpolation Value"))
            }
            _ => return,
        }
    }

    if element.kind() == TOKEN_THEN || element.kind() == TOKEN_ELSE {
        block.set_indent(anchor_indent, RuleName::new("String Interpolation Value"))
    } else {
        block.set_indent(anchor_indent.indent(), RuleName::new("String Interpolation"))
    }
}

pub(super) fn single_line_comment_indent(
    token: &SyntaxToken,
    model: &mut FmtModel,
    anchor_set: &PatternSet<&Pattern>,
) {
    let syntax_element = &token.clone().into();
    let anchor_indent = match indent_anchor(&syntax_element, model, anchor_set) {
        Some((_anchor, indent)) => indent,
        _ => IndentLevel::default(),
    };
    let block = model.block_for(&syntax_element, BlockPosition::Before);
    let prev_is_token_in = prev_non_whitespace_token_sibling(syntax_element)
        .map(|e| e.kind() == TOKEN_IN)
        .unwrap_or(false);
    if prev_is_token_in {
        block.set_indent(anchor_indent, RuleName::new("Comment Single Line Value"));
        return;
    }
    return;
}
/// Computes an anchoring element, together with its indent.
///
/// By default, the anchor is an ancestor of `element` which itself is the first
/// element on the line.
///
/// Elements from `anchor_set` are considered anchors even if they don't begin
/// the line.
pub(super) fn indent_anchor(
    element: &SyntaxElement,
    model: &mut FmtModel,
    anchor_set: &PatternSet<&Pattern>,
) -> Option<(SyntaxNode, IndentLevel)> {
    let parent = element.parent()?;
    for node in parent.ancestors() {
        let block = model.block_for(&node.clone().into(), BlockPosition::Before);
        if block.has_newline() {
            return Some((node.clone(), block.indent()));
        }
        if anchor_set.matching(node.clone().into()).next().is_some() {
            let indent = model.indent_of(&node);
            return Some((node, indent));
        }
        // For the root node, the block will typically be empty, but it still
        // should be considered an indent anchor.
        if node.kind() == NODE_ROOT {
            return Some((node, IndentLevel::default()));
        }
    }
    None
}

pub(super) fn indent_custom_anchor(
    element: &SyntaxElement,
    model: &mut FmtModel,
    kind: SyntaxKind,
    anchor_set: &PatternSet<&Pattern>,
) -> Option<IndentLevel> {
    let indent = IndentLevel::default();
    // Calculates current indent on NODE_STRING_INTERPOLATION inside NODE_STRING
    let parent = element.parent()?;
    let init_indent = match indent_anchor(&parent.clone().into(), model, anchor_set) {
        None => indent,
        Some((_, top_indent)) => top_indent,
    };

    let original_anchor = parent.ancestors().filter(|e| e.kind() == kind);
    fn count_indent(mut acc: IndentLevel, _node: SyntaxNode) -> IndentLevel {
        acc += IndentLevel::default().indent();
        acc
    }
    let node_indent = original_anchor.fold(init_indent, count_indent);
    Some(node_indent)
}

impl FmtModel {
    /// Calculates current indent level for node.
    fn indent_of(&mut self, node: &SyntaxNode) -> IndentLevel {
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
