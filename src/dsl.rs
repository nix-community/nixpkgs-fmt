//! This module contains a definition of pattern-based formatting DSL.
use std::fmt;

use rnix::{SyntaxElement, SyntaxKind};

#[derive(Debug)]
pub(crate) struct Pattern {
    pub(crate) parent: Pred,
    pub(crate) child: Pred,
}

impl Pattern {
    pub(crate) fn matches(&self, element: SyntaxElement) -> bool {
        self.child.matches(element.kind())
            && element.parent().map(|it| self.parent.matches(it.kind())) == Some(true)
    }
}

pub(crate) struct Pred(Box<Fn(SyntaxKind) -> bool>);

impl fmt::Debug for Pred {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("Pred { ... }")
    }
}

impl Pred {
    fn matches(&self, kind: SyntaxKind) -> bool {
        (self.0)(kind)
    }
}

impl From<SyntaxKind> for Pred {
    fn from(kind: SyntaxKind) -> Pred {
        Pred(Box::new(move |it| it == kind))
    }
}

impl From<&'static [SyntaxKind]> for Pred {
    fn from(kinds: &'static [SyntaxKind]) -> Pred {
        Pred(Box::new(move |it| kinds.contains(&it)))
    }
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

pub(crate) struct SpacingRuleBuilder {
    parent: Pred,
    child: Option<Pred>,
    value: Option<SpaceValue>,
    loc: Option<SpaceLoc>,
}

pub(crate) fn inside(parent: impl Into<Pred>) -> SpacingRuleBuilder {
    SpacingRuleBuilder {
        parent: parent.into(),
        child: None,
        value: None,
        loc: None,
    }
}

impl SpacingRuleBuilder {
    pub(crate) fn around(mut self, kind: impl Into<Pred>) -> SpacingRuleBuilder {
        self.child = Some(kind.into());
        self.loc = Some(SpaceLoc::Around);
        self
    }
    pub(crate) fn before(mut self, kind: impl Into<Pred>) -> SpacingRuleBuilder {
        self.child = Some(kind.into());
        self.loc = Some(SpaceLoc::Before);
        self
    }
    pub(crate) fn after(mut self, kind: impl Into<Pred>) -> SpacingRuleBuilder {
        self.child = Some(kind.into());
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
        let space = builder.space();
        let child = builder.child.unwrap();
        SpacingRule {
            pattern: Pattern {
                parent: builder.parent,
                child,
            },
            space,
        }
    }
}

#[derive(Debug)]
pub(crate) struct IndentRule {
    pub(crate) pattern: Pattern,
}

pub(crate) fn indent(parent: impl Into<Pred>, child: impl Into<Pred>) -> IndentRule {
    IndentRule {
        pattern: Pattern {
            parent: parent.into(),
            child: child.into(),
        },
    }
}
