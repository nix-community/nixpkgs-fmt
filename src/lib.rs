#[macro_use]
mod dsl;
mod engine;
mod rules;
mod tree_utils;

use rnix::{SmolStr, SyntaxNode, TextRange, TreeArc};

use crate::tree_utils::walk_tokens;

/// The result of formatting.
///
/// From this Diff, you can get either the resulting `String`, or the
/// reformatted syntax node.
#[derive(Debug)]
pub struct FmtDiff {
    original_node: TreeArc<SyntaxNode>,
    edits: Vec<AtomEdit>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AtomEdit {
    delete: TextRange,
    insert: SmolStr,
}

impl FmtDiff {
    pub fn has_changes(&self) -> bool {
        !self.edits.is_empty()
    }

    pub fn to_string(&self) -> String {
        // TODO: don't copy strings all over the place
        let old_text = walk_tokens(&self.original_node)
            .map(|it| it.text().as_str())
            .collect::<String>();

        let mut total_len = old_text.len();
        for atom in self.edits.iter() {
            total_len += atom.insert.len();
            total_len -= u32::from(atom.delete.end() - atom.delete.start()) as usize;
        }

        let mut buf = String::with_capacity(total_len);
        let mut prev = 0;
        for atom in self.edits.iter() {
            let start = u32::from(atom.delete.start()) as usize;
            let end = u32::from(atom.delete.end()) as usize;
            if start > prev {
                buf.push_str(&old_text[prev..start]);
            }
            buf.push_str(&atom.insert);
            prev = end;
        }
        buf.push_str(&old_text[prev..old_text.len()]);
        assert_eq!(buf.len(), total_len);
        buf
    }

    pub fn to_node(&self) -> TreeArc<SyntaxNode> {
        unimplemented!()
    }
}

pub fn reformat_node(node: &SyntaxNode) -> FmtDiff {
    let rules = rules::spacing();
    engine::format(&rules, node)
}

pub fn reformat_string(text: &str) -> String {
    let ast = rnix::parse(text);
    let root_node = ast.node();
    let diff = reformat_node(root_node);
    diff.to_string()
}

