use rnix::{
    nodes::NODE_STRING,
    tokenizer::tokens::TOKEN_STRING_CONTENT,
    SyntaxElement, SyntaxNode, TextRange, TextUnit,
};

use crate::{
    pattern::{PatternSet, Pattern},
    engine::{indentation::{IndentLevel, indent_anchor}, FmtModel},
    AtomEdit,
};

pub(super) fn fix<'a>(element: SyntaxElement<'a>, model: &mut FmtModel<'a>, anchor_set: &PatternSet<&Pattern>) {
    let node = match element.as_node() {
        Some(it) => it,
        None => return,
    };
    match node.kind() {
        NODE_STRING => fix_string_indentation(node, model, anchor_set),
        _ => return,
    }
}

fn fix_string_indentation<'a>(node: &'a SyntaxNode, model: &mut FmtModel<'a>, anchor_set: &PatternSet<&Pattern>) {
    let indent = match indent_anchor(node.into(), model, anchor_set) {
        None => return,
        Some((_element, indent)) => indent,
    };
    let target_indent = indent.indent();
    let string_bits = node
        .children_with_tokens()
        .filter_map(|it| it.as_token())
        .filter(|it| it.kind() == TOKEN_STRING_CONTENT)
        .collect::<Vec<_>>();
    let lines = string_bits.iter().flat_map(|it| it.text().lines());

    let first_line_is_blank = lines.clone().next().map(is_not_blank) == Some(false);

    if !first_line_is_blank {
        return;
    }

    let common_indent = lines
        .filter(|&s| is_not_blank(s))
        .map(|s| {
            let idx = s.find(|c| c != ' ').unwrap();
            &s[..idx]
        })
        .map(IndentLevel::from_str)
        .min();

    let common_indent = match common_indent {
        None => return,
        Some(indent) => indent,
    };

    if target_indent == common_indent {
        return;
    }

    for string_bit in string_bits {
        let mut start_offset = string_bit.range().start();
        let mut last_blank = None;
        for line in string_bit.text().lines() {
            last_blank = None;
            if is_not_blank(line) {
                let len = TextUnit::of_str(common_indent.as_str());
                model.raw_edit(AtomEdit {
                    delete: TextRange::offset_len(start_offset, len),
                    insert: target_indent.as_str().into(),
                })
            } else {
                last_blank = Some(line);
            }
            start_offset += TextUnit::of_str(line) + TextUnit::of_str("\n");
        }

        if last_blank.is_some() {
            start_offset -= TextUnit::of_str("\n")
        }
        // Special case: if this line is last, indent anyway
        if start_offset == node.range().end() - TextUnit::of_str("''") {
            let to = start_offset;
            let from = to - last_blank.map(TextUnit::of_str).unwrap_or_default();
            model.raw_edit(AtomEdit {
                delete: TextRange::from_to(from, to),
                insert: target_indent.as_str().into(),
            })
        }
    }
}

fn is_not_blank(s: &str) -> bool {
    !s.trim().is_empty()
}
