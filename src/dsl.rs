//! This module contains a definition of pattern-based formatting DSL.
use rnix::{SyntaxElement, SyntaxKind};

use crate::{
    pattern::Pattern,
    tree_utils::{next_non_whitespace_sibling, prev_non_whitespace_sibling},
};

/// `SpacingRule` describes whitespace requirements between `SyntaxElement` Note
/// that it doesn't handle indentation (first whitespace on a line), there's
/// `IndentRule` for that!
#[derive(Debug)]
pub(crate) struct SpacingRule {
    /// An element to which this spacing rule applies
    pub(crate) pattern: Pattern,
    /// How much space to add/remove at the start or end of the element.
    pub(crate) space: Space,
}

/// Make `SpacingRule` usable with `PatternSet`
impl AsRef<Pattern> for SpacingRule {
    fn as_ref(&self) -> &Pattern {
        &self.pattern
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Space {
    /// How much space to add.
    pub(crate) value: SpaceValue,
    /// Should the space be added before, after or around the element?
    pub(crate) loc: SpaceLoc,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum SpaceValue {
    /// Single whitespace char, like ` `
    Single,
    /// Single whitespace char, like ` `, but preserve existing line break.
    SingleOptionalNewline,
    /// A single newline (`\n`) char
    Newline,
    /// No whitespace at all.
    None,
    /// If the parent element fits into a single line, a single space.
    /// Otherwise, at least one newline.
    /// Existing newlines are preserved.
    SingleOrNewline,
    /// If the parent element fits into a single line, no space.
    /// Otherwise, at least one newline.
    /// Existing newlines are preserved.
    NoneOrNewline,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum SpaceLoc {
    /// Before the element.
    Before,
    /// After the element.
    After,
    /// On the both sides of the element.
    Around,
}

/// A builder to conveniently specify a set of `SpacingRule`s
#[derive(Debug, Default)]
pub(crate) struct SpacingDsl {
    pub(crate) rules: Vec<SpacingRule>,
}

impl SpacingDsl {
    /// Add a single rule.
    ///
    /// This is a low-level method for special cases, common cases are handled
    /// by a more convenient `SpacingRuleBuilder`.
    pub(crate) fn rule(&mut self, rule: SpacingRule) -> &mut SpacingDsl {
        self.rules.push(rule);
        self
    }
    /// Specify a spacing rule for an element which is a child of `parent`.
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

/// A builder to conveniently specify a single rule.
pub(crate) struct SpacingRuleBuilder<'a> {
    dsl: &'a mut SpacingDsl,
    parent: Pattern,
    child: Option<Pattern>,
    between: Option<(SyntaxKind, SyntaxKind)>,
    loc: Option<SpaceLoc>,
}

impl<'a> SpacingRuleBuilder<'a> {
    /// The rule applies to both sides of the element `child`.
    pub(crate) fn around(mut self, child: impl Into<Pattern>) -> SpacingRuleBuilder<'a> {
        self.child = Some(child.into());
        self.loc = Some(SpaceLoc::Around);
        self
    }
    /// The rule applies to the leading whitespace before `child`.
    pub(crate) fn before(mut self, child: impl Into<Pattern>) -> SpacingRuleBuilder<'a> {
        self.child = Some(child.into());
        self.loc = Some(SpaceLoc::Before);
        self
    }
    /// The rule applies to the trailing whitespace after `child`.
    pub(crate) fn after(mut self, child: impl Into<Pattern>) -> SpacingRuleBuilder<'a> {
        self.child = Some(child.into());
        self.loc = Some(SpaceLoc::After);
        self
    }
    /// The rule applies to the whitespace between the two nodes.
    pub(crate) fn between(mut self, left: SyntaxKind, right: SyntaxKind) -> SpacingRuleBuilder<'a> {
        self.between = Some((left, right));
        self.loc = Some(SpaceLoc::After);
        self
    }
    /// The rule applies if the `cond` is true.
    pub(crate) fn when(mut self, cond: fn(SyntaxElement<'_>) -> bool) -> SpacingRuleBuilder<'a> {
        let pred = cond.into();
        let prev = self.child.take().unwrap();
        self.child = Some(prev & pred);
        self
    }
    /// Enforce single whitespace character.
    pub(crate) fn single_space(self) -> &'a mut SpacingDsl {
        self.finish(SpaceValue::Single)
    }
    pub(crate) fn single_space_or_optional_newline(self) -> &'a mut SpacingDsl {
        self.finish(SpaceValue::SingleOptionalNewline)
    }
    /// Enforce the absence of any space.
    pub(crate) fn no_space(self) -> &'a mut SpacingDsl {
        self.finish(SpaceValue::None)
    }
    /// Enforce a single whitespace or newline character.
    pub(crate) fn single_space_or_newline(self) -> &'a mut SpacingDsl {
        self.finish(SpaceValue::SingleOrNewline)
    }
    /// Enforce a absence of whitespace or a newline character.
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

