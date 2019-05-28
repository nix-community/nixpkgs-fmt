//! This module applies the rules from `super::dsl` to a `SyntaxNode`, to
//! get a `FmtDiff`.
use rnix::{
    tokenizer::tokens::TOKEN_WHITESPACE, SmolStr, SyntaxElement, SyntaxNode, SyntaxToken,
    TextRange, TextUnit, WalkEvent,
};

use crate::{
    dsl::{SpaceLoc, SpaceValue, SpacingRule},
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
        if !self.pattern.matches(element) {
            return;
        }

        match self.space {
            Some(space) => match space.loc {
                SpaceLoc::Before => ensure_space_before(diff, element, space.value),
                SpaceLoc::After => ensure_space_after(diff, element, space.value),
                SpaceLoc::Around => ensure_space_around(diff, element, space.value),
            },
            None => (),
        }
    }
}

fn walk<'a>(node: &'a SyntaxNode) -> impl Iterator<Item = SyntaxElement<'a>> {
    node.preorder_with_tokens().filter_map(|event| match event {
        WalkEvent::Enter(_) => None,
        WalkEvent::Leave(element) => Some(element),
    })
}

fn ensure_space_around(diff: &mut FmtDiff, element: SyntaxElement, value: SpaceValue) {
    ensure_space_before(diff, element, value);
    ensure_space_after(diff, element, value);
}

fn ensure_space_before(diff: &mut FmtDiff, element: SyntaxElement, value: SpaceValue) {
    match element.prev_sibling_or_token() {
        None => return,
        Some(SyntaxElement::Token(token)) if token.kind() == TOKEN_WHITESPACE => {
            if let Some(text) = value.replace_ws(token) {
                diff.replace(token.range(), text.into())
            }
        }
        Some(_) => {
            if let Some(text) = value.insert_ws() {
                diff.insert(element.range().start(), text.into())
            }
        }
    };
}

fn ensure_space_after(diff: &mut FmtDiff, element: SyntaxElement, value: SpaceValue) {
    match element.next_sibling_or_token() {
        None => return,
        Some(SyntaxElement::Token(token)) if token.kind() == TOKEN_WHITESPACE => {
            if let Some(text) = value.replace_ws(token) {
                diff.replace(token.range(), text.into())
            }
        }
        Some(_) => {
            if let Some(text) = value.insert_ws() {
                diff.insert(element.range().end(), text.into())
            }
        }
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

impl SpaceValue {
    fn replace_ws(&self, token: SyntaxToken) -> Option<&'static str> {
        match (self, token.text().as_str()) {
            (SpaceValue::Single, " ") => return None,
            (SpaceValue::Single, _) => return Some(" "),
            (SpaceValue::None, _) => return Some(""),
        }
    }

    fn insert_ws(&self) -> Option<&'static str> {
        match self {
            SpaceValue::Single => Some(" "),
            SpaceValue::None => None,
        }
    }
}
