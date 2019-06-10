use std::{fmt, ops, sync::Arc};

use rnix::{SyntaxElement, SyntaxKind};

#[derive(Clone)]
pub(crate) struct Pred(Arc<dyn (Fn(SyntaxElement<'_>) -> bool)>);

impl fmt::Debug for Pred {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("Pred { ... }")
    }
}

impl Pred {
    pub(crate) fn parent_child(parent: Pred, child: Pred) -> Pred {
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
