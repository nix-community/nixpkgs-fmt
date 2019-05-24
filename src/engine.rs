//! This module applies the rules from `super::dsl` to a `SyntaxNode`, to
//! get a `FmtDiff`.
use rnix::{
    SmolStr, SyntaxElement, SyntaxNode, TextRange, TextUnit, WalkEvent,
    tokenizer::tokens::TOKEN_WHITESPACE
};

use crate::{
    dsl::{Space, SpacingRule},
    AtomEdit, FmtDiff,
};

pub(crate) fn format(rules: &[SpacingRule], root: &SyntaxNode) -> FmtDiff {
    let mut diff = FmtDiff {
        original_node: root.to_owned(),
        edits: Vec::new(),
    };
    for element in walk(root) {
        for rule in rules.iter() {
            rule.apply(element, &mut diff)
        }
    }
    diff
}

impl SpacingRule {
    fn apply(&self, element: SyntaxElement, diff: &mut FmtDiff) {
        if !self.matches(element) {
            return;
        }
        match self.space {
            Some(Space::Single) => ensure_single_space_around(element, diff),
            None => (),
        }
    }

    fn matches(&self, element: SyntaxElement) -> bool {
        element.kind() == self.child && element.parent().map(|it| it.kind()) == Some(self.parent)
    }
}

fn walk<'a>(node: &'a SyntaxNode) -> impl Iterator<Item = SyntaxElement<'a>> {
    node.preorder_with_tokens().filter_map(|event| match event {
        WalkEvent::Enter(_) => None,
        WalkEvent::Leave(element) => Some(element),
    })
}

fn ensure_single_space_around(element: SyntaxElement, diff: &mut FmtDiff) {
    ensure_single_space_before(element, diff);
    ensure_single_space_after(element, diff);
}

fn ensure_single_space_before(element: SyntaxElement, diff: &mut FmtDiff) {
    match element.prev_sibling_or_token() {
        None => return,
        Some(SyntaxElement::Token(token)) if token.kind() == TOKEN_WHITESPACE => {
            if token.text() != " " {
                diff.replace(token.range(), " ".into())
            }
        }
        Some(_) => diff.insert(element.range().start(), " ".into()),
    };
}

fn ensure_single_space_after(element: SyntaxElement, diff: &mut FmtDiff) {
    match element.next_sibling_or_token() {
        None => return,
        Some(SyntaxElement::Token(token)) if token.kind() == TOKEN_WHITESPACE => {
            if token.text() != " " {
                diff.replace(token.range(), " ".into())
            }
        }
        Some(_) => diff.insert(element.range().end(), " ".into()),
    }
}

impl FmtDiff {
    fn insert(&mut self, offset: TextUnit, text: SmolStr) {
        self.edits.push(AtomEdit {
            delete: TextRange::offset_len(offset, 0.into()),
            insert: text,
        })
    }
    fn replace(&mut self, range: TextRange, text: SmolStr) {
        self.edits.push(AtomEdit {
            delete: range,
            insert: text,
        })
    }
}
