//! This module applies the rules from `super::dsl` to a `SyntaxNode`, to
//! get a `FmtDiff`.
mod fmt_model;
mod indentation;
mod spacing;
mod fixes;

use rnix::{SyntaxNode, TextRange};
use smol_str::SmolStr;

use crate::{
    dsl::{IndentDsl, RuleName, SpacingDsl},
    engine::fmt_model::{BlockPosition, FmtModel, SpaceBlock, SpaceBlockOrToken},
    pattern::PatternSet,
    tree_utils::walk_non_whitespace_non_interpol,
    AtomEdit, FmtDiff,
};

pub(crate) enum ExtraInfo<'a> {
    // Explanation contains a vector of edits, each of which is optionally paired with the rule name
    // that caused the edit.
    Explanation(&'a mut Vec<(AtomEdit, Option<RuleName>)>),
    /// Edits contains spacing edits to be applied to the document and indent edits to be applied in
    /// a second, separate transaction.
    Edits { spacing_edits: &'a mut Vec<AtomEdit>, indent_edits: &'a mut Vec<AtomEdit> },
    None,
}

/// The main entry point for formatting
pub(crate) fn reformat(
    spacing_dsl: &SpacingDsl,
    indent_dsl: &IndentDsl,
    node: &SyntaxNode,
    // Passing this enum is just a cute type-safe way for the caller to
    // select what extra info they need.
    mut extra_info: ExtraInfo,
) -> SyntaxNode {
    // First, adjust spacing rules between the nodes.
    // This can force some newlines.
    let mut model = FmtModel::new(node.clone());

    // First, adjust spacing rules between the nodes.
    // This can force some newlines.
    let spacing_rule_set = PatternSet::new(spacing_dsl.rules.iter());
    for element in walk_non_whitespace_non_interpol(node) {
        for rule in spacing_rule_set.matching(element.clone()) {
            rule.apply(&element, &mut model)
        }
    }

    let spacing_diff = model.into_diff();
    if let ExtraInfo::Explanation(explanation) = &mut extra_info {
        if spacing_diff.has_changes() {
            explanation.extend(spacing_diff.edits.clone())
        }
    } else if let ExtraInfo::Edits { spacing_edits, .. } = &mut extra_info {
        spacing_edits
            .extend(spacing_diff.edits.iter().map(|(ae, _)| ae.clone()).collect::<Vec<_>>());
    }
    let node = spacing_diff.to_node();

    // Next, for each node which starts the newline, adjust the indent.
    let mut model = FmtModel::new(node.clone());

    let anchor_set = PatternSet::new(indent_dsl.anchors.iter());
    for element in walk_non_whitespace_non_interpol(&node) {
        let block = model.block_for(&element, BlockPosition::Before);
        if !block.has_newline() {
            // No need to indent an element if it doesn't start a line
            continue;
        }
        // In cases like
        //
        // ```nix
        //   param:
        //     body
        // ```
        //
        // we only indent top-level node (lambda), and not it's first child (parameter)
        // TODO: Remove it when refactoring indentation engine.
        if element.parent().map(|it| it.text_range().start()) == Some(element.text_range().start())
        {
            continue;
        }

        let mut matching = indent_dsl.rules.iter().filter(|it| it.matches(&element));
        if let Some(rule) = matching.next() {
            rule.apply(&element, &mut model, &anchor_set);
            assert!(matching.next().is_none(), "more that one indent rule matched");
        } else {
            indentation::default_indent(&element, &mut model, &anchor_set)
        }
    }

    // Finally, do custom touch-ups like re-indenting of string literals and
    // replacing URLs with string literals.
    for element in walk_non_whitespace_non_interpol(&node) {
        fixes::fix(element, &mut model, &anchor_set)
    }

    let indent_diff = model.into_diff();
    if let ExtraInfo::Explanation(explanation) = extra_info {
        // We don't add indentation explanations if we had whitespace changes,
        // as that'll require fixing up the original ranges. This could be done,
        // but it's not clear if it is really necessary.
        if indent_diff.has_changes() && explanation.is_empty() {
            explanation.extend(indent_diff.edits.clone())
        }
    } else if let ExtraInfo::Edits { indent_edits, .. } = extra_info {
        indent_edits.extend(indent_diff.edits.iter().map(|(ae, _)| ae.clone()).collect::<Vec<_>>());
    }
    indent_diff.to_node()
}

impl FmtDiff {
    fn replace(&mut self, range: TextRange, text: SmolStr, reason: Option<RuleName>) {
        self.edits.push((AtomEdit { delete: range, insert: text }, reason))
    }
}
