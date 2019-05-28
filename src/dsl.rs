//! This module contains a definition of pattern-based formatting DSL.
use rnix::{SyntaxKind, SyntaxElement};

#[derive(Debug)]
pub(crate) struct Pattern {
    pub(crate) parent: SyntaxKind,
    pub(crate) child: SyntaxKind,
}

impl Pattern {
    pub(crate) fn matches(&self, element: SyntaxElement) -> bool {
        element.kind() == self.child && element.parent().map(|it| it.kind()) == Some(self.parent)
    }
}

#[derive(Debug)]
pub(crate) struct SpacingRule {
    pub(crate) pattern: Pattern,
    pub(crate) space: Option<Space>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Space {
    pub(crate) value: SpaceValue,
    pub(crate) loc: SpaceLoc,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum SpaceValue {
    SingleOrNewline,
    Single,
    None,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum SpaceLoc {
    Before,
    After,
    Around,
}

#[rustfmt::skip]
macro_rules! T {
    (=)   => (rnix::tokenizer::tokens::TOKEN_ASSIGN);
    ('{') => (rnix::tokenizer::tokens::TOKEN_CURLY_B_OPEN);
    ('}') => (rnix::tokenizer::tokens::TOKEN_CURLY_B_CLOSE);
    ('[') => (rnix::tokenizer::tokens::TOKEN_SQUARE_B_OPEN);
    (']') => (rnix::tokenizer::tokens::TOKEN_SQUARE_B_CLOSE);
    (++) => (rnix::tokenizer::tokens::TOKEN_CONCAT);
    (==) => (rnix::tokenizer::tokens::TOKEN_EQUAL);
    (:) => (rnix::tokenizer::tokens::TOKEN_COLON);
    (;) => (rnix::tokenizer::tokens::TOKEN_SEMICOLON);
    (.) => (rnix::tokenizer::tokens::TOKEN_DOT);
}

pub(crate) struct SpacingRuleBuilder {
    parent: SyntaxKind,
    child: Option<SyntaxKind>,
    value: Option<SpaceValue>,
    loc: Option<SpaceLoc>,
}

pub(crate) fn inside(parent: SyntaxKind) -> SpacingRuleBuilder {
    SpacingRuleBuilder {
        parent,
        child: None,
        value: None,
        loc: None,
    }
}

impl SpacingRuleBuilder {
    pub(crate) fn around(mut self, kind: SyntaxKind) -> SpacingRuleBuilder {
        self.child = Some(kind);
        self.loc = Some(SpaceLoc::Around);
        self
    }
    pub(crate) fn before(mut self, kind: SyntaxKind) -> SpacingRuleBuilder {
        self.child = Some(kind);
        self.loc = Some(SpaceLoc::Before);
        self
    }
    pub(crate) fn after(mut self, kind: SyntaxKind) -> SpacingRuleBuilder {
        self.child = Some(kind);
        self.loc = Some(SpaceLoc::After);
        self
    }
    pub(crate) fn single_space(mut self) -> SpacingRuleBuilder {
        self.value = Some(SpaceValue::Single);
        self
    }
    pub(crate) fn single_space_or_newline(mut self) -> SpacingRuleBuilder {
        self.value = Some(SpaceValue::SingleOrNewline);
        self
    }
    pub(crate) fn no_space(mut self) -> SpacingRuleBuilder {
        self.value = Some(SpaceValue::None);
        self
    }
    fn space(&self) -> Option<Space> {
        Some(Space {
            value: self.value?,
            loc: self.loc?,
        })
    }
}

impl From<SpacingRuleBuilder> for SpacingRule {
    fn from(builder: SpacingRuleBuilder) -> SpacingRule {
        let child = builder.child.unwrap();
        SpacingRule {
            pattern: Pattern { parent: builder.parent, child },
            space: builder.space(),
        }
    }
}
