//! This module applies the rules from `super::dsl` to a `SyntaxNode`, to
//! get a `FmtDiff`.
mod fmt_model;
mod indentation;
mod spacing;

use rnix::{SmolStr, SyntaxNode, TextRange};

use crate::{
    dsl::{IndentDsl, SpacingDsl},
    engine::fmt_model::{BlockPosition, FmtModel, SpaceBlock, SpaceBlockOrToken},
    pattern::PatternSet,
    tree_utils::walk_non_whitespace,
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
            indentation::default_indent(element, &mut model, &anchor_set)
        }
    }

    model.into_diff()
}

impl FmtDiff {
    fn replace(&mut self, range: TextRange, text: SmolStr) {
        self.edits.push(AtomEdit {
            delete: range,
            insert: text,
        })
    }
}
