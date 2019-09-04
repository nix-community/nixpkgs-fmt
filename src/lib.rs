#[macro_use]
mod dsl;
mod engine;
mod rules;
mod tree_utils;
mod pattern;

use std::borrow::Cow;

use rnix::{SmolStr, SyntaxNode, TextRange};

use crate::tree_utils::walk_tokens;

/// The result of formatting.
///
/// From this Diff, you can get either the resulting `String`, or the
/// reformatted syntax node.
#[derive(Debug)]
pub struct FmtDiff {
    original_node: SyntaxNode,
    edits: Vec<AtomEdit>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtomEdit {
    pub delete: TextRange,
    pub insert: SmolStr,
}

impl FmtDiff {
    /// Get the diff of deletes and inserts
    pub fn text_diff(&self) -> Vec<AtomEdit> {
        self.edits.to_vec()
    }

    /// Whether or not formatting did caused any changes
    pub fn has_changes(&self) -> bool {
        !self.edits.is_empty()
    }

    /// Apply the formatting suggestions and return the new string
    pub fn to_string(&self) -> String {
        // TODO: don't copy strings all over the place
        let old_text =
            walk_tokens(&self.original_node).map(|it| it.to_string()).collect::<String>();

        let mut total_len = old_text.len();
        let mut edits = self.edits.clone();
        edits.sort_by_key(|edit| edit.delete.start());

        for atom in edits.iter() {
            total_len += atom.insert.len();
            total_len -= u32::from(atom.delete.end() - atom.delete.start()) as usize;
        }

        let mut buf = String::with_capacity(total_len);
        let mut prev = 0;
        for atom in edits.iter() {
            let start = u32::from(atom.delete.start()) as usize;
            let end = u32::from(atom.delete.end()) as usize;
            if start > prev {
                buf.push_str(&old_text[prev..start]);
            }
            buf.push_str(&atom.insert);
            prev = end;
        }
        buf.push_str(&old_text[prev..]);
        assert_eq!(buf.len(), total_len);
        buf
    }

    /// Apply the formatting suggestions and return the new node
    pub fn to_node(&self) -> SyntaxNode {
        unimplemented!()
    }
}

pub fn reformat_node(node: &SyntaxNode) -> FmtDiff {
    let spacing = rules::spacing();
    let indentation = rules::indentation();
    engine::format(&spacing, &indentation, node)
}

pub fn reformat_string(text: &str) -> String {
    let (mut text, line_endings) = convert_to_unix_line_endings(text);

    // Forcibly convert tabs to spaces as a pre-pass
    if text.contains('\t') {
        text = Cow::Owned(text.replace('\t', "  "))
    }

    let ast = rnix::parse(&*text);
    let root_node = ast.node();
    let diff = reformat_node(&root_node);
    let res = diff.to_string();
    match line_endings {
        LineEndings::Unix => res,
        LineEndings::Dos => convert_to_dos_line_endings(res),
    }
}

enum LineEndings {
    Unix,
    Dos,
}

fn convert_to_unix_line_endings(text: &str) -> (Cow<str>, LineEndings) {
    if !text.contains("\r\n") {
        return (Cow::Borrowed(text), LineEndings::Unix);
    }
    (Cow::Owned(text.replace("\r\n", "\n")), LineEndings::Dos)
}

fn convert_to_dos_line_endings(text: String) -> String {
    text.replace('\n', "\r\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_dos_line_endings() {
        assert_eq!(&reformat_string("{foo = 92;\n}"), "{\n  foo = 92;\n}\n");
        assert_eq!(&reformat_string("{foo = 92;\r\n}"), "{\r\n  foo = 92;\r\n}\r\n")
    }

    #[test]
    fn converts_tabs_to_spaces() {
        assert_eq!(&reformat_string("{\n\tfoo = 92;\t}\n"), "{\n  foo = 92;\n}\n");
    }
}
