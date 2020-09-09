use rnix::SyntaxElement;

use crate::{
    dsl::{RuleName, SpaceLoc, SpaceValue, SpacingRule},
    engine::{BlockPosition, FmtModel, SpaceBlock},
    tree_utils::has_newline,
};

impl SpacingRule {
    pub(super) fn apply(&self, element: &SyntaxElement, model: &mut FmtModel) {
        if !self.pattern.matches(element) {
            return;
        }
        if self.space.loc.is_before() {
            let block = model.block_for(element, BlockPosition::Before);
            ensure_space(element, block, self.space.value, self.name);
        }
        if self.space.loc.is_after() {
            let block = model.block_for(element, BlockPosition::After);
            ensure_space(element, block, self.space.value, self.name);
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

fn ensure_space(
    element: &SyntaxElement,
    block: &mut SpaceBlock,
    value: SpaceValue,
    rule_name: Option<RuleName>,
) {
    match value {
        SpaceValue::Single => block.set_text(" ", rule_name),
        SpaceValue::SingleOptionalNewline => {
            if !block.has_newline() {
                block.set_text(" ", rule_name)
            }
        }
        SpaceValue::Newline => {
            // This will ignore if the block already has a newline
            if !block.has_newline() {
                block.set_text("\n", rule_name)
            }
        }
        SpaceValue::None => block.set_text("", rule_name),
        SpaceValue::NoneOptionalNewline => {
            if !block.has_newline() {
                block.set_text("", rule_name)
            }
        }
        SpaceValue::SingleOrNewline => {
            let parent_is_multiline = element.parent().map_or(false, |it| has_newline(&it));
            if parent_is_multiline {
                block.set_line_break_preserving_existing_newlines(None)
            } else {
                block.set_text(" ", rule_name)
            }
        }
        SpaceValue::NoneOrNewline => {
            let parent_is_multiline = element.parent().map_or(false, |it| has_newline(&it));
            if parent_is_multiline {
                block.set_line_break_preserving_existing_newlines(None)
            } else {
                block.set_text("", rule_name)
            }
        }
    }
}
