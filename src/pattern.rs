//! This module defines `Pattern`: a predicate over syntax elements

use std::{fmt, ops, sync::Arc};

use rnix::{SyntaxElement, SyntaxKind};

#[derive(Clone)]
pub(crate) struct Pattern(Arc<dyn (Fn(SyntaxElement<'_>) -> bool)>);

impl fmt::Debug for Pattern {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("Pattern { ... }")
    }
}

impl Pattern {
    pub(crate) fn parent_child(parent: Pattern, child: Pattern) -> Pattern {
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

impl ops::BitAnd for Pattern {
    type Output = Pattern;
    fn bitand(self, other: Pattern) -> Pattern {
        Pattern(Arc::new(move |it| self.matches(it) && other.matches(it)))
    }
}

impl<F> From<F> for Pattern
where
    F: for<'a> Fn(SyntaxElement<'a>) -> bool + 'static,
{
    fn from(f: F) -> Pattern {
        Pattern(Arc::new(f))
    }
}

impl From<SyntaxKind> for Pattern {
    fn from(kind: SyntaxKind) -> Pattern {
        Pattern(Arc::new(move |it| it.kind() == kind))
    }
}

impl From<&'_ [SyntaxKind]> for Pattern {
    fn from(kinds: &[SyntaxKind]) -> Pattern {
        let kinds = kinds.to_vec();
        Pattern(Arc::new(move |it| kinds.contains(&it.kind())))
    }
}

macro_rules! from_array {
    ($($arity:literal),*) => {$(
        impl From<[SyntaxKind; $arity]> for Pattern {
            fn from(kinds: [SyntaxKind; $arity]) -> Pattern {
                Pattern::from(&kinds[..])
            }
        }
    )*}
}

from_array!(0, 1, 2, 3, 4, 5, 6, 7, 8);
