//! This module contains specific `super::dsl` rules for formatting nix language.
use rnix::{
    types::{Lambda, LetIn, TypedNode, With},
    NodeOrToken, SyntaxElement, SyntaxKind,
    SyntaxKind::*,
    T,
};

use crate::{
    dsl::{self, IndentDsl, IndentValue::*, SpacingDsl},
    pattern::p,
    tree_utils::{
        has_newline, next_non_whitespace_sibling, next_sibling, not_on_top_level, on_top_level,
        prev_non_whitespace_sibling, prev_sibling, prev_token_sibling,
    },
};

#[rustfmt::skip]
pub(crate) fn spacing() -> SpacingDsl {
    let mut dsl = SpacingDsl::default();

    dsl
        .test("{ a=92; }", "{ a = 92; }")
        .rule("Space before =")
        .inside(NODE_KEY_VALUE).before(T![=]).single_space()

        .rule("Space after =")
        .inside(NODE_KEY_VALUE).after(T![=]).single_space_or_optional_newline()

        .test("{ a = 92 ; }", "{ a = 92; }")
        .inside(NODE_KEY_VALUE).before(T![;]).no_space_or_optional_newline()
        .inside(NODE_KEY_VALUE).before(T![;]).when(after_literal).no_space()
        .inside(NODE_KEY_VALUE).before(NODE_IF_ELSE).when(not_inline_if).single_space_or_newline()
        .inside(NODE_KEY_VALUE).before(NODE_LET_IN).when(inline_let_in).single_space_or_newline()

        .test("a++\nb", "a ++\nb")
        .test("a==  b", "a == b")
        .test("a!=  b", "a != b")
        .test("a++  b", "a ++ b")
        .test("a+  b", "a + b")
        .test("a  -   b", "a - b")
        .test("a*  b", "a * b")
        .test("a/  b", "a / b")
        .inside(NODE_BIN_OP).around(BIN_OPS).single_space_or_optional_newline()

        .test("foo . bar . baz", "foo.bar.baz")
        .inside(NODE_SELECT).around(T![.]).no_space()
        .test("{} :92", "{}: 92")
        .inside(NODE_LAMBDA).before(T![:]).no_space()
        .inside(NODE_LAMBDA).after(T![:]).single_space_or_optional_newline()
        .inside(NODE_LAMBDA).before(NODE_IF_ELSE).when(not_inline_if).single_space_or_newline()
        .inside(NODE_LAMBDA).before(NODE_LET_IN).single_space_or_newline()

        .test("[1 2 3]", "[ 1 2 3 ]")
        .inside(NODE_LIST).after(T!["["]).single_space_or_newline()
        .inside(NODE_LIST).before(T!["]"]).single_space_or_newline()
        .inside(NODE_LIST).after(T!["["]).when(inline_with_attr_set).no_space()
        .inside(NODE_LIST).before(T!["]"]).when(inline_with_attr_set).no_space()
        .test("[]", "[ ]")
        .inside(NODE_LIST).between(T!["["], T!["]"]).single_space_or_optional_newline()
        .inside(NODE_LIST).between(VALUES, VALUES).single_space_or_newline()
        .inside(NODE_LIST).between(VALUES, TOKEN_COMMENT).single_space_or_optional_newline()
        .inside(NODE_LIST).between(TOKEN_COMMENT, VALUES).single_space_or_newline()

        .test("( 92 )", "(92)")

        .inside(NODE_PAREN).after(T!["("]).no_space_or_optional_newline()
        .inside(NODE_PAREN).before(T![")"]).no_space_or_optional_newline()
        .inside(NODE_PAREN).after(T!["("]).when(has_no_brackets).no_space_or_newline()
        .inside(NODE_PAREN).before(T![")"]).when(has_no_brackets).no_space_or_newline()

        .test("{foo = 92;}", "{ foo = 92; }")
        .inside(NODE_ATTR_SET).after(T!["{"]).single_space_or_newline()
        .inside(NODE_ATTR_SET).before(T!["}"]).single_space_or_newline()
        .test("{}", "{ }")
        .inside(NODE_ATTR_SET).between(T!["{"], T!["}"]).single_space()
        .inside(NODE_ATTR_SET).before(NODE_KEY_VALUE).single_space_or_optional_newline()
        .inside(NODE_ATTR_SET).between(NODE_KEY_VALUE, NODE_KEY_VALUE).single_space_or_newline()
        .inside(NODE_ATTR_SET).between(NODE_INHERIT, [NODE_INHERIT, TOKEN_COMMENT]).single_space_or_optional_newline()
        .inside(NODE_ATTR_SET).between(NODE_KEY_VALUE, TOKEN_COMMENT).single_space_or_optional_newline()

        .test("{arg}: 92", "{ arg }: 92")
        .inside(NODE_PATTERN).after(T!["{"]).single_space()
        .inside(NODE_PATTERN).between(T!["{"], TOKEN_COMMENT).single_space_or_newline()
        .inside(NODE_PATTERN).before(T!["}"]).single_space_or_newline()
        .test("{ }: 92", "{}: 92")
        .inside(NODE_PATTERN).between(T!["{"], T!["}"]).no_space()

        .test("{ foo,bar }: 92", "{ foo, bar }: 92")
        .inside(NODE_PATTERN).after(T![,]).single_space()
        .inside(NODE_PATTERN).before(T![,]).no_space_or_newline()

        .test("{ inherit( x )  y  z  ; }", "{ inherit (x) y z; }")
        .inside(NODE_INHERIT).around(NODE_INHERIT_FROM).single_space_or_optional_newline()
        .inside(NODE_INHERIT).around(T![;]).no_space_or_optional_newline()
        .inside(NODE_INHERIT).before(NODE_IDENT).single_space_or_optional_newline()
        .inside(NODE_INHERIT).before(NODE_OR_DEFAULT).single_space_or_optional_newline()
        .inside(NODE_INHERIT).after(NODE_IDENT).no_space_or_optional_newline()
        .inside(NODE_INHERIT_FROM).after(T!["("]).no_space()
        .inside(NODE_INHERIT_FROM).before(T![")"]).no_space()

        .inside(NODE_WITH).before(NODE_LET_IN).single_space_or_optional_newline()

        .test("let   foo = bar;in  92", "let foo = bar; in 92")
        .inside(NODE_LET_IN).after(T![let]).single_space_or_optional_newline()
        .inside(NODE_LET_IN).around(T![in]).single_space_or_optional_newline()
        .inside(NODE_LET_IN).after(NODE_KEY_VALUE).single_space_or_optional_newline()
        .inside(NODE_LET_IN).before(NODE_KEY_VALUE).when(let_header_has_newline).newline()
        .inside(NODE_LET_IN).around(T![in]).when(let_header_has_newline).newline()

        .test("{a?3}: a", "{ a ? 3 }: a")
        .inside(NODE_PAT_ENTRY).around(T![?]).single_space()

        .test("f  x", "f x")
        .inside(NODE_APPLY).between(VALUES, VALUES).single_space_or_optional_newline()
        .inside(NODE_APPLY).before(VALUES).when(should_be_newline).single_space_or_newline()

        .test("if  cond  then  tru  else  fls", "if cond then tru else fls")
        .inside(NODE_IF_ELSE).after(T![if]).single_space_or_optional_newline()
        .inside(NODE_IF_ELSE).around([T![else],T![then]]).single_space_or_optional_newline()
        .inside(NODE_IF_ELSE).after(T![then]).when(has_expression_node).single_space_or_newline()
        .inside(NODE_IF_ELSE).after(T![else]).when(has_expression_node).single_space_or_newline()
        
        // special-case to force a linebreak before `=` in
        //
        // ```nix
        // {
        //   long_key = { x
        //              , y
        //              , z
        //              }: body
        // }
        // ```
        .add_rule(dsl::SpacingRule {
            name: None,
            pattern: p(T![=]) & (p(next_sibling_is_multiline_lambda_pattern) | p(next_sibling_is_multiline_letin_pattern)) ,
            space: dsl::Space { loc: dsl::SpaceLoc::After, value: dsl::SpaceValue::Newline }
        })

        // special-cased rules for leading and trailing whitespace
        .add_rule(dsl::SpacingRule {
            name: None,
            pattern: NODE_ROOT.into(),
            space: dsl::Space { loc: dsl::SpaceLoc::Before, value: dsl::SpaceValue::None }
        })

        .add_rule(dsl::SpacingRule {
            name: None,
            pattern: NODE_ROOT.into(),
            space: dsl::Space { loc: dsl::SpaceLoc::After, value: dsl::SpaceValue::Newline }
        })

        ;

    dsl
}

