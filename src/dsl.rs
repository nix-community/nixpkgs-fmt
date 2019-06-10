//! This module contains a definition of pattern-based formatting DSL.
use rnix::{SyntaxElement, SyntaxKind};

use crate::{
    pattern::Pattern,
    tree_utils::{next_non_whitespace_sibling, prev_non_whitespace_sibling},
};

#[derive(Debug)]
pub(crate) struct SpacingRule {
    pub(crate) pattern: Pattern,
    pub(crate) space: Space,
}

impl AsRef<Pattern> for SpacingRule {
    fn as_ref(&self) -> &Pattern {
        &self.pattern
    }
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
    pub(crate) fn inside(&mut self, parent: impl Into<Pattern>) -> SpacingRuleBuilder<'_> {
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
    parent: Pattern,
    child: Option<Pattern>,
    between: Option<(SyntaxKind, SyntaxKind)>,
    loc: Option<SpaceLoc>,
}

impl<'a> SpacingRuleBuilder<'a> {
    pub(crate) fn around(mut self, kind: impl Into<Pattern>) -> SpacingRuleBuilder<'a> {
        self.child = Some(kind.into());
        self.loc = Some(SpaceLoc::Around);
        self
    }
    pub(crate) fn before(mut self, kind: impl Into<Pattern>) -> SpacingRuleBuilder<'a> {
        self.child = Some(kind.into());
        self.loc = Some(SpaceLoc::Before);
        self
    }
    pub(crate) fn after(mut self, kind: impl Into<Pattern>) -> SpacingRuleBuilder<'a> {
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
            let child = Pattern::from(left)
                & Pattern::from(move |it: SyntaxElement<'_>| {
                    next_non_whitespace_sibling(it).map(|it| it.kind() == right) == Some(true)
                });
            let rule = SpacingRule {
                pattern: child.with_parent(self.parent.clone()),
                space: Space {
                    value,
                    loc: SpaceLoc::After,
                },
            };
            self.dsl.rule(rule);

            let child = Pattern::from(right)
                & Pattern::from(move |it: SyntaxElement<'_>| {
                    prev_non_whitespace_sibling(it).map(|it| it.kind() == left) == Some(true)
                });
            let rule = SpacingRule {
                pattern: child.with_parent(self.parent),
                space: Space {
                    value,
                    loc: SpaceLoc::Before,
                },
            };
            self.dsl.rule(rule);
        } else {
            let rule = SpacingRule {
                pattern: self.child.unwrap().with_parent(self.parent),
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
    pub(crate) pattern: Pattern,
}

impl AsRef<Pattern> for IndentRule {
    fn as_ref(&self) -> &Pattern {
        &self.pattern
    }
}

#[derive(Default)]
pub(crate) struct IndentDsl {
    pub(crate) rules: Vec<IndentRule>,
}

impl IndentDsl {
    pub(crate) fn inside(&mut self, parent: impl Into<Pattern>) -> IndentRuleBuilder<'_> {
        IndentRuleBuilder {
            dsl: self,
            parent: parent.into(),
            when: None,
        }
    }
}

pub(crate) struct IndentRuleBuilder<'a> {
    dsl: &'a mut IndentDsl,
    parent: Pattern,
    when: Option<Pattern>,
}

impl<'a> IndentRuleBuilder<'a> {
    pub(crate) fn when(mut self, cond: fn(SyntaxElement<'_>) -> bool) -> IndentRuleBuilder<'a> {
        self.when = Some(cond.into());
        self
    }

    pub(crate) fn indent(self, child: impl Into<Pattern>) -> &'a mut IndentDsl {
        let mut child: Pattern = child.into();
        if let Some(cond) = self.when {
            child = child & cond;
        }
        let indent_rule = IndentRule {
            pattern: child.with_parent(self.parent),
        };
        self.dsl.rules.push(indent_rule);
        self.dsl
    }
}
