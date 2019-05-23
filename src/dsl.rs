//! This module contains a definition of pattern-based formatting DSL.
use rowan::SyntaxKind;

pub(crate) struct SpacingRule {
    pub(crate) parent: SyntaxKind,
    pub(crate) child: SyntaxKind,
    pub(crate) space: Option<Space>,
}

pub(crate) enum Space {
    Single,
}

macro_rules! T {
    (=) => {
        rnix::tokenizer::tokens::TOKEN_ASSIGN
    };
}

pub(crate) struct SpacingRuleBuilder {
    parent: SyntaxKind,
    child: Option<SyntaxKind>,
    space: Option<Space>,
}

pub(crate) fn inside(parent: SyntaxKind) -> SpacingRuleBuilder {
    SpacingRuleBuilder {
        parent,
        child: None,
        space: None,
    }
}

impl SpacingRuleBuilder {
    pub(crate) fn around(mut self, kind: SyntaxKind) -> SpacingRuleBuilder {
        self.child = Some(kind);
        self
    }
    pub(crate) fn single_space(mut self) -> SpacingRuleBuilder {
        self.space = Some(Space::Single);
        self
    }
}

impl From<SpacingRuleBuilder> for SpacingRule {
    fn from(builder: SpacingRuleBuilder) -> SpacingRule {
        let child = builder.child.unwrap();
        SpacingRule {
            parent: builder.parent,
            child,
            space: builder.space,
        }
    }
}
