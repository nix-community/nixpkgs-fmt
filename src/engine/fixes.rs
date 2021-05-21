use std::{cmp::min, convert::TryFrom};

use rnix::{
    NodeOrToken, SyntaxElement,
    SyntaxKind::{NODE_STRING, TOKEN_COMMENT, TOKEN_STRING_CONTENT, TOKEN_WHITESPACE},
    SyntaxNode, SyntaxToken, TextRange, TextSize,
};

use super::indentation::single_line_comment_indent;
use crate::{
    engine::{
        indentation::{indent_anchor, IndentLevel},
        BlockPosition, FmtModel,
    },
    pattern::{Pattern, PatternSet},
    AtomEdit,
};

pub(super) fn fix(element: SyntaxElement, model: &mut FmtModel, anchor_set: &PatternSet<&Pattern>) {
    match element {
        NodeOrToken::Node(node) => {
            if let NODE_STRING = node.kind() {
                fix_string_indentation(&node, model, anchor_set)
            }
        }
        NodeOrToken::Token(token) => {
            if let TOKEN_COMMENT = token.kind() {
                fix_comment_indentation(&token, model, anchor_set)
            }
        }
    }
}

fn fix_string_indentation(
    node: &SyntaxNode,
    model: &mut FmtModel,
    anchor_set: &PatternSet<&Pattern>,
) {
    let quote_indent = {
        let element: SyntaxElement = node.clone().into();
        let block = model.block_for(&element, BlockPosition::Before);
        if block.text().contains('\n') {
            IndentLevel::from_whitespace_block(block.text())
        } else {
            match indent_anchor(&element, model, anchor_set) {
                None => return,
                Some((_element, indent)) => indent,
            }
        }
    };
    let content_indent = quote_indent.indent();

    let indent_ranges: Vec<TextRange> = node_indent_ranges(node).collect();

    let (first_indent, last_indent) = match (indent_ranges.first(), indent_ranges.last()) {
        (Some(first), Some(last)) => (first, last),
        _ => return,
    };

    let first_line_is_blank =
        first_indent.start() == node.text_range().start() + TextSize::of("''\n");

    let last_line_is_blank = last_indent.end() + TextSize::of("''") == node.text_range().end();

    if !first_line_is_blank {
        return;
    }

    let content_ranges =
        if last_line_is_blank { &indent_ranges[..indent_ranges.len() - 1] } else { &indent_ranges };

    let common_indent = match content_ranges.iter().map(|it| it.len()).min() {
        Some(it) => it,
        None => return,
    };

    if content_indent != IndentLevel::from_len(common_indent) {
        for &range in content_ranges.iter() {
            let delete = TextRange::at(range.start(), min(common_indent, range.len()));
            model.raw_edit(AtomEdit { delete, insert: content_indent.into() })
        }
    }

    if last_line_is_blank && last_indent.len() != quote_indent.len() {
        model.raw_edit(AtomEdit { delete: *last_indent, insert: quote_indent.into() })
    }
}

/// If we indent multiline block comment, we should indent it's content as well.
fn fix_comment_indentation(
    token: &SyntaxToken,
    model: &mut FmtModel,
    anchor_set: &PatternSet<&Pattern>,
) {
    let is_block_comment = token.text().starts_with("/*");
    let normal_indent = match indent_anchor(&token.clone().into(), model, anchor_set) {
        None => return,
        Some((_element, indent)) => indent,
    };
    let block = model.block_for(&token.clone().into(), BlockPosition::Before);
    if !is_block_comment {
        single_line_comment_indent(token, model, anchor_set);
        return;
    }

    let comment_indent = {
        if block.text().contains('\n') {
            IndentLevel::from_whitespace_block(block.text())
        } else {
            normal_indent
        }
    };

    let content_indent = comment_indent.indent();
    let mut curr_offset = token.text_range().start();
    let mut first = true;
    for line in token.text().lines() {
        let offset = curr_offset;
        curr_offset += TextSize::of(line) + TextSize::of('\n');
        if first {
            first = false;
            continue;
        }
        let last_line_only_end_block = line.ends_with("*/") || line.trim_start() == "*/";
        let start_with_asterisk = line.trim_start().starts_with("*");
        let current_indent = IndentLevel::get_whitespace_block(line);
        if let Some(ws_end) = line.find(|it| it != ' ') {
            let delete =
                TextRange::at(offset, TextSize::try_from(ws_end).expect("woah big number"));
            if last_line_only_end_block || start_with_asterisk {
                model.raw_edit(AtomEdit {
                    delete,
                    insert: comment_indent.add_alignment(current_indent).into(),
                })
            } else {
                model.raw_edit(AtomEdit {
                    delete,
                    insert: content_indent.adjust_alignment(current_indent).into(),
                })
            }
        }
    }
}

/// For indented string like
///
/// ```nix
/// ''
///   hello
///     world
/// ''
/// ```
///
/// returns the ranges, corresponding to indentation. That is `"  "` before
/// hello, `"    "` before world and `""` before the last `''`.
fn node_indent_ranges(indented_string: &SyntaxNode) -> impl Iterator<Item = TextRange> {
    indented_string
        .descendants_with_tokens()
        .filter_map(|it| it.into_token())
        .filter(|it| it.kind() == TOKEN_STRING_CONTENT || it.kind() == TOKEN_WHITESPACE)
        .flat_map(|string_bit| {
            let start_offset = string_bit.text_range().start();
            string_indent_ranges(string_bit.text())
                .into_iter()
                .map(move |range| range + start_offset)
        })
}

fn string_indent_ranges(mut s: &str) -> Vec<TextRange> {
    let mut offset = 0;
    std::iter::from_fn(move || loop {
        let indent_start = s.find('\n')? + 1;
        s = &s[indent_start..];
        offset += indent_start;

        let indent_len = s.find(|c| c != ' ').unwrap_or_else(|| s.len());
        s = &s[indent_len..];
        offset += indent_len;
        if s.starts_with('\n') {
            continue;
        }

        return Some(TextRange::new(
            TextSize::try_from(offset - indent_len).expect("woah big numbers"),
            TextSize::try_from(offset).expect("woah big numbers"),
        ));
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_indent_ranges() {
        let text = r#"{
  python =
    ''
    for i in range(10):
        print(i)
    '';
}"#;

        let ast = rnix::parse(text);
        let node = crate::tree_utils::walk(&ast.node())
            .filter_map(|it| it.into_node())
            .find(|node| node.kind() == NODE_STRING)
            .unwrap();
        let indent_ranges: Vec<TextRange> = node_indent_ranges(&node).collect();
        assert_eq!(
            indent_ranges,
            vec![
                TextRange::new(20.into(), 24.into()),
                TextRange::new(44.into(), 52.into()),
                TextRange::new(61.into(), 65.into()),
            ]
        );

        let text = r#"{
  python =
    ''python
    for i in range(${range}):
        print(i)
    '';
}"#;

        let ast = rnix::parse(text);
        let node = crate::tree_utils::walk(&ast.node())
            .filter_map(|it| it.into_node())
            .find(|node| node.kind() == NODE_STRING)
            .unwrap();
        let indent_ranges: Vec<TextRange> = node_indent_ranges(&node).collect();
        assert_eq!(
            indent_ranges,
            vec![
                TextRange::new(26.into(), 30.into()),
                TextRange::new(56.into(), 64.into()),
                TextRange::new(73.into(), 77.into()),
            ]
        );
    }
}