fn after_literal(element: &SyntaxElement) -> bool {
    fn is_literal(kind: SyntaxKind) -> bool {
        kind == NODE_ATTR_SET || kind == NODE_LIST
    }

    let prev = prev_sibling(element);
    return if let Some(body) = prev.clone().and_then(With::cast).and_then(|w| w.body()) {
        is_literal(body.kind())
    } else {
        prev.map(|it| is_literal(it.kind())) == Some(true)
    };
}

fn has_no_brackets(element: &SyntaxElement) -> bool {
    let parent = match element.parent() {
        None => return false,
        Some(it) => it,
    };
    parent.children().all(|it| match it.kind() {
        NODE_ATTR_SET | NODE_PATTERN | NODE_LIST => false,
        NODE_IF_ELSE | NODE_BIN_OP | NODE_WITH | NODE_LAMBDA => {
            before_token_has_newline(&it.into())
        }
        NODE_APPLY => {
            if let Some(value) =
                it.first_child().and_then(rnix::types::Apply::cast).and_then(|e| e.value())
            {
                match value.kind() {
                    NODE_ATTR_SET | NODE_PAREN => return false,
                    _ => return true && before_token_has_newline(&it.into()),
                }
            }
            return false;
        }
        _ => true,
    })
}

fn inline_with_attr_set(element: &SyntaxElement) -> bool {
    fn inline_attr_set(element: &SyntaxElement) -> Option<bool> {
        let inline_attr = element
            .parent()?
            .descendants_with_tokens()
            .find(|e| e.kind() == NODE_ATTR_SET)
            .map(|t| before_token_has_newline(&t));
        inline_attr
    }
    inline_attr_set(element) == Some(false)
        && element.parent().and_then(|e| e.first_child().map(|n| n.kind() == NODE_ATTR_SET))
            == Some(true)
}

