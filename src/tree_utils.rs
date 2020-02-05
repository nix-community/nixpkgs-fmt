use std::iter::successors;

use rnix::{
    NodeOrToken, SyntaxElement, SyntaxKind::TOKEN_WHITESPACE, SyntaxNode, SyntaxToken, WalkEvent,
};

pub(crate) fn walk(node: &SyntaxNode) -> impl Iterator<Item = SyntaxElement> {
    node.preorder_with_tokens().filter_map(|event| match event {
        WalkEvent::Enter(element) => Some(element),
        WalkEvent::Leave(_) => None,
    })
}
pub(crate) fn walk_non_whitespace(node: &SyntaxNode) -> impl Iterator<Item = SyntaxElement> {
    node.preorder_with_tokens().filter_map(|event| match event {
        WalkEvent::Enter(element) => Some(element).filter(|it| it.kind() != TOKEN_WHITESPACE),
        WalkEvent::Leave(_) => None,
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
pub(crate) fn next_token_sibling(element: &SyntaxElement) -> Option<SyntaxToken> {
    successors(element.next_sibling_or_token(), |it| it.next_sibling_or_token()).find_map(
        |element| match element {
            NodeOrToken::Node(_) => None,
            NodeOrToken::Token(it) => Some(it),
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
