//! This module applies the rules from `super::dsl` to a `SyntaxNode`, to
//! get a `FmtDiff`.
use std::collections::HashMap;

use rnix::{
    nodes::NODE_ROOT, tokenizer::tokens::TOKEN_WHITESPACE, SmolStr, SyntaxElement, SyntaxNode,
    SyntaxToken, TextRange, TextUnit,
};

use crate::{
    dsl::{IndentDsl, IndentRule, SpaceLoc, SpaceValue, SpacingDsl, SpacingRule},
    pattern::PatternSet,
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

    // First, adjust spacing rules between the nodes.
    // This can force some newlines.
    let spacing_rule_set = PatternSet::new(spacing_dsl.rules.iter());
    for element in walk_non_whitespace(root) {
        for rule in spacing_rule_set.matching(element) {
            rule.apply(element, &mut model)
        }
    }

    // Next, for each node which starts the newline, adjust the indent.
    let indent_rule_set = PatternSet::new(indent_dsl.rules.iter());
    for element in walk_non_whitespace(root) {
        let block = model.block_for(element, BlockPosition::Before);
        if !block.has_newline() {
            // No need to indent an element if it doesn't start a line
            continue;
        }
        let mut matching = indent_rule_set.matching(element);
        if let Some(rule) = matching.next() {
            rule.apply(element, &mut model);
            assert!(
                matching.next().is_none(),
                "more that one indent rule matched"
            );
        } else {
            default_indent(element, &mut model)
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
    fn set_line_break_preserving_existing_newlines(&mut self) {
        if self.text().contains('\n') {
            return;
        }
        self.set_text("\n");
    }
    fn set_text(&mut self, text: &str) {
        match self.original {
            OriginalSpace::Some(token) if token.text() == text && self.new_text.is_none() => return,
            _ => self.new_text = Some(text.into()),
        }
    }
    fn set_indent(&mut self, level: usize) {
        let indent = " ".repeat(INDENT_SIZE * level);
        let newlines: String = self.text().chars().filter(|&it| it == '\n').collect();
        self.set_text(&format!("{}{}", newlines, indent));
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
            return match original_space {
                Some(SyntaxElement::Token(token)) if token.kind() == TOKEN_WHITESPACE => {
                    if let Some(&existing) = match position {
                        Before => self.by_end_offset.get(&token.range().end()),
                        After => self.by_start_offset.get(&token.range().start()),
                    } {
                        &mut self.blocks[existing]
                    } else {
                        self.push_block(SpaceBlock::new(OriginalSpace::Some(token)))
                    }
                }
                _ => {
                    let offset = match position {
                        Before => root_node.range().start(),
                        After => root_node.range().end(),
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
            Before => element.range().start(),
            After => element.range().end(),
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

        let original_space = match original_token {
            Some(SyntaxElement::Token(token)) if token.kind() == TOKEN_WHITESPACE => {
                OriginalSpace::Some(token)
            }
            Some(_) => OriginalSpace::None { offset },
            _ => match element.parent() {
                Option::Some(parent) => return self.block_for(parent.into(), position),
                None => OriginalSpace::None { offset },
            },
        };

        self.push_block(SpaceBlock::new(original_space))
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
        SpaceValue::Single => block.set_text(" "),
        SpaceValue::Newline => block.set_text("\n"),
        SpaceValue::None => block.set_text(""),
        SpaceValue::SingleOrNewline => {
            let parent_is_multiline = element.parent().map_or(false, has_newline);
            if parent_is_multiline {
                block.set_line_break_preserving_existing_newlines()
            } else {
                block.set_text(" ")
            }
        }
        SpaceValue::NoneOrNewline => {
            let parent_is_multiline = element.parent().map_or(false, has_newline);
            if parent_is_multiline {
                block.set_line_break_preserving_existing_newlines()
            } else {
                block.set_text("")
            }
        }
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
