use rnix::{SyntaxElement, SyntaxNode, tokenizer::tokens::TOKEN_WHITESPACE};

use crate::{
    engine::{FmtModel, BlockPosition, INDENT_SIZE},
    dsl::IndentRule,
    pattern::{Pattern, PatternSet},
    tree_utils::preceding_tokens
};

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
            _ => 0,
        };
        let block = model.block_for(element, BlockPosition::Before);
        block.set_indent(anchor_indent + 1);
    }
}

pub(super) fn default_indent<'a>(
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
