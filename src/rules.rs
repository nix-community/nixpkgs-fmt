//! This module contains specific `super::dsl` rules for formatting nix language.
use rnix::{
    parser::nodes::*,
    types::{Apply, Lambda, Operation, SetEntry, TypedNode, With},
    SyntaxElement, SyntaxKind, T,
};

use crate::{
    dsl::{self, IndentDsl, SpacingDsl},
    pattern::Pattern,
    tree_utils::{has_newline, prev_sibling},
};

#[rustfmt::skip]
pub(crate) fn spacing() -> SpacingDsl {
    // Note: comments with fat arrow are tests!
    let mut dsl = SpacingDsl::default();

    dsl
        // { a=92; } => { a = 92; }
        .inside(NODE_SET_ENTRY).before(T![=]).single_space()
        .inside(NODE_SET_ENTRY).after(T![=]).single_space_or_optional_newline()

        // { a = 92 ; } => { a = 92; }
        .inside(NODE_SET_ENTRY).before(T![;]).no_space_or_optional_newline()
        .inside(NODE_SET_ENTRY).before(T![;]).when(after_literal).no_space()
        .inside(NODE_SET_ENTRY).before(T![;]).when(after_multiline_binop).single_space_or_newline()

        // a==  b => a == b
        // a!=  b => a != b
        // a++  b => a ++ b
        // a+  b => a + b
        // a  -   b => a - b
        // a*  b => a * b
        // a/  b => a / b
        .inside(NODE_OPERATION).after(BIN_OPS).single_space()
        .inside(NODE_OPERATION).before(BIN_OPS).single_space_or_newline()

        // foo . bar . baz => foo.bar.baz
        .inside(NODE_INDEX_SET).around(T![.]).no_space()

        // {} :92 => {}: 92
        .inside(NODE_LAMBDA).before(T![:]).no_space()
        .inside(NODE_LAMBDA).after(T![:]).single_space_or_optional_newline()

        // [1 2 3] => [ 1 2 3 ]
        .inside(NODE_LIST).after(T!["["]).single_space_or_newline()
        .inside(NODE_LIST).before(T!["]"]).single_space_or_newline()
        // [ ] => []
        .inside(NODE_LIST).between(T!["["], T!["]"]).no_space()
        .inside(NODE_LIST).after(VALUES).single_space_or_newline()
        .inside(NODE_LIST).after(TOKEN_COMMENT).single_space_or_newline()

        // {foo = 92;} => { foo = 92; }
        .inside(NODE_SET).after(T!["{"]).single_space_or_newline()
        .inside(NODE_SET).before(T!["}"]).single_space_or_newline()
        // { } => {}
        .inside(NODE_SET).between(T!["{"], T!["}"]).no_space()
        .inside(NODE_SET).after(NODE_SET_ENTRY).single_space_or_newline()

        // {arg}: 92 => { arg }: 92
        .inside(NODE_PATTERN).after(T!["{"]).single_space()
        .inside(NODE_PATTERN).before(T!["}"]).single_space_or_newline()
        // { }: 92 => {}: 92
        .inside(NODE_PATTERN).between(T!["{"], T!["}"]).no_space()

        // { foo,bar }: 92 => { foo, bar }: 92
        .inside(NODE_PATTERN).after(T![,]).single_space()
        .inside(NODE_PATTERN).before(T![,]).no_space_or_newline()

        // { inherit( x )  y  z  ; } => { inherit (x) y z; }
        .inside(NODE_INHERIT).around(NODE_INHERIT_FROM).single_space()
        .inside(NODE_INHERIT).before(T![;]).no_space()
        .inside(NODE_INHERIT).before(NODE_IDENT).single_space()
        .inside(NODE_INHERIT_FROM).after(T!["("]).no_space()
        .inside(NODE_INHERIT_FROM).before(T![")"]).no_space()

        // let   foo = bar;in  92 => let foo = bar; in 92
        .inside(NODE_LET_IN).after(T![let]).single_space_or_newline()
        .inside(NODE_LET_IN).around(T![in]).single_space_or_newline()

        // {a?3}: a => { a ? 3 }: a
        .inside(NODE_PAT_ENTRY).around(T![?]).single_space()

        // special-cased rules for leading and trailing whitespace
        .rule(dsl::SpacingRule {
            pattern: NODE_ROOT.into(),
            space: dsl::Space { loc: dsl::SpaceLoc::Before, value: dsl::SpaceValue::None }
        })
        .rule(dsl::SpacingRule {
            pattern: NODE_ROOT.into(),
            space: dsl::Space { loc: dsl::SpaceLoc::After, value: dsl::SpaceValue::Newline }
        })
        ;

    dsl
}

