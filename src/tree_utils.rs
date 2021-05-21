use std::iter::successors;

use rnix::{
    NodeOrToken, SyntaxElement,
    SyntaxKind::{
        NODE_APPLY, NODE_ASSERT, NODE_IF_ELSE, NODE_LAMBDA, NODE_LET_IN, NODE_PAREN, NODE_ROOT,
        NODE_STRING_INTERPOL, NODE_WITH, TOKEN_WHITESPACE,
    },
    SyntaxNode, SyntaxToken, WalkEvent,
};

pub(crate) fn walk(node: &SyntaxNode) -> impl Iterator<Item = SyntaxElement> {
    node.preorder_with_tokens().filter_map(|event| match event {
        WalkEvent::Enter(element) => Some(element),
        WalkEvent::Leave(_) => None,
    })
}

pub(crate) fn walk_non_whitespace_non_interpol(
    node: &SyntaxNode,
) -> impl Iterator<Item = SyntaxElement> {
    let mut interpool_level = 0;
    node.preorder_with_tokens().filter_map(move |event| {
        match &event {
            WalkEvent::Enter(element) if element.kind() == NODE_STRING_INTERPOL => {
                interpool_level += 1
            }
            WalkEvent::Leave(element) if element.kind() == NODE_STRING_INTERPOL => {
                interpool_level -= 1
            }
            _ => (),
        }
        match event {
            WalkEvent::Enter(element) => {
                Some(element).filter(|it| interpool_level == 0 && it.kind() != TOKEN_WHITESPACE)
            }
            WalkEvent::Leave(_) => None,
        }
    })
}

pub(crate) fn walk_tokens(node: &SyntaxNode) -> impl Iterator<Item = SyntaxToken> {
    walk(node).filter_map(|element| match element {
        NodeOrToken::Token(token) => Some(token),
        _ => None,
    })
}

pub(crate) fn has_newline(node: &SyntaxNode) -> bool {
    walk_tokens(node).any(|it| it.text().contains('\n'))
}

pub(crate) fn prev_sibling(element: &SyntaxElement) -> Option<SyntaxNode> {
    successors(element.prev_sibling_or_token(), |it| it.prev_sibling_or_token()).find_map(
        |element| match element {
            NodeOrToken::Node(it) => Some(it),
            NodeOrToken::Token(_) => None,
        },
    )
}

pub(crate) fn next_sibling(element: &SyntaxElement) -> Option<SyntaxNode> {
    successors(element.next_sibling_or_token(), |it| it.next_sibling_or_token()).find_map(
        |element| match element {
            NodeOrToken::Node(it) => Some(it),
            NodeOrToken::Token(_) => None,
        },
    )
}

pub(crate) fn not_on_top_level(element: &SyntaxElement) -> bool {
    !on_top_level(element)
}

pub(crate) fn on_top_level(element: &SyntaxElement) -> bool {
    let parent = match element.parent() {
        None => return true,
        Some(it) => it,
    };
    match parent.kind() {
        NODE_ROOT => true,
        NODE_LAMBDA | NODE_APPLY | NODE_WITH | NODE_ASSERT | NODE_LET_IN | NODE_IF_ELSE
        | NODE_PAREN => on_top_level(&parent.into()),
        _ => false,
    }
}

pub(crate) fn prev_token_sibling(element: &SyntaxElement) -> Option<SyntaxToken> {
    successors(element.prev_sibling_or_token(), |it| it.prev_sibling_or_token()).find_map(
        |element| match element {
            NodeOrToken::Node(_) => None,
            NodeOrToken::Token(it) => Some(it),
        },
    )
}

pub(crate) fn prev_non_whitespace_token_sibling(element: &SyntaxElement) -> Option<SyntaxToken> {
    successors(element.prev_sibling_or_token(), |it| it.prev_sibling_or_token()).find_map(
        |element| match element {
            NodeOrToken::Node(_) => None,
            NodeOrToken::Token(it) => {
                if it.kind() == TOKEN_WHITESPACE {
                    None
                } else {
                    Some(it)
                }
            }
        },
    )
}

pub(crate) fn prev_non_whitespace_sibling(element: &SyntaxElement) -> Option<SyntaxElement> {
    successors(element.prev_sibling_or_token(), |it| it.prev_sibling_or_token())
        .find(|it| it.kind() != TOKEN_WHITESPACE)
}

pub(crate) fn next_non_whitespace_sibling(element: &SyntaxElement) -> Option<SyntaxElement> {
    successors(element.next_sibling_or_token(), |it| it.next_sibling_or_token())
        .find(|it| it.kind() != TOKEN_WHITESPACE)
}

pub(crate) fn preceding_tokens(node: &SyntaxNode) -> impl Iterator<Item = SyntaxToken> {
    successors(node.first_token().and_then(|it| it.prev_token()), |it| it.prev_token())
}
