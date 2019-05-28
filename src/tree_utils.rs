use rnix::{SyntaxNode, SyntaxToken, SyntaxElement, WalkEvent};

pub(crate) fn walk<'a>(node: &'a SyntaxNode) -> impl Iterator<Item = SyntaxElement<'a>> {
    node.preorder_with_tokens().filter_map(|event| match event {
        WalkEvent::Enter(_) => None,
        WalkEvent::Leave(element) => Some(element),
    })
}
pub(crate) fn walk_tokens<'a>(node: &'a SyntaxNode) -> impl Iterator<Item = SyntaxToken<'a>> {
    walk(node).filter_map(|element| match element {
        SyntaxElement::Token(token) => Some(token),
        _ => None,
    })
}

pub(crate) fn has_newline(node: &SyntaxNode) -> bool {
    walk_tokens(node).any(|it| it.text().contains('\n'))
}