/// `IndentRule` describes how an element should be indented.
///
/// `IndentRule`s are only effective for elements which begin the line.
///
/// Note that currently we support only two kinds of indentation:
/// * the same, as parent (default)
/// * indent relative to the parent.
///
/// For this reason, `IndentRule` specifies only the pattern.
#[derive(Debug)]
pub(crate) struct IndentRule {
    /// Pattern that should match the element which is indented
    pub(crate) pattern: Pattern,
    /// Pattern that should match the anchoring element, relative to which we
    /// calculate the indent
    ///
    /// in
    ///
    /// ```nix
    /// {
    ///   f = x:
    ///      x * 2
    ///   ;
    /// }
    /// ```
    ///
    /// when we indent lambda body, `x * 2` is the thing to which the `pattern`
    /// applies and `f = x ...` is the thing to which the `anchor_pattern`
    /// applies.
    pub(crate) anchor_pattern: Option<Pattern>,
}

/// Make `IndentRule` usable with `PatternSet`
impl AsRef<Pattern> for IndentRule {
    fn as_ref(&self) -> &Pattern {
        &self.pattern
    }
}

/// A builder to conveniently specify a set of `IndentRule`s.
#[derive(Default)]
pub(crate) struct IndentDsl {
    pub(crate) rules: Vec<IndentRule>,
}

impl IndentDsl {
    /// Specify a rule for an element which is a child of `parent`.
    pub(crate) fn inside(&mut self, parent: impl Into<Pattern>) -> IndentRuleBuilder<'_> {
        IndentRuleBuilder {
            dsl: self,
            parent: parent.into(),
            when: None,
            when_anchor: None,
        }
    }
}

/// A builder to conveniently specify a single `IndentRule`.
pub(crate) struct IndentRuleBuilder<'a> {
    dsl: &'a mut IndentDsl,
    parent: Pattern,
    when: Option<Pattern>,
    when_anchor: Option<Pattern>,
}

impl<'a> IndentRuleBuilder<'a> {
    /// Indent the specified `child` element.
    pub(crate) fn indent(self, child: impl Into<Pattern>) -> &'a mut IndentDsl {
        let mut child: Pattern = child.into();
        if let Some(cond) = self.when {
            child = child & cond;
        }
        let indent_rule = IndentRule {
            pattern: child.with_parent(self.parent),
            anchor_pattern: self.when_anchor,
        };
        self.dsl.rules.push(indent_rule);
        self.dsl
    }
    /// Only apply this rule when `cond` is true.
    pub(crate) fn when(mut self, cond: fn(SyntaxElement<'_>) -> bool) -> IndentRuleBuilder<'a> {
        self.when = Some(cond.into());
        self
    }
    #[allow(unused)]
    pub(crate) fn when_anchor(mut self, cond: fn(SyntaxElement<'_>) -> bool) -> IndentRuleBuilder<'a> {
        self.when_anchor = Some(cond.into());
        self
    }
}
