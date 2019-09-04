use std::collections::HashMap;

use rnix::{
    NodeOrToken, SmolStr, SyntaxElement,
    SyntaxKind::{NODE_ROOT, TOKEN_COMMENT, TOKEN_WHITESPACE},
    SyntaxNode, SyntaxToken, TextRange, TextUnit,
};

use crate::{dsl::RuleName, engine::FmtDiff, tree_utils::preceding_tokens, AtomEdit};

/// `FmtModel` is a data structure to which we apply formatting rules.
///
/// It wraps a syntax trees and adds `SpaceBlock`s. `SpaceBlock` represents a
/// run (potentially empty) of whitespace characters. We create whitespace
/// blocks for existing whitespace tokens. However, if two non-whitespace tokens
/// are joined together in the syntax tree, we still create an empty
/// `SpaceBlock` to represent the space between them. That way, rules don't have
/// to separately handle a case when whitespace node is completely missing from
/// the original tree.
///
/// The `FmtModel` is a mutable data structure, formatting rules work by
/// changing the actual `SpacingBlock`s. For this reason, the order of
/// application of the rules is significant.
///
/// We maintain the invariant that no two `SpaceBlock`s are directly adjoint to
/// each other.
#[derive(Debug)]
pub(super) struct FmtModel {
    original_node: SyntaxNode,
    /// We store `SpaceBlock`s in array. With this setup, we can refer to a
    /// specific block by index, dodging many lifetime issues.
    blocks: Vec<SpaceBlock>,
    /// Maps offset to an index of the block, for which the offset is the start
    /// offset.
    by_start_offset: HashMap<TextUnit, usize>,
    /// Maps offset to an index of the block, for which the offset is the end
    /// offset.
    by_end_offset: HashMap<TextUnit, usize>,
    /// Arbitrary non-whitespace edits created by the last formatter phase.
    fixes: Vec<AtomEdit>,
}

#[derive(Debug)]
pub(super) struct SpaceBlock {
    original: OriginalSpace,
    /// Block's textual content, which is seen and modified by formatting rules.
    change: Option<SpaceChange>,
    /// If this block requires a newline to preserve semantics.
    ///
    /// True for blocks after comments. The engine takes care to never remove
    /// newline, even if some interaction of rules asks us to do so.
    semantic_newline: bool,
}

#[derive(Debug)]
struct SpaceChange {
    new_text: SmolStr,
    originating_rule: Option<RuleName>,
}

#[derive(Debug, Clone, Copy)]
pub(super) enum BlockPosition {
    Before,
    After,
}

/// Original whitespace token, if any, that backs a `SpaceBlock.
#[derive(Debug)]
pub(super) enum OriginalSpace {
    Some(SyntaxToken),
    None { offset: TextUnit },
}

impl OriginalSpace {
    fn text_range(&self) -> TextRange {
        match self {
            OriginalSpace::Some(token) => token.text_range(),
            OriginalSpace::None { offset } => TextRange::from_to(*offset, *offset),
        }
    }
}

impl SpaceBlock {
    fn new(original: OriginalSpace) -> SpaceBlock {
        let semantic_newline = match &original {
            OriginalSpace::Some(token) => {
                token.text().contains('\n') && is_line_comment(token.prev_sibling_or_token())
            }
            OriginalSpace::None { .. } => false,
        };
        SpaceBlock { original, change: None, semantic_newline }
    }
    pub(super) fn set_line_break_preserving_existing_newlines(&mut self, rule: Option<RuleName>) {
        if self.has_newline() {
            return;
        }
        self.set_text("\n", rule);
    }
    pub(super) fn set_text(&mut self, text: &str, rule: Option<RuleName>) {
        if self.semantic_newline && !text.contains('\n') {
            return;
        }
        match &self.original {
            OriginalSpace::Some(token) if token.text() == text && self.change.is_none() => return,
            _ => self.change = Some(SpaceChange { new_text: text.into(), originating_rule: rule }),
        }
    }
    pub(super) fn text(&self) -> &str {
        if let Some(change) = &self.change {
            return change.new_text.as_str();
        }
        self.original_text()
    }
    pub(crate) fn original_text(&self) -> &str {
        match &self.original {
            OriginalSpace::Some(token) => token.text().as_str(),
            OriginalSpace::None { .. } => "",
        }
    }
    pub(super) fn has_newline(&self) -> bool {
        self.text().contains('\n')
    }
}

pub(super) enum SpaceBlockOrToken<'a> {
    SpaceBlock(&'a mut SpaceBlock),
    Token(SyntaxToken),
}

impl FmtModel {
    pub(super) fn new(original_node: SyntaxNode) -> FmtModel {
        FmtModel {
            original_node,
            blocks: vec![],
            by_start_offset: HashMap::default(),
            by_end_offset: HashMap::default(),
            fixes: vec![],
        }
    }