fn after_literal(node: SyntaxElement<'_>) -> bool {
    let prev = prev_sibling(node);
    return if let Some(w) = prev.and_then(With::cast) {
        is_literal(w.body().kind())
    } else {
        prev.map(|it| is_literal(it.kind())) == Some(true)
    };

    fn is_literal(kind: SyntaxKind) -> bool {
        kind == NODE_SET || kind == NODE_LIST
    }
}

fn after_multiline_binop(node: SyntaxElement<'_>) -> bool {
    let prev = prev_sibling(node);
    return if let Some(op) = prev.and_then(Operation::cast) {
        has_newline(op.node())
    } else {
        false
    };
}

#[rustfmt::skip]
pub(crate) fn indentation() -> IndentDsl {
    let mut dsl = IndentDsl::default();
    dsl
        .anchor(NODE_PAT_ENTRY)
        .anchor(Pattern::from(rhs_of_binop))

        .inside(NODE_LIST).indent(VALUES)
        .inside(ENTRY_OWNERS).indent([NODE_SET_ENTRY, NODE_INHERIT])

        .inside(NODE_LAMBDA).when(lambda_body_not_on_top_level).indent(VALUES)
        .inside(NODE_APPLY).when(apply_arg).indent(VALUES)

        .inside(NODE_SET_ENTRY).indent(VALUES)
        .inside(NODE_SET_ENTRY).when_anchor(set_entry_with_single_line_value).indent(T![;])
        .inside(NODE_OPERATION).when_anchor(set_entry_with_single_line_value).indent(BIN_OPS)
        .inside(NODE_WITH)
            .when(with_body)
            .when_anchor(set_entry_with_single_line_value)
            .indent(VALUES)

        // FIXME: don't force indent if comment is on the first line
        .inside(NODE_LIST).indent(TOKEN_COMMENT)
        .inside(ENTRY_OWNERS).indent(TOKEN_COMMENT)
        ;
    dsl
}

fn lambda_body_not_on_top_level(body: SyntaxElement<'_>) -> bool {
    fn find(body: SyntaxElement<'_>) -> Option<bool> {
        let body = body.as_node()?;
        let lambda = body.parent().and_then(Lambda::cast)?;
        Some(lambda.body() == body && lambda.node().parent()?.kind() != NODE_ROOT)
    }

    find(body) == Some(true)
}

fn with_body(body: SyntaxElement<'_>) -> bool {
    fn find(body: SyntaxElement<'_>) -> Option<bool> {
        let body = body.as_node()?;
        let with = body.parent().and_then(With::cast)?;
        Some(with.body() == body)
    }

    find(body) == Some(true)
}

fn apply_arg(arg: SyntaxElement<'_>) -> bool {
    fn find(arg: SyntaxElement<'_>) -> Option<bool> {
        let arg = arg.as_node()?;
        let apply = arg.parent().and_then(Apply::cast)?;
        Some(apply.value() == arg)
    }

    find(arg) == Some(true)
}

fn set_entry_with_single_line_value(entry: SyntaxElement<'_>) -> bool {
    fn find(entry: SyntaxElement<'_>) -> Option<bool> {
        let entry = entry.as_node().and_then(SetEntry::cast)?;
        let mut value = entry.value();
        if Operation::cast(value).is_none() {
            return Some(true);
        }
        while let Some(op) = Operation::cast(value) {
            value = op.value1()
        }
        Some(!has_newline(value))
    }
    find(entry) == Some(true)
}

fn rhs_of_binop(rhs: SyntaxElement<'_>) -> bool {
    fn find(rhs: SyntaxElement<'_>) -> Option<bool> {
        let op = rhs.parent().and_then(Operation::cast)?;
        Some(op.value2() == rhs.as_node()?)
    }
    find(rhs) == Some(true)
}

static ENTRY_OWNERS: &[SyntaxKind] = &[NODE_SET, NODE_LET_IN];

