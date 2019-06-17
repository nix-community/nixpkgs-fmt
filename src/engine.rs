//! This module applies the rules from `super::dsl` to a `SyntaxNode`, to
//! get a `FmtDiff`.
mod fmt_model;

use rnix::{tokenizer::tokens::TOKEN_WHITESPACE, SmolStr, SyntaxElement, SyntaxNode, TextRange};

use crate::{
    dsl::{IndentDsl, IndentRule, SpaceLoc, SpaceValue, SpacingDsl, SpacingRule},
    engine::fmt_model::{BlockPosition, FmtModel, SpaceBlock},
    pattern::{Pattern, PatternSet},
    tree_utils::{has_newline, preceding_tokens, walk_non_whitespace},
    AtomEdit, FmtDiff,
};

const INDENT_SIZE: usize = 2;

pub(crate) fn format(
    spacing_dsl: &SpacingDsl,
    indent_dsl: &IndentDsl,
    root: &SyntaxNode,
) -> FmtDiff {
    let mut model = FmtModel::new(root);

    // First, adjust spacing rules between the nodes.
    // This can force some newlines.
    let spacing_rule_set = PatternSet::new(spacing_dsl.rules.iter());
    for element in walk_non_whitespace(root) {
        for rule in spacing_rule_set.matching(element) {
            rule.apply(element, &mut model)
        }
    }

    // Next, for each node which starts the newline, adjust the indent.
    let indent_rule_set = PatternSet::new(indent_dsl.rules.iter());
    let anchor_set = PatternSet::new(indent_dsl.anchors.iter());
    for element in walk_non_whitespace(root) {
        let block = model.block_for(element, BlockPosition::Before);
        if !block.has_newline() {
            // No need to indent an element if it doesn't start a line
            continue;
        }
        let mut matching = indent_rule_set.matching(element);
        if let Some(rule) = matching.next() {
            rule.apply(element, &mut model, &anchor_set);
            assert!(
                matching.next().is_none(),
                "more that one indent rule matched"
            );
        } else {
            default_indent(element, &mut model, &anchor_set)
        }
    }

    model.into_diff()
}

impl SpacingRule {
    fn apply<'a>(&self, element: SyntaxElement<'a>, model: &mut FmtModel<'a>) {
        if !self.pattern.matches(element) {
            return;
        }
        if self.space.loc.is_before() {
            let block = model.block_for(element, BlockPosition::Before);
            ensure_space(element, block, self.space.value);
        }
        if self.space.loc.is_after() {
            let block = model.block_for(element, BlockPosition::After);
            ensure_space(element, block, self.space.value);
        }
    }
}

impl SpaceLoc {
    fn is_before(self) -> bool {
        match self {
            SpaceLoc::Before | SpaceLoc::Around => true,
            SpaceLoc::After => false,
        }
    }
    fn is_after(self) -> bool {
        match self {
            SpaceLoc::After | SpaceLoc::Around => true,
            SpaceLoc::Before => false,
        }
    }
}

fn ensure_space(element: SyntaxElement, block: &mut SpaceBlock, value: SpaceValue) {
    match value {
        SpaceValue::Single => block.set_text(" "),
        SpaceValue::SingleOptionalNewline => {
            if !block.has_newline() {
                block.set_text(" ")
            }
        }
        SpaceValue::Newline => block.set_text("\n"),
        SpaceValue::None => block.set_text(""),
        SpaceValue::SingleOrNewline => {
            let parent_is_multiline = element.parent().map_or(false, has_newline);
            if parent_is_multiline {
                block.set_line_break_preserving_existing_newlines()
            } else {
                block.set_text(" ")
            }
        }
        SpaceValue::NoneOrNewline => {
            let parent_is_multiline = element.parent().map_or(false, has_newline);
            if parent_is_multiline {
                block.set_line_break_preserving_existing_newlines()
            } else {
                block.set_text("")
            }
        }
    }
}

impl IndentRule {
    fn apply<'a>(
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
            _ => 0,
        };
        let block = model.block_for(element, BlockPosition::Before);
        block.set_indent(anchor_indent + 1);
    }
}

fn default_indent<'a>(
    element: SyntaxElement<'a>,
    model: &mut FmtModel<'a>,
    anchor_set: &PatternSet<&Pattern>,
) {
    let anchor_indent = match indent_anchor(element, model, anchor_set) {
        Some((_anchor, indent)) => indent,
        _ => 0,
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
fn indent_anchor<'a>(
    element: SyntaxElement<'a>,
    model: &mut FmtModel<'a>,
    anchor_set: &PatternSet<&Pattern>,
) -> Option<(&'a SyntaxNode, usize)> {
    let parent = element.parent()?;
    for node in parent.ancestors() {
        let block = model.block_for(node.into(), BlockPosition::Before);
        if block.has_newline() {
            return Some((node, block.indent_level()));
        }
        if anchor_set.matching(node.into()).next().is_some() {
            let indent = calc_indent(node, model);
            return Some((node, indent));
        }
    }
    None
}

/// Calculates current indent level for node.
fn calc_indent<'a>(node: &'a SyntaxNode, model: &mut FmtModel<'a>) -> usize {
    // The impl is tricky: we need to account for whitespace in `model`, which
    // might be different from original whitespace in the syntax tree
    let mut node_indent = 0;

    let block = model.block_for(node.into(), BlockPosition::Before);
    let (len, has_newline) = len_of_last_line(block.text());
    node_indent += len;

    if !has_newline {
        for token in preceding_tokens(node).filter(|it| it.kind() != TOKEN_WHITESPACE) {
            let (len, has_newline) = len_of_last_line(token.text());
            node_indent += len;
            if has_newline {
                break;
            }

            let block = model.block_for(token.into(), BlockPosition::Before);
            let (len, has_newline) = len_of_last_line(block.text());
            node_indent += len;
            if has_newline {
                break;
            }
        }
    }

    return node_indent / INDENT_SIZE;

    fn len_of_last_line(s: &str) -> (usize, bool) {
        if let Some(idx) = s.rfind('\n') {
            return (s[idx + 1..].chars().count(), true);
        }
        (s.chars().count(), false)
    }
}

impl FmtDiff {
    fn replace(&mut self, range: TextRange, text: SmolStr) {
        self.edits.push(AtomEdit {
            delete: range,
            insert: text,
        })
    }
}
