//! This module applies the rules from `super::dsl` to a `SyntaxNode`, to
//! get a `FmtDiff`.
use std::collections::HashMap;

use rnix::{
    tokenizer::tokens::TOKEN_WHITESPACE, SmolStr, SyntaxElement, SyntaxNode, SyntaxToken,
    TextRange, TextUnit,
};

use crate::{
    dsl::{SpaceLoc, SpaceValue, SpacingRule},
    tree_utils::{has_newline, walk},
    AtomEdit, FmtDiff,
};

pub(crate) fn format(rules: &[SpacingRule], root: &SyntaxNode) -> FmtDiff {
    let mut model = FmtModel::new(root);
    for element in walk(root) {
        for rule in rules.iter() {
            rule.apply(element, &mut model)
        }
    }
    model.into_diff()
}

#[derive(Debug)]
pub(crate) struct FmtModel<'a> {
    original_node: &'a SyntaxNode,
    blocks: Vec<SpaceBlock<'a>>,
    by_start_offset: HashMap<TextUnit, usize>,
    by_end_offset: HashMap<TextUnit, usize>,
}

#[derive(Debug)]
pub(crate) struct SpaceBlock<'a> {
    original: OriginalSpace<'a>,
    new_text: Option<SmolStr>,
}

#[derive(Debug, Clone, Copy)]
enum BlockPosition {
    Before,
    After,
}

#[derive(Debug)]
enum OriginalSpace<'a> {
    Some(SyntaxToken<'a>),
    None { offset: TextUnit },
}

impl<'a> OriginalSpace<'a> {
    fn text_range(&self) -> TextRange {
        match *self {
            OriginalSpace::Some(token) => token.range(),
            OriginalSpace::None { offset } => TextRange::from_to(offset, offset),
        }
    }
}

impl<'a> SpaceBlock<'a> {
    fn new(original: OriginalSpace<'a>) -> SpaceBlock<'a> {
        SpaceBlock {
            original,
            new_text: None,
        }
    }
    fn set_text(&mut self, text: &str) {
        match self.original {
            OriginalSpace::Some(token) if token.text() == text => return,
            _ => self.new_text = Some(text.into())
        }
    }
}

impl<'a> FmtModel<'a> {
    fn new(original_node: &'a SyntaxNode) -> FmtModel<'a> {
        FmtModel {
            original_node,
            blocks: vec![],
            by_start_offset: HashMap::default(),
            by_end_offset: HashMap::default(),
        }
    }

    fn into_diff(self) -> FmtDiff {
        let mut diff = FmtDiff {
            original_node: self.original_node.to_owned(),
            edits: vec![],
        };
        for block in self.blocks {
            if let Some(new_next) = block.new_text {
                diff.replace(block.original.text_range(), new_next);
            }
        }
        diff
    }

    fn block_for(
        &mut self,
        element: SyntaxElement<'a>,
        position: BlockPosition,
    ) -> &mut SpaceBlock<'a> {
        match position {
            BlockPosition::Before => {
                let offset = element.range().start();
                if let Some(&idx) = self.by_end_offset.get(&offset) {
                    &mut self.blocks[idx]
                } else {
                    let original = match element.prev_sibling_or_token() {
                        Some(SyntaxElement::Token(token)) if token.kind() == TOKEN_WHITESPACE => {
                            OriginalSpace::Some(token)
                        }
                        _ => OriginalSpace::None { offset },
                    };
                    self.push_block(SpaceBlock::new(original))
                }
            }
            BlockPosition::After => {
                let offset = element.range().end();
                if let Some(&idx) = self.by_start_offset.get(&offset) {
                    &mut self.blocks[idx]
                } else {
                    let original = match element.next_sibling_or_token() {
                        Some(SyntaxElement::Token(token)) if token.kind() == TOKEN_WHITESPACE => {
                            OriginalSpace::Some(token)
                        }
                        _ => OriginalSpace::None { offset },
                    };
                    self.push_block(SpaceBlock::new(original))
                }
            }
        }
    }

    fn push_block(&mut self, block: SpaceBlock<'a>) -> &mut SpaceBlock<'a> {
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

impl SpacingRule {
    fn apply<'a>(&self, element: SyntaxElement<'a>, model: &mut FmtModel<'a>) {
        if !self.pattern.matches(element) {
            return;
        }
        let space = match self.space {
            Some(it) => it,
            None => return,
        };

        if space.loc.is_before() {
            let block = model.block_for(element, BlockPosition::Before);
            ensure_space(element, block, space.value);
        }
        if space.loc.is_after() {
            let block = model.block_for(element, BlockPosition::After);
            ensure_space(element, block, space.value);
        }
    }
}

impl SpaceLoc {
    fn is_before(self) -> bool {
        match self {
            SpaceLoc::Before | SpaceLoc::Around => true,
            SpaceLoc::After => false,
        }
    }
    fn is_after(self) -> bool {
        match self {
            SpaceLoc::After | SpaceLoc::Around => true,
            SpaceLoc::Before => false,
        }
    }
}

fn ensure_space(element: SyntaxElement, block: &mut SpaceBlock, value: SpaceValue) {
    match value {
        SpaceValue::SingleOrNewline => {
            let has_newline = element.parent().map_or(false, has_newline);
            block.set_text(if has_newline { "\n" } else { " " });
        }
        SpaceValue::Single => block.set_text(" "),
        SpaceValue::None => block.set_text(""),
    }
}

impl FmtDiff {
    fn replace(&mut self, range: TextRange, text: SmolStr) {
        self.edits.push(AtomEdit {
            delete: range,
            insert: text,
        })
    }
}
