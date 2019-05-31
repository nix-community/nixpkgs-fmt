//! This module applies the rules from `super::dsl` to a `SyntaxNode`, to
//! get a `FmtDiff`.
use std::collections::HashMap;

use rnix::{
    tokenizer::tokens::TOKEN_WHITESPACE, SmolStr, SyntaxElement, SyntaxNode, SyntaxToken,
    TextRange, TextUnit,
};

use crate::{
    dsl::{IndentDsl, IndentRule, SpaceLoc, SpaceValue, SpacingDsl, SpacingRule},
    tree_utils::{has_newline, walk_non_whitespace},
    AtomEdit, FmtDiff,
};

const INDENT_SIZE: usize = 2;

pub(crate) fn format(
    spacing_dsl: &SpacingDsl,
    indent_dsl: &IndentDsl,
    root: &SyntaxNode,
) -> FmtDiff {
    let mut model = FmtModel::new(root);

    for element in walk_non_whitespace(root) {
        for rule in spacing_dsl.rules.iter() {
            rule.apply(element, &mut model)
        }
    }

    for element in walk_non_whitespace(root) {
        let block = model.block_for(element, BlockPosition::Before);
        if !block.has_newline() {
            continue;
        }
        match indent_dsl
            .rules
            .iter()
            .find(|it| it.pattern.matches(element))
        {
            Some(rule) => rule.apply(element, &mut model),
            None => default_indent(element, &mut model),
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
            OriginalSpace::Some(token) if token.text() == text && self.new_text.is_none() => return,
            _ => self.new_text = Some(text.into()),
        }
    }
    fn set_indent(&mut self, level: usize) {
        let indent = " ".repeat(INDENT_SIZE * level);
        self.set_text(&format!("\n{}", indent));
    }
    fn text(&self) -> &str {
        if let Some(text) = self.new_text.as_ref() {
            return text.as_str();
        }
        match self.original {
            OriginalSpace::Some(token) => token.text().as_str(),
            OriginalSpace::None { .. } => "",
        }
    }
    fn indent_level(&self) -> usize {
        let text = self.text();
        match text.rfind('\n') {
            None => return 0,
            Some(idx) => text[idx + 1..].chars().count() / INDENT_SIZE,
        }
    }
    fn has_newline(&self) -> bool {
        self.text().contains('\n')
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
        assert!(element.kind() != TOKEN_WHITESPACE);
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
                        Some(_) => OriginalSpace::None { offset },
                        _ => match element.parent() {
                            Option::Some(parent) => return self.block_for(parent.into(), position),
                            None => OriginalSpace::None { offset },
                        },
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
                        Some(_) => OriginalSpace::None { offset },
                        _ => match element.parent() {
                            Option::Some(parent) => return self.block_for(parent.into(), position),
                            None => OriginalSpace::None { offset },
                        },
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
        if self.space.loc.is_before() {
            let block = model.block_for(element, BlockPosition::Before);
            ensure_space(element, block, self.space.value);
        }
        if self.space.loc.is_after() {
            let block = model.block_for(element, BlockPosition::After);
            ensure_space(element, block, self.space.value);
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

impl IndentRule {
    fn apply<'a>(&self, element: SyntaxElement<'a>, model: &mut FmtModel<'a>) {
        assert!(self.pattern.matches(element));
        let parent_indent = match element.parent() {
            None => return,
            Some(it) => current_indent(it, model),
        };
        let block = model.block_for(element, BlockPosition::Before);
        block.set_indent(parent_indent + 1);
    }
}

fn default_indent<'a>(element: SyntaxElement<'a>, model: &mut FmtModel<'a>) {
    let parent_indent = match element.parent() {
        None => return,
        Some(it) => current_indent(it, model),
    };
    let block = model.block_for(element, BlockPosition::Before);
    block.set_indent(parent_indent);
}

fn current_indent<'a>(node: &'a SyntaxNode, model: &mut FmtModel<'a>) -> usize {
    for node in node.ancestors() {
        let block = model.block_for(node.into(), BlockPosition::Before);
        if block.has_newline() {
            return block.indent_level();
        }
    }
    0
}

impl FmtDiff {
    fn replace(&mut self, range: TextRange, text: SmolStr) {
        self.edits.push(AtomEdit {
            delete: range,
            insert: text,
        })
    }
}