fn should_be_newline(element: &SyntaxElement) -> bool {
    let parent = match element.parent() {
        None => return false,
        Some(it) => it,
    };

    let apply = match element.parent() {
        None => return false,
        Some(it) => rnix::types::Apply::cast(it),
    };

    match element.kind() {
        NODE_APPLY => false,
        NODE_SELECT | NODE_IDENT => {
            if is_argument(element) {
                return has_newline(&parent);
            }
            false
        }
        NODE_PAREN | NODE_ATTR_SET => match last_argument_in_function(element) {
            true => {
                if let Some(fun) = apply.clone().and_then(|e| e.lambda()) {
                    match fun.kind() {
                        // node is in multi-argument function
                        NODE_APPLY => {
                            if let Some(node) = prev_non_whitespace_sibling(element) {
                                return node.as_node().map(|t| has_newline(t)).unwrap_or(false);
                            }
                            return false;
                        }
                        // node is unary function
                        _ => return false,
                    }
                }
                return false;
            }
            false => return has_newline(&parent),
        },
        _ => false,
    }
}

fn is_argument(element: &SyntaxElement) -> bool {
    match prev_sibling(element) {
        None => false,
        Some(it) => it.kind() == NODE_APPLY,
    }
}

fn last_argument_in_function(element: &SyntaxElement) -> bool {
    let is_last_argument = match element.parent() {
        None => false,
        Some(it) => match next_sibling(&it.into()) {
            None => true,
            _ => false,
        },
    };

    is_last_argument
}