    pub(super) fn into_diff(self) -> FmtDiff {
        let mut diff = FmtDiff { original_node: self.original_node.to_owned(), edits: vec![] };
        for block in self.blocks {
            if let Some(change) = block.change {
                diff.replace(block.original.text_range(), change.new_text);
            }
        }
        diff.edits.extend(self.fixes.into_iter());
        diff
    }

    /// This method gets a `SpaceBlock` before or after element. It's pretty
    /// complicated, because it needs to handle these different cases:
    /// * We could have already created the block. In this case, we should
    ///   return the existing block instead of creating a new one.
    /// * There may, or may not be, backing original whitespace token for the
    ///   block.
    /// * The necessary whitespace token is not necessary a sibling of
    ///   `element`, it might be a sibling of `element`'s ancestor.
    /// * Finally, root node is special, as it doesn't have siblings and instead
    ///   leading and trailing whitespace appear as children.
    pub(super) fn block_for(
        &mut self,
        element: &SyntaxElement,
        position: BlockPosition,
    ) -> &mut SpaceBlock {
        use BlockPosition::{After, Before};

        assert!(element.kind() != TOKEN_WHITESPACE);

        // Special case, for root node, leading and trailing ws are children of
        // the root. For all other node types, they are siblings
        if element.kind() == NODE_ROOT {
            let root_node = element.as_node().unwrap();
            let original_space = match position {
                BlockPosition::Before => root_node.first_child_or_token(),
                BlockPosition::After => root_node.last_child_or_token(),
            };
            return match &original_space {
                Some(NodeOrToken::Token(token)) if token.kind() == TOKEN_WHITESPACE => {
                    if let Some(&existing) = match position {
                        Before => self.by_end_offset.get(&token.text_range().end()),
                        After => self.by_start_offset.get(&token.text_range().start()),
                    } {
                        &mut self.blocks[existing]
                    } else {
                        self.push_block(SpaceBlock::new(OriginalSpace::Some(token.clone())))
                    }
                }
                _ => {
                    let offset = match position {
                        Before => root_node.text_range().start(),
                        After => root_node.text_range().end(),
                    };

                    if let Some(&existing) = match position {
                        Before => self.by_end_offset.get(&offset),
                        After => self.by_start_offset.get(&offset),
                    } {
                        &mut self.blocks[existing]
                    } else {
                        self.push_block(SpaceBlock::new(OriginalSpace::None { offset }))
                    }
                }
            };
        }

        let offset = match position {
            Before => element.text_range().start(),
            After => element.text_range().end(),
        };

        if let Some(&existing) = match position {
            Before => self.by_end_offset.get(&offset),
            After => self.by_start_offset.get(&offset),
        } {
            return &mut self.blocks[existing];
        }

        let original_token = match position {
            Before => element.prev_sibling_or_token(),
            After => element.next_sibling_or_token(),
        };

        let original_space = match &original_token {
            Some(NodeOrToken::Token(token)) if token.kind() == TOKEN_WHITESPACE => {
                OriginalSpace::Some(token.clone())
            }
            Some(_) => OriginalSpace::None { offset },
            _ => match element.parent() {
                Option::Some(parent) => return self.block_for(&parent.into(), position),
                None => OriginalSpace::None { offset },
            },
        };

        self.push_block(SpaceBlock::new(original_space))
    }

    /// Traverses tokens and space blocks that precede the given `node`.
    ///
    /// This is implemented as internal iterator due to lifetime issues.
    pub(super) fn with_preceding_elements(
        &mut self,
        node: &SyntaxNode,
        f: &mut impl FnMut(SpaceBlockOrToken<'_>) -> bool,
    ) {
        let block = self.block_for(&node.clone().into(), BlockPosition::Before);
        if f(SpaceBlockOrToken::SpaceBlock(block)) {
            return;
        }

        for token in preceding_tokens(node).filter(|it| it.kind() != TOKEN_WHITESPACE) {
            if f(SpaceBlockOrToken::Token(token.clone())) {
                return;
            }

            let block = self.block_for(&token.clone().into(), BlockPosition::Before);
            if f(SpaceBlockOrToken::SpaceBlock(block)) {
                return;
            }
        }
    }

    pub(super) fn raw_edit(&mut self, edit: AtomEdit) {
        self.fixes.push(edit)
    }

    fn push_block(&mut self, block: SpaceBlock) -> &mut SpaceBlock {
        let idx = self.blocks.len();
        let range = block.original.text_range();

        let prev = self.by_start_offset.insert(range.start(), idx);
        assert!(prev.is_none());
        let prev = self.by_end_offset.insert(range.end(), idx);
        assert!(prev.is_none());

        self.blocks.push(block);
        self.blocks.last_mut().unwrap()
    }
}

fn is_line_comment(node: Option<SyntaxElement>) -> bool {
    match node {
        Some(NodeOrToken::Token(token)) => {
            token.kind() == TOKEN_COMMENT && token.text().starts_with('#')
        }
        _ => false,
    }
}