static VALUES: &[SyntaxKind] = &[
    NODE_IDENT,
    NODE_INDEX_SET,
    NODE_LAMBDA,
    NODE_LET_IN,
    NODE_LIST,
    NODE_OPERATION,
    NODE_PAREN,
    NODE_SET,
    NODE_STRING,
    NODE_VALUE,
    NODE_APPLY,
];

static BIN_OPS: &[SyntaxKind] = &[
    T!["//"],
    T![++],
    T![+],
    T![-],
    T![*],
    T![/],
    T![==],
    T![=>],
    T![<],
    T![>],
    T![<=],
    T![!=],
    T![||],
    T![&&],
];

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
    };

    use crate::reformat_string;

    #[test]
    fn smoke() {
        TestCase {
            name: None,
            before: "{
foo = x:
92;
}"
            .into(),
            after: "{
  foo = x:
    92;
}
"
            .into(),
        }
        .run()
        .map_err(|e| panic!(e))
        .unwrap();
    }

    /// For convenience, tests in this module are specified inline in comments,
    /// right next to the corresponding rule definition. This test looks at the
    /// text of this file, extracts test cases from comments and checks them.
    #[test]
    fn test_inline_tests() {
        let this_file = include_str!("./rules.rs");
        let tests = TestCase::collect_from_comments(this_file);
        run(&tests);
    }

    #[test]
    fn test_bad_good_tests() {
        let test_data = {
            let dir = env!("CARGO_MANIFEST_DIR");
            PathBuf::from(dir).join("test_data")
        };
        let tests = TestCase::collect_from_dir(&test_data);
        run(&tests);
    }

    #[derive(Debug)]
    struct TestCase {
        name: Option<String>,
        before: String,
        after: String,
    }

    impl TestCase {
        fn try_from(line: &str) -> Option<TestCase> {
            let divisor = line.find("=>")?;
            let before = format!("{}\n", line[..divisor].trim());
            let after = format!("{}\n", line[divisor + 3..].trim());
            Some(TestCase {
                name: None,
                before,
                after,
            })
        }

        fn collect_from_comments(text: &str) -> Vec<TestCase> {
            let res = text
                .lines()
                .filter_map(|line| line.find("// ").map(|idx| &line[idx + 3..]))
                .filter_map(TestCase::try_from)
                .collect::<Vec<_>>();

            assert!(res.len() > 0);
            res
        }

        fn collect_from_dir(dir: &Path) -> Vec<TestCase> {
            let mut res = vec![];
            for entry in fs::read_dir(dir).unwrap() {
                let entry = entry.unwrap();
                let file_name = entry.file_name();
                let before_name = file_name.to_str().unwrap();
                if before_name.ends_with(".bad.nix") {
                    let after_name = before_name.replace(".bad.", ".good.");
                    let test_case = TestCase {
                        name: Some(after_name.to_string()),
                        before: fs::read_to_string(dir.join(before_name)).unwrap(),
                        after: fs::read_to_string(dir.join(&after_name)).unwrap_or_else(|_err| {
                            panic!("{} not found", after_name);
                        }),
                    };
                    res.push(test_case);
                }
            }
            assert!(res.len() > 0);
            res
        }

        fn run(&self) -> Result<(), String> {
            let name = self.name.as_ref().map(|it| it.as_str()).unwrap_or("");
            let expected = &self.after;
            let actual = &reformat_string(&self.before);
            if expected != actual {
                return Err(format!(
                    "\n\nAssertion failed: wrong formatting\
                     \nTest: {}\n\
                     \nBefore:\n{}\n\
                     \nAfter:\n{}\n\
                     \nExpected:\n{}\n",
                    name, self.before, actual, self.after,
                ));
            }
            let second_round = &reformat_string(actual);
            if actual != second_round {
                return Err(format!(
                    "\n\nAssertion failed: formatting is not idempotent\
                     \nTest: {}\n\
                     \nBefore:\n{}\n\
                     \nAfter:\n{}\n",
                    name, actual, second_round,
                ));
            }
            Ok(())
        }
    }

    fn run(tests: &[TestCase]) {
        let mut n_failed = 0;
        for test in tests {
            if let Err(msg) = test.run() {
                n_failed += 1;
                eprintln!("{}", msg)
            }
        }
        if n_failed > 0 {
            panic!(
                "{} failed test cases out of {} total",
                n_failed,
                tests.len()
            )
        }
    }
}