fn has_expression_node(element: &SyntaxElement) -> bool {
    if let Some(el) = next_non_whitespace_sibling(element) {
        match el.kind() {
            NODE_APPLY | NODE_PAREN | NODE_LET_IN => {
                let node = match el.as_node() {
                    Some(val) => val,
                    None => return false,
                };
                return has_newline(node);
            }
            _ => return false,
        }
    }
    return false;
}

fn not_inline_if(element: &SyntaxElement) -> bool {
    fn not_inline_if_then_else(element: &SyntaxElement) -> Option<bool> {
        let first_el = element
            .parent()?
            .descendants_with_tokens()
            .take_while(|e| e.kind() != T![else])
            .filter(|element| match element {
                NodeOrToken::Token(_) => true,
                _ => false,
            })
            .any(|t| t.as_token().map(|e| e.text().contains("\n")).unwrap_or(false));

        Some(first_el)
    }

    not_inline_if_then_else(element) == Some(true)
}

fn before_token_has_newline(element: &SyntaxElement) -> bool {
    prev_token_sibling(element).map(|e| e.text().contains("\n")).unwrap_or(false)
}

fn next_sibling_is_multiline_lambda_pattern(element: &SyntaxElement) -> bool {
    fn find(element: &SyntaxElement) -> Option<bool> {
        let lambda = next_non_whitespace_sibling(element)?.into_node().and_then(Lambda::cast)?;
        let pattern = lambda.arg().and_then(rnix::types::Pattern::cast)?;
        Some(has_newline(pattern.node()))
    }
    find(element) == Some(true)
}

fn next_sibling_is_multiline_letin_pattern(element: &SyntaxElement) -> bool {
    fn find(element: &SyntaxElement) -> Option<bool> {
        let letin = next_non_whitespace_sibling(element)?.into_node().and_then(LetIn::cast)?;
        let header = letin
            .node()
            .children_with_tokens()
            .into_iter()
            .take_while(|x| match x {
                NodeOrToken::Node(_) => true,
                NodeOrToken::Token(token) => token.kind() != TOKEN_IN,
            })
            .any(|child| match child {
                NodeOrToken::Node(node) => has_newline(&node),
                NodeOrToken::Token(token) => token.text().contains("\n"),
            });
        Some(header)
    }
    find(element) == Some(true)
}

fn inline_let_in(element: &SyntaxElement) -> bool {
    element
        .as_node()
        .map(|n| {
            n.children_with_tokens()
                .take_while(|e| e.kind() != T![in])
                .any(|t| t.as_token().map(|n| n.text().contains("\n")).unwrap_or(false))
        })
        .unwrap_or(false)
}

fn let_header_has_newline(element: &SyntaxElement) -> bool {
    let letin = match element {
        NodeOrToken::Token(token) => Some(token.parent()),
        NodeOrToken::Node(node) => node.parent(),
    };
    // element.as_node().and_then(|x|x.parent());
    letin
        .map(|x| {
            let mut nodes: Vec<SyntaxElement> = x
                .children_with_tokens()
                .into_iter()
                .take_while(|x| match x {
                    NodeOrToken::Node(_) => true,
                    NodeOrToken::Token(token) => token.kind() != TOKEN_IN,
                })
                .collect();
            nodes.pop();
            nodes.into_iter().any(|child| match child {
                NodeOrToken::Node(node) => has_newline(&node),
                NodeOrToken::Token(token) => token.text().contains("\n"),
            })
        })
        .unwrap_or(false)
}

