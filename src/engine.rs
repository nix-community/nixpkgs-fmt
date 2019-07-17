//! This module applies the rules from `super::dsl` to a `SyntaxNode`, to
//! get a `FmtDiff`.
mod fmt_model;
mod indentation;
mod spacing;
mod fixes;

use rnix::{SmolStr, SyntaxNode, TextRange};

use crate::{
    dsl::{SpacingDsl, IndentDsl},
    engine::fmt_model::{BlockPosition, FmtModel, SpaceBlock, SpaceBlockOrToken},
    pattern::PatternSet,
    tree_utils::walk_non_whitespace,
    AtomEdit, FmtDiff,
};


/// The main entry point for formatting
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
    let anchor_set = PatternSet::new(indent_dsl.anchors.iter());
    for element in walk_non_whitespace(root) {
        let block = model.block_for(element, BlockPosition::Before);
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
        if element.parent().map(|it| it.range().start()) == Some(element.range().start()) {
            continue;
        }

        let mut matching = indent_dsl.rules.iter().filter(|it| it.matches(element));
        if let Some(rule) = matching.next() {
            rule.apply(element, &mut model, &anchor_set);
            assert!(matching.next().is_none(), "more that one indent rule matched");
        } else {
            indentation::default_indent(element, &mut model, &anchor_set)
        }
    }

    // Finally, do custom touch-ups like re-indenting of string literals and
    // replacing URLs with string literals.
    for element in walk_non_whitespace(root) {
        fixes::fix(element, &mut model, &anchor_set)
    }

    model.into_diff()
}

impl FmtDiff {
    fn replace(&mut self, range: TextRange, text: SmolStr) {
        self.edits.push(AtomEdit { delete: range, insert: text })
    }
}
