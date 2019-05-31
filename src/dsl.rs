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

#[derive(Debug)]
pub(crate) struct SpacingRule {
    pub(crate) pattern: Pattern,
    pub(crate) space: Space,
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

#[derive(Debug, Default)]
pub(crate) struct SpacingDsl {
    pub(crate) rules: Vec<SpacingRule>,
}

impl SpacingDsl {
    pub(crate) fn inside(&mut self, parent: impl Into<Pred>) -> SpacingRuleBuilder<'_> {
        SpacingRuleBuilder {
            dsl: self,
            parent: parent.into(),
            child: None,
            loc: None,
        }
    }
}

pub(crate) struct SpacingRuleBuilder<'a> {
    dsl: &'a mut SpacingDsl,
    parent: Pred,
    child: Option<Pred>,
    loc: Option<SpaceLoc>,
}

impl<'a> SpacingRuleBuilder<'a> {
    pub(crate) fn around(mut self, kind: impl Into<Pred>) -> SpacingRuleBuilder<'a> {
        self.child = Some(kind.into());
        self.loc = Some(SpaceLoc::Around);
        self
    }
    pub(crate) fn before(mut self, kind: impl Into<Pred>) -> SpacingRuleBuilder<'a> {
        self.child = Some(kind.into());
        self.loc = Some(SpaceLoc::Before);
        self
    }
    pub(crate) fn after(mut self, kind: impl Into<Pred>) -> SpacingRuleBuilder<'a> {
        self.child = Some(kind.into());
        self.loc = Some(SpaceLoc::After);
        self
    }
    pub(crate) fn single_space(self) -> &'a mut SpacingDsl {
        self.finish(SpaceValue::Single)
    }
    pub(crate) fn single_space_or_newline(self) -> &'a mut SpacingDsl {
        self.finish(SpaceValue::SingleOrNewline)
    }
    pub(crate) fn no_space(self) -> &'a mut SpacingDsl {
        self.finish(SpaceValue::None)
    }
    fn finish(self, value: SpaceValue) -> &'a mut SpacingDsl {
        let space = Space {
            value,
            loc: self.loc.unwrap(),
        };
        let rule = SpacingRule {
            pattern: Pattern {
                parent: self.parent,
                child: self.child.unwrap(),
            },
            space,
        };
        self.dsl.rules.push(rule);
        self.dsl
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
