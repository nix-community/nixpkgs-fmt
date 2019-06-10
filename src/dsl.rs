//! This module contains a definition of pattern-based formatting DSL.
use std::{fmt, ops, sync::Arc};

use rnix::{SyntaxElement, SyntaxKind};

use crate::tree_utils::{next_non_whitespace_sibling, prev_non_whitespace_sibling};

#[derive(Clone)]
pub(crate) struct Pred(Arc<dyn (Fn(SyntaxElement<'_>) -> bool)>);

impl fmt::Debug for Pred {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("Pred { ... }")
    }
}

impl Pred {
    fn parent_child(parent: Pred, child: Pred) -> Pred {
        let p = move |element: SyntaxElement<'_>| {
            child.matches(element)
                && element.parent().map(|it| parent.matches(it.into())) == Some(true)
        };
        p.into()
    }
    pub(crate) fn matches(&self, element: SyntaxElement<'_>) -> bool {
        (self.0)(element)
    }
}

impl ops::BitAnd for Pred {
    type Output = Pred;
    fn bitand(self, other: Pred) -> Pred {
        Pred(Arc::new(move |it| self.matches(it) && other.matches(it)))
    }
}

impl<F> From<F> for Pred
where
    F: for<'a> Fn(SyntaxElement<'a>) -> bool + 'static,
{
    fn from(f: F) -> Pred {
        Pred(Arc::new(f))
    }
}

impl From<SyntaxKind> for Pred {
    fn from(kind: SyntaxKind) -> Pred {
        Pred(Arc::new(move |it| it.kind() == kind))
    }
}

impl From<&'_ [SyntaxKind]> for Pred {
    fn from(kinds: &[SyntaxKind]) -> Pred {
        let kinds = kinds.to_vec();
        Pred(Arc::new(move |it| kinds.contains(&it.kind())))
    }
}

macro_rules! from_array {
    ($($arity:literal),*) => {$(
        impl From<[SyntaxKind; $arity]> for Pred {
            fn from(kinds: [SyntaxKind; $arity]) -> Pred {
                Pred::from(&kinds[..])
            }
        }
    )*}
}

from_array!(0, 1, 2, 3, 4, 5, 6, 7, 8);

#[derive(Debug)]
pub(crate) struct SpacingRule {
    pub(crate) pattern: Pred,
    pub(crate) space: Space,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Space {
    pub(crate) value: SpaceValue,
    pub(crate) loc: SpaceLoc,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum SpaceValue {
    Single,
    Newline,
    None,
    SingleOrNewline,
    NoneOrNewline,
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
    pub(crate) fn rule(&mut self, rule: SpacingRule) -> &mut SpacingDsl {
        self.rules.push(rule);
        self
    }
    pub(crate) fn inside(&mut self, parent: impl Into<Pred>) -> SpacingRuleBuilder<'_> {
        SpacingRuleBuilder {
            dsl: self,
            parent: parent.into(),
            child: None,
            between: None,
            loc: None,
        }
    }
}

pub(crate) struct SpacingRuleBuilder<'a> {
    dsl: &'a mut SpacingDsl,
    parent: Pred,
    child: Option<Pred>,
    between: Option<(SyntaxKind, SyntaxKind)>,
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
    pub(crate) fn between(mut self, left: SyntaxKind, right: SyntaxKind) -> SpacingRuleBuilder<'a> {
        self.between = Some((left, right));
        self.loc = Some(SpaceLoc::After);
        self
    }
    pub(crate) fn when(mut self, cond: fn(SyntaxElement<'_>) -> bool) -> SpacingRuleBuilder<'a> {
        let pred = cond.into();
        let prev = self.child.take().unwrap();
        self.child = Some(prev & pred);
        self
    }
    pub(crate) fn single_space(self) -> &'a mut SpacingDsl {
        self.finish(SpaceValue::Single)
    }
    pub(crate) fn no_space(self) -> &'a mut SpacingDsl {
        self.finish(SpaceValue::None)
    }
    pub(crate) fn single_space_or_newline(self) -> &'a mut SpacingDsl {
        self.finish(SpaceValue::SingleOrNewline)
    }
    pub(crate) fn no_space_or_newline(self) -> &'a mut SpacingDsl {
        self.finish(SpaceValue::NoneOrNewline)
    }
    fn finish(self, value: SpaceValue) -> &'a mut SpacingDsl {
        assert!(self.between.is_some() ^ self.child.is_some());
        if let Some((left, right)) = self.between {
            let rule = SpacingRule {
                pattern: Pred::parent_child(
                    self.parent.clone(),
                    Pred::from(left)
                        & Pred::from(move |it: SyntaxElement<'_>| {
                            next_non_whitespace_sibling(it).map(|it| it.kind() == right)
                                == Some(true)
                        }),
                ),
                space: Space {
                    value,
                    loc: SpaceLoc::After,
                },
            };
            self.dsl.rule(rule);

            let rule = SpacingRule {
                pattern: Pred::parent_child(
                    self.parent,
                    Pred::from(right)
                        & Pred::from(move |it: SyntaxElement<'_>| {
                            prev_non_whitespace_sibling(it).map(|it| it.kind() == left)
                                == Some(true)
                        }),
                ),
                space: Space {
                    value,
                    loc: SpaceLoc::Before,
                },
            };
            self.dsl.rule(rule);
        } else {
            let rule = SpacingRule {
                pattern: Pred::parent_child(self.parent, self.child.unwrap()),
                space: Space {
                    value,
                    loc: self.loc.unwrap(),
                },
            };
            self.dsl.rule(rule);
        }
        self.dsl
    }
}

#[derive(Debug)]
pub(crate) struct IndentRule {
    pub(crate) pattern: Pred,
}

#[derive(Default)]
pub(crate) struct IndentDsl {
    pub(crate) rules: Vec<IndentRule>,
}

impl IndentDsl {
    pub(crate) fn inside(&mut self, parent: impl Into<Pred>) -> IndentRuleBuilder<'_> {
        IndentRuleBuilder {
            dsl: self,
            parent: parent.into(),
            when: None,
        }
    }
}

pub(crate) struct IndentRuleBuilder<'a> {
    dsl: &'a mut IndentDsl,
    parent: Pred,
    when: Option<Pred>,
}

impl<'a> IndentRuleBuilder<'a> {
    pub(crate) fn when(mut self, cond: fn(SyntaxElement<'_>) -> bool) -> IndentRuleBuilder<'a> {
        self.when = Some(cond.into());
        self
    }

    pub(crate) fn indent(self, child: impl Into<Pred>) -> &'a mut IndentDsl {
        let mut child: Pred = child.into();
        if let Some(cond) = self.when {
            child = child & cond;
        }
        let indent_rule = IndentRule {
            pattern: Pred::parent_child(self.parent, child),
        };
        self.dsl.rules.push(indent_rule);
        self.dsl
    }
}
