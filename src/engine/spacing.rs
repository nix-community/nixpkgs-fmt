use rnix::SyntaxElement;

use crate::{
    engine::{FmtModel, SpaceBlock, BlockPosition},
    dsl::{SpacingRule, SpaceLoc, SpaceValue},
    tree_utils::has_newline,
};

impl SpacingRule {
    pub(super) fn apply<'a>(&self, element: SyntaxElement<'a>, model: &mut FmtModel<'a>) {
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
        SpaceValue::SingleOptionalNewline => {
            if !block.has_newline() {
                block.set_text(" ")
            }
        }
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