#[rustfmt::skip]
pub(crate) fn indentation() -> IndentDsl {
    let mut dsl = IndentDsl::default();
    dsl
        .anchor([NODE_PAT_ENTRY, NODE_PATTERN])


        .rule("Indent binops")
            .inside(p(NODE_BIN_OP) & p(after_concat_is_newline) & p(not_on_top_level))
            .set(Indent)
            .test(r#"
                {
                foo = bar ++
                [ baz ];
                }
            "#, r#"
                {
                  foo = bar ++
                    [ baz ];
                }
            "#)
        .rule("Indent binops top level")
            .inside(p(NODE_BIN_OP) & p(on_top_level))
            .not_matching(p(T![++]) | p(VALUES))
            .set(Indent)
            .test(r#"
                {
                foo = bar ++
                [ baz ];
                }
            "#, r#"
                {
                  foo = bar ++
                    [ baz ];
                }
            "#)

        .rule("Indent list content")
            .inside(NODE_LIST)
            .not_matching([T!["["], T!["]"]])
            .set(Indent)
            .test(r#"
                [
                92
                ]
            "#, r#"
                [
                  92
                ]
            "#)
        .rule("Indent parenthesized expressions")
            .inside(NODE_PAREN)
            .not_matching([T!["("],T![")"]])
            .set(Indent)
            .test(r#"
                (
                92
                )
            "#, r#"
                (
                  92
                )
            "#)
        
        .rule("Indent attribute set content")
            .inside(NODE_ATTR_SET)
            .not_matching([T!["{"], T!["}"]])
            .set(Indent)
            .test(r#"
                {
                foo = bar;
                }
            "#, r#"
                {
                  foo = bar;
                }
            "#)

        .rule("Indent newline let bindings ")
            .inside(p(NODE_LET_IN) & p(newline_let))
            .not_matching([T![let], T![in]])
            .set(Indent)

        .rule("Indent let bindings after key value")
            .inside(p(NODE_LET_IN) & p(no_newline_let))
            .not_matching(p([T![let], T![in], NODE_WITH, NODE_ASSERT]) | p(VALUES))
            .set(Indent)
            .test(r#"
                (
                  let
                  x = 1;
                  inherit z;
                  in
                  x
                )
            "#, r#"
                (
                  let
                    x = 1;
                    inherit z;
                  in
                  x
                )
            "#)


        .rule("Indent attribute value")
            .inside(NODE_KEY_VALUE)
            .not_matching(T![;])
            .set(Indent)
            .test(r#"
                {
                  foo =
                  92;
                }
            "#, r#"
                {
                  foo =
                    92;
                }
            "#)

        .rule("Indent lambda parameters")
            .inside(NODE_PATTERN)
            .not_matching([T!["{"], T!["}"], T![,]])
            .set(Indent)
            .test(r#"
                {
                # comment
                foo ? bar
                , baz
                }: foo
            "#, r#"
                {
                  # comment
                  foo ? bar
                , baz
                }: foo
            "#)

        .rule("Indent lambda body")
            .inside(p(NODE_LAMBDA) & p(not_on_top_level) & p(pattern_not_newline))
            .set(Indent)
        .rule("Indent newline lambda body")
            .inside(p(NODE_LAMBDA) & p(not_on_top_level) & p(pattern_newline) & p(lambda_inside_node_pattern))
            .not_matching(p(TOKEN_COMMENT))
            .set(Indent)
         .rule("Indent newline lambda body")
            .inside(p(NODE_LAMBDA) & p(not_on_top_level) & p(pattern_newline) & p(lambda_outside_node_pattern))
            .not_matching(p(TOKEN_COMMENT) | p(VALUES))
            .set(Indent)
            .test(r#"
                {}:
                  {
                foo =
                  # describe bar
                  bar:
                  # describe baz
                  baz:
                  fnbody;
                }
            "#, r#"
                {}:
                {
                  foo =
                    # describe bar
                    bar:
                    # describe baz
                    baz:
                    fnbody;
                }
            "#)

        .rule("Indent top-level apply arg")
            .inside(p(NODE_APPLY) & p(on_top_level))
            .not_matching([T!["{"], T!["}"], NODE_ATTR_SET])
            .set(Indent)
            .test(r#"
                foo
                bar baz
            "#, r#"
                foo
                  bar
                  baz
            "#)

        .rule("Indent apply arg")
            .inside(p(NODE_APPLY) & p(not_on_top_level) & p(not_inline_apply))
            .not_matching([T!["{"], T!["}"]])
            .set(Indent)
            .test(r#"
                foo
                bar baz
            "#, r#"
                foo
                  bar
                  baz
            "#)

        .rule("Indent apply arg")
            .inside(p(NODE_APPLY) & p(not_on_top_level) & p(inline_apply))
            .not_matching([T!["{"], T!["}"]])
            .set(Indent)

        .rule("Indent with body in attribute")
            .inside([NODE_WITH, NODE_ASSERT])
            .when_anchor(NODE_KEY_VALUE)
            .set(Indent)
            .test(r#"
                with foo;
                  {
                  bar = with baz;
                  body;
                  }
            "#, r#"
                with foo;
                {
                  bar = with baz;
                    body;
                }
            "#)

        .rule("Indent or default")
            .inside(NODE_OR_DEFAULT)
            .set(Indent)
            .test(r#"
                {
                  x = foo or
                  bar;
                }
            "#, r#"
                {
                  x = foo or
                    bar;
                }
            "#)

        .rule("Indent if-then-else")
            .inside(p(NODE_IF_ELSE) & p(inline_if_else))
            .not_matching(p([T![if], T![then], T![else]]) | p(VALUES))
            .set(Indent)

        .rule("Indent if-then-else")
            .inside(p(NODE_IF_ELSE) & p(not_inline_if_else))
            .not_matching([T![if], T![then], T![else], TOKEN_COMMENT])
            .set(Indent)
            .test(r#"
                if
                foo
                then
                bar
                else
                baz
            "#, r#"
                if
                  foo
                then
                  bar
                else
                  baz
            "#)

        .rule("Indent inherit parts")
            .inside(NODE_INHERIT)
            .set(Indent)
            .test(r#"
                {
                  inherit
                  (builtins)
                  # comment
                  toString
                  ;
                }
            "#, r#"
                {
                  inherit
                    (builtins)
                    # comment
                    toString
                    ;
                }
            "#)
    ;

    dsl
}

fn inline_apply(element: &SyntaxElement) -> bool {
    !not_inline_apply(element)
}

fn not_inline_apply(element: &SyntaxElement) -> bool {
    fn not_inline_function_apply(element: &SyntaxElement) -> Option<bool> {
        let first_el = element
            .ancestors()
            .filter(|e| e.kind() == NODE_APPLY)
            .max_by_key(|e| e.text_range().end());
        first_el.map(|e| before_token_has_newline(&e.into()))
    }
    not_inline_function_apply(element) == Some(true)
}

fn inline_if_else(element: &SyntaxElement) -> bool {
    !not_inline_if_else(element)
}

fn not_inline_if_else(element: &SyntaxElement) -> bool {
    fn not_inline_if_then_else(element: &SyntaxElement) -> Option<bool> {
        let first_el = element
            .as_node()?
            .descendants_with_tokens()
            .take_while(|e| e.kind() != T![else])
            .filter(|element| match element {
                NodeOrToken::Token(_) => true,
                _ => false,
            })
            .any(|t| t.as_token().map(|e| e.text().contains("\n")).unwrap_or(false));

        Some(first_el)
    }

    not_inline_if_then_else(element) == Some(true)
}

fn pattern_newline(element: &SyntaxElement) -> bool {
    fn lambda_has_newline(element: &SyntaxElement) -> Option<bool> {
        let first_el = element
            .ancestors()
            .find(|e| e.kind() == NODE_KEY_VALUE)?
            .children()
            .find(|e| e.kind() == NODE_LAMBDA)?;

        Some(
            first_el.first_child().map(|e| has_newline(&e)).unwrap_or(false)
                || before_token_has_newline(&first_el.into()),
        )
    }

    lambda_has_newline(element) == Some(true) || before_token_has_newline(element)
}

fn pattern_not_newline(element: &SyntaxElement) -> bool {
    !pattern_newline(element)
}

fn lambda_inside_node_pattern(element: &SyntaxElement) -> bool {
    element.ancestors().any(|e| e.kind() == NODE_PATTERN)
}

fn lambda_outside_node_pattern(element: &SyntaxElement) -> bool {
    !lambda_inside_node_pattern(element)
}

fn after_concat_is_newline(element: &SyntaxElement) -> bool {
    fn node_newline(element: &SyntaxElement) -> Option<bool> {
        let first_el = element.as_node()?.descendants().filter(|e| e.kind() != NODE_BIN_OP).nth(0);
        first_el.map(|e| has_newline(&e))
    }
    fn prev_newline(element: &SyntaxElement) -> Option<bool> {
        let first_el = element
            .ancestors()
            .filter(|e| e.kind() == NODE_BIN_OP)
            .max_by_key(|e| e.text_range().end());
        first_el.map(|e| before_token_has_newline(&e.into()))
    }

    node_newline(element) == Some(false) && prev_newline(element) == Some(false)
}

fn newline_let(element: &SyntaxElement) -> bool {
    let inline_let_in = inline_let_in(element);
    let inside_key_value = element.parent().map(|e| e.kind() == NODE_KEY_VALUE).unwrap_or(false);

    !before_token_has_newline(element) && inside_key_value && !inline_let_in
}

fn no_newline_let(element: &SyntaxElement) -> bool {
    !newline_let(element)
}

static VALUES: &[SyntaxKind] = &[
    NODE_LAMBDA,
    NODE_IDENT,
    NODE_SELECT,
    NODE_LET_IN,
    NODE_LIST,
    NODE_BIN_OP,
    NODE_PAREN,
    NODE_ATTR_SET,
    NODE_STRING,
    NODE_LITERAL,
    NODE_APPLY,
    NODE_IF_ELSE,
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

    use crate::{
        reformat_string,
        rules::{indentation, spacing},
    };

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

    /// For convenience, tests in this module are specified inline with a
    /// `.test` dsl methods, right next to the corresponding rule definition.
    /// This test extracts such test cases and checks them.
    #[test]
    fn test_inline_spacing_tests() {
        let rules = spacing();
        let tests: Vec<TestCase> = rules
            .tests
            .iter()
            .map(|&(before, after)| {
                let before = before.to_string();
                let after = format!("{}\n", after);
                TestCase::from_before_after(before, after)
            })
            .collect();
        run(&tests)
    }

    #[test]
    fn test_inline_indentation_tests() {
        let rules = indentation();
        let tests: Vec<TestCase> = rules
            .tests
            .iter()
            .map(|&(before, after)| {
                let before = unindent::unindent(before);
                let after = unindent::unindent(after);
                TestCase::from_before_after(before, after)
            })
            .collect();
        run(&tests)
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

    #[test]
    fn test_nixpkgs_repository_bad_good_tests() {
        let test_data = {
            let dir = env!("CARGO_MANIFEST_DIR");
            PathBuf::from(dir).join("test_data/nixpkgs_repository")
        };
        let tests = TestCase::collect_from_dir(&test_data);
        run(&tests);
    }

    #[test]
    fn test_syntax_errors_tests() {
        let test_data = {
            let dir = env!("CARGO_MANIFEST_DIR");
            PathBuf::from(dir).join("test_data/syntax_errors")
        };
        let tests = TestCase::collect_from_dir(&test_data);
        run(&tests);
    }

    #[test]
    fn test_fuzz_failures() {
        let failures_dir = {
            let dir = env!("CARGO_MANIFEST_DIR");
            PathBuf::from(dir).join("fuzz/artifacts/fmt")
        };
        for entry in fs::read_dir(failures_dir).unwrap() {
            let entry = entry.unwrap();
            if !entry.file_type().unwrap().is_file() {
                continue;
            }
            let text = fs::read_to_string(entry.path()).unwrap();
            let _ = crate::reformat_string(&text);
        }
    }

    #[derive(Debug)]
    struct TestCase {
        name: Option<String>,
        before: String,
        after: String,
    }

    impl TestCase {
        fn from_before_after(before: String, after: String) -> TestCase {
            TestCase { name: None, before, after }
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
            panic!("{} failed test cases out of {} total", n_failed, tests.len())
        }
    }
}
