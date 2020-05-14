//! This module contains specific `super::dsl` rules for formatting nix language.
use rnix::{
    types::{Lambda, TypedNode, With},
    NodeOrToken, SyntaxElement, SyntaxKind,
    SyntaxKind::*,
    T,
};

use crate::{
    dsl::{self, IndentDsl, IndentValue::*, SpacingDsl},
    pattern::p,
    tree_utils::{
        has_newline, next_non_whitespace_sibling, next_token_sibling, not_on_top_level,
        on_top_level, prev_non_whitespace_parent, prev_non_whitespace_sibling, prev_sibling,
        prev_token_parent, prev_token_sibling,
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
        //.inside(NODE_LAMBDA).before(NODE_IDENT).when(prev_parent_is_newline).when(next_is_select_attrset).newline()

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
        //.inside(NODE_PAREN).before(NODE_APPLY).when()
        //.inside(NODE_PAREN).before(T![")"]).when(paren_open_newline).newline()
        //.inside(NODE_PAREN).before(T![")"]).when(node_inside_paren).when(between_open_paren_newline).newline()
        //.inside(NODE_PAREN).before(T![")"]).when(node_inside_paren).when(between_open_paren_not_newline).no_space()

        //.inside(NODE_PAREN).before(T![")"]).when(prev_is_let).newline()
        //.inside(NODE_PAREN).before(T![")"]).when(prev_is_if).when(not_inside_node_interpol).newline()
        //.inside(NODE_PAREN).before(T![")"]).when(inside_multiple_argument_function).newline()
        .test("{foo = 92;}", "{ foo = 92; }")
        .inside(NODE_ATTR_SET).after(T!["{"]).single_space_or_newline()
        .inside(NODE_ATTR_SET).before(T!["}"]).single_space_or_newline()
        .test("{}", "{ }")
        .inside(NODE_ATTR_SET).between(T!["{"], T!["}"]).single_space()
        .inside(NODE_ATTR_SET).before(NODE_KEY_VALUE).single_space_or_optional_newline()
        .inside(NODE_ATTR_SET).between(NODE_KEY_VALUE, NODE_KEY_VALUE).single_space_or_newline()
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
        .inside(NODE_INHERIT).after(NODE_IDENT).no_space_or_optional_newline()
        .inside(NODE_INHERIT_FROM).after(T!["("]).no_space()
        .inside(NODE_INHERIT_FROM).before(T![")"]).no_space()

        .test("let   foo = bar;in  92", "let foo = bar; in 92")
        .inside(NODE_LET_IN).after(T![let]).single_space_or_optional_newline()
        .inside(NODE_LET_IN).around(T![in]).single_space_or_optional_newline()
        .inside(NODE_LET_IN).before(T![in]).when(header_has_multi_value).when(before_token_not_newline).newline()
        .inside(NODE_LET_IN).after(T![in]).when(in_body_newline).newline()
        .inside(NODE_LET_IN).before(NODE_KEY_VALUE).when(header_has_multi_value).when(before_token_not_newline).newline()

        .test("{a?3}: a", "{ a ? 3 }: a")
        .inside(NODE_PAT_ENTRY).around(T![?]).single_space()

        .test("f  x", "f x")
        .inside(NODE_APPLY).between(VALUES, VALUES).single_space_or_optional_newline()
        .inside(NODE_APPLY).before(NODE_PAREN).when(last_argument_in_function).single_space()
        .inside(NODE_APPLY).before(NODE_PAREN).when(last_argument_in_function).when(multi_argument_in_function).when(between_argument_has_newline).newline()
        .inside(NODE_APPLY).before(NODE_PAREN).when(non_last_argument_in_function).when(between_argument_has_newline).when(not_inside_node_interpol).newline()
        .inside(NODE_APPLY).before(NODE_PAREN).when(inside_node_interpol).single_space_or_optional_newline()
        .inside(NODE_APPLY).before(NODE_LIST).when(last_argument_in_function).single_space()
        .inside(NODE_APPLY).before(NODE_LIST).when(last_argument_in_function).when(multi_argument_in_function).when(between_argument_has_newline).newline()
        .inside(NODE_APPLY).before(NODE_ATTR_SET).when(last_argument_in_function).single_space()
        .inside(NODE_APPLY).before(NODE_ATTR_SET).when(non_last_argument_in_function).when(node_apply_has_newline).when(not_inside_node_interpol).newline()
        .inside(NODE_APPLY).before(NODE_IDENT).when(node_is_argument).when(last_argument_in_function).when(node_apply_has_newline).when(not_inside_node_interpol).newline()
        .inside(NODE_APPLY).before(NODE_IDENT).when(node_is_argument).when(non_last_argument_in_function).when(between_argument_has_newline).when(not_inside_node_interpol).newline()
        .inside(NODE_APPLY).before(NODE_SELECT).when(node_is_argument).when(last_argument_in_function).when(node_apply_has_newline).when(not_inside_node_interpol).newline()
        .inside(NODE_APPLY).before(NODE_SELECT).when(node_is_argument).when(non_last_argument_in_function).when(between_argument_has_newline).when(not_inside_node_interpol).newline()
        .inside(NODE_APPLY).before(NODE_IDENT).when(node_is_function).when(outside_inline_pattern).when(between_argument_has_newline).when(not_inside_node_interpol).newline()
        .inside(NODE_APPLY).before(NODE_SELECT).when(node_is_function).when(outside_inline_pattern).when(between_argument_has_newline).when(not_inside_node_interpol).newline()

        .test("if  cond  then  tru  else  fls", "if cond then tru else fls")
        .inside(NODE_IF_ELSE).before(T![if]).when(not_on_top_level).when(not_inline_if).single_space_or_newline()
        .inside(NODE_IF_ELSE).before(T![if]).when(after_else_is_inline_if).single_space()
        .inside(NODE_IF_ELSE).before(T![if]).when(inside_node_interpol).when(between_if_then_not_newline).no_space()
        .inside(NODE_IF_ELSE).before(T![then]).when(before_token_has_newline).newline()
        .inside(NODE_IF_ELSE).before(T![then]).single_space_or_optional_newline()
        .inside(NODE_IF_ELSE).after([T![if],T![then]]).single_space_or_optional_newline()
        .inside(NODE_IF_ELSE).around(T![else]).single_space_or_optional_newline()
        .inside(NODE_IF_ELSE).after(T![else]).when(after_else_has_newline).newline()

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
            pattern: p(T![=]) & p(next_sibling_is_multiline_lambda_pattern),
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

        .add_rule(dsl::SpacingRule {
            name: None,
            pattern: p(T![let]) & p(after_let_newline),
            space: dsl::Space { loc: dsl::SpaceLoc::After, value: dsl::SpaceValue::Newline }
        })

        .add_rule(dsl::SpacingRule {
            name: None,
            pattern: p(T![let]) & p(let_inside_lambda_or_paren),
            space: dsl::Space { loc: dsl::SpaceLoc::Before, value: dsl::SpaceValue::Newline }
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

fn node_apply_has_newline(element: &SyntaxElement) -> bool {
    fn top_level_node_function(element: &SyntaxElement) -> Option<bool> {
        element
            .parent()?
            .ancestors()
            .take_while(|e| {
                e.kind() != NODE_KEY_VALUE && e.kind() != NODE_IF_ELSE && e.kind() != NODE_PAREN
            })
            .filter(|e| e.kind() == NODE_APPLY)
            .max_by_key(|e| e.text_range().start())
            .map(|e| has_newline(&e))
    }
    top_level_node_function(element).unwrap_or(false)
}

fn between_argument_has_newline(element: &SyntaxElement) -> bool {
    fn newline_in_between(element: &SyntaxElement) -> bool {
        let prev_argument =
            prev_token_sibling(element).map(|e| e.text().contains("\n")).unwrap_or(false);
        let prev_sibling = prev_sibling(element).map(|e| has_newline(&e)).unwrap_or(false);
        prev_argument || prev_sibling
    }

    fn between_arg_newline(element: &SyntaxElement) -> Option<bool> {
        let list_node_apply = element
            .parent()?
            .ancestors()
            .take_while(|e| {
                e.kind() != NODE_KEY_VALUE && e.kind() != NODE_IF_ELSE && e.kind() != NODE_PAREN
            })
            .filter_map(|e| match e.kind() {
                NODE_APPLY => e.last_child(),
                _ => None,
            });
        let exist_newline = list_node_apply.fold(false, |b, e| b || newline_in_between(&e.into()));
        Some(exist_newline)
    }

    between_arg_newline(element).unwrap_or(false)
}

fn node_is_argument(element: &SyntaxElement) -> bool {
    fn is_argument(element: &SyntaxElement) -> Option<bool> {
        let element_text_range = element.text_range().start();
        Some(element.parent()?.last_child()?.text_range().start() == element_text_range)
    }
    is_argument(element).unwrap_or(false)
}

fn last_argument_in_function(element: &SyntaxElement) -> bool {
    fn is_last_argument(element: &SyntaxElement) -> Option<bool> {
        let inside_apply = element.parent()?.kind() == NODE_APPLY;
        let last_argument = element.parent()?.parent()?.kind() != NODE_APPLY;
        Some(last_argument && inside_apply)
    }
    is_last_argument(element).unwrap_or(false)
}

// Check whether the function is unary function or not
fn multi_argument_in_function(element: &SyntaxElement) -> bool {
    prev_sibling(element).map(|e| e.kind() == NODE_APPLY).unwrap_or(false)
}

fn non_last_argument_in_function(element: &SyntaxElement) -> bool {
    !last_argument_in_function(element)
}

fn node_is_function(element: &SyntaxElement) -> bool {
    fn is_function(element: &SyntaxElement) -> Option<bool> {
        let element_text_range = element.text_range().start();
        Some(element.parent()?.first_child()?.text_range().start() == element_text_range)
    }
    is_function(element).unwrap_or(false) && not_on_top_level(element)
}

// Special case if function is nested inside certain node
fn outside_inline_pattern(element: &SyntaxElement) -> bool {
    let node = element.ancestors().find(|e| {
        e.kind() == NODE_PATTERN
            || e.kind() == NODE_BIN_OP
            || e.kind() == NODE_IF_ELSE
            || e.kind() == NODE_WITH
            || e.kind() == NODE_INHERIT
            || e.kind() == NODE_OR_DEFAULT
            || e.kind() == NODE_ASSERT
            || e.kind() == NODE_LAMBDA
    });

    node.and_then(|e| match e.kind() {
        NODE_ASSERT => {
            let closing_semicolon = element.text_range().start();
            let has_newline = e
                .descendants_with_tokens()
                .filter(|element| match element {
                    NodeOrToken::Token(_) => true,
                    _ => false,
                })
                .take_while(|e| e.text_range().start() != closing_semicolon)
                .any(|t| t.as_token().map(|n| n.text().contains("\n")).unwrap_or(false));
            Some(has_newline)
        }
        NODE_LAMBDA => Some(false),

        NODE_IF_ELSE => Some(false),

        _ => Some(true),
    })
    .unwrap_or(true)
}

/*fn paren_open_newline(element: &SyntaxElement) -> bool {
    fn after_paren_open_newline(element: &SyntaxElement) -> Option<bool> {
        element
            .parent()?
            .first_child_or_token()
            .and_then(|e| next_token_sibling(&e).map(|e| e.text().contains("\n")))
    }
    after_paren_open_newline(element) == Some(true)
}

// Check whether there is exists function call inside parentheses
fn inside_multiple_argument_function(element: &SyntaxElement) -> bool {
    fn multiple_argument_function(element: &SyntaxElement) -> Option<bool> {
        let first_node = element.parent()?.first_child()?;
        let contain_node_apply = first_node.clone().kind() == NODE_APPLY;
        let contain_multiple_argument_function =
            multi_argument_in_function(&first_node.clone().last_child()?.into());
        if contain_node_apply && contain_multiple_argument_function {
            let exist_newline = first_node
                .clone()
                .descendants_with_tokens()
                .filter(|element| match element {
                    NodeOrToken::Token(_) => true,
                    _ => false,
                })
                .any(|t| t.as_token().map(|n| n.text().contains("\n")).unwrap_or(false));
            Some(exist_newline)
        } else {
            None
        }
    }

    multiple_argument_function(element).unwrap_or(false)
}

fn multiline_string(element: &SyntaxElement) -> bool {
    if element.kind() == NODE_STRING {
        element.as_node().map(|e| has_newline(&e)).unwrap_or(false);
    }
    false
}*/

fn has_no_brackets(element: &SyntaxElement) -> bool {
    let parent = match element.parent() {
        None => return true,
        Some(it) => it,
    };
    parent.children().all(|it| {
        it.kind() != NODE_ATTR_SET
            && it.kind() != NODE_PATTERN
            && it.kind() != NODE_LAMBDA
            && it.kind() != NODE_APPLY
            && it.kind() != NODE_WITH
            && it.kind() != NODE_BIN_OP
            && it.kind() != NODE_IF_ELSE
            && it.kind() != NODE_LIST
    })
}

/*fn node_inside_paren(element: &SyntaxElement) -> bool {
    fn inside_paren_exist_node(element: &SyntaxElement) -> Option<bool> {
        let open_paren_token_unit = element.parent()?.first_child_or_token()?.text_range().start();
        let paren_contain_node_unit = element
            .parent()?
            .descendants()
            .find(|e| {
                e.kind() == NODE_LIST || e.kind() == NODE_ATTR_SET || multiline_string(element)
            })?
            .ancestors()
            .find(|e| e.kind() == NODE_PAREN)?
            .text_range()
            .start();
        Some(open_paren_token_unit == paren_contain_node_unit)
    }

    fn node_outside_key_value(element: &SyntaxElement) -> Option<bool> {
        let paren_contain_node_unit = element
            .parent()?
            .descendants_with_tokens()
            .take_while(|e| {
                e.kind() != NODE_LET_IN && e.kind() != NODE_KEY_VALUE && e.kind() != NODE_IF_ELSE
            })
            .any(|t| t.kind() == NODE_LIST || t.kind() == NODE_ATTR_SET || t.kind() == NODE_STRING);

        Some(paren_contain_node_unit)
    }

    inside_paren_exist_node(element).unwrap_or(false)
        && node_outside_key_value(element) == Some(true)
}

fn between_open_paren_not_newline(element: &SyntaxElement) -> bool {
    !between_open_paren_newline(element)
}

fn between_open_paren_newline(element: &SyntaxElement) -> bool {
    fn after_paren_open_newline(element: &SyntaxElement) -> Option<bool> {
        let between_paren_node_has_newline = element
            .parent()?
            .descendants_with_tokens()
            .take_while(|n| {
                n.kind() != NODE_ATTR_SET && n.kind() != NODE_LIST && n.kind() != NODE_STRING
            })
            .filter(|element| match element {
                NodeOrToken::Token(_) => true,
                _ => false,
            })
            .any(|t| t.as_token().map(|n| n.text().contains("\n")).unwrap_or(false));

        Some(between_paren_node_has_newline)
    }
    after_paren_open_newline(element) == Some(true)
}

fn next_is_select_attrset(element: &SyntaxElement) -> bool {
    let select = next_sibling(element).map(|n| n.kind() == NODE_SELECT).unwrap_or(false);
    let attrset = next_sibling(element).map(|n| n.kind() == NODE_ATTR_SET).unwrap_or(false);
    select || attrset
}
*/
fn after_else_is_inline_if(element: &SyntaxElement) -> bool {
    let token_else = prev_non_whitespace_parent(element)
        .and_then(|e| e.into_token().map(|t| t.kind() == T![else]))
        .unwrap_or(false);
    let has_newline = prev_token_parent(element).map(|t| t.text().contains("\n")).unwrap_or(false);
    token_else & !has_newline
}

fn not_inside_node_interpol(element: &SyntaxElement) -> bool {
    !inside_node_interpol(element)
}

fn inside_node_interpol(element: &SyntaxElement) -> bool {
    element.ancestors().any(|n| n.kind() == NODE_STRING_INTERPOL)
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

fn between_if_then_not_newline(element: &SyntaxElement) -> bool {
    fn not_inline_if_then_else(element: &SyntaxElement) -> Option<bool> {
        let first_el = element
            .parent()?
            .descendants_with_tokens()
            .take_while(|e| e.kind() != T![then])
            .filter(|element| match element {
                NodeOrToken::Token(_) => true,
                _ => false,
            })
            .any(|t| t.as_token().map(|e| e.text().contains("\n")).unwrap_or(false));

        Some(first_el)
    }

    not_inline_if_then_else(element) == Some(false)
}

// This function to make sure let..in inside else get expanded
fn after_else_has_newline(element: &SyntaxElement) -> bool {
    next_non_whitespace_sibling(element)
        .and_then(|e| match e.as_node() {
            Some(node) => {
                if node.kind() == NODE_LET_IN {
                    Some(has_newline(&node))
                } else {
                    Some(false)
                }
            }
            _ => Some(false),
        })
        .unwrap_or(false)
}

/*fn prev_is_let(element: &SyntaxElement) -> bool {
    prev_non_whitespace_sibling(element)
        .and_then(|e| e.into_node().map(|n| n.kind() == NODE_LET_IN))
        .unwrap_or(false)
}

fn prev_is_if(element: &SyntaxElement) -> bool {
    let node_if = prev_non_whitespace_sibling(element)
        .and_then(|e| e.into_node().map(|n| n.kind() == NODE_IF_ELSE))
        .unwrap_or(false);
    let is_expanded_if = element.parent().map(|e| has_newline(&e)).unwrap_or(false);
    node_if && is_expanded_if
}

fn prev_parent_is_newline(element: &SyntaxElement) -> bool {
    prev_token_parent(element).map(|n| n.text().contains('\n')).unwrap_or(false)
}

fn paren_on_top_level(element: &SyntaxElement) -> bool {
    let parent = match element.parent() {
        None => return true,
        Some(it) => it,
    };
    match parent.kind() {
        NODE_ROOT => true,
        NODE_SELECT | NODE_PAREN | NODE_APPLY => paren_on_top_level(&parent.into()),
        _ => false,
    }
}*/

fn before_token_has_newline(element: &SyntaxElement) -> bool {
    prev_token_sibling(element).map(|e| e.text().contains("\n")).unwrap_or(false)
}

fn before_token_not_newline(element: &SyntaxElement) -> bool {
    !before_token_has_newline(element)
}

fn next_sibling_is_multiline_lambda_pattern(element: &SyntaxElement) -> bool {
    fn find(element: &SyntaxElement) -> Option<bool> {
        let lambda = next_non_whitespace_sibling(element)?.into_node().and_then(Lambda::cast)?;
        let pattern = lambda.arg().and_then(rnix::types::Pattern::cast)?;
        Some(has_newline(pattern.node()))
    }
    find(element) == Some(true)
}

fn header_has_multi_value(element: &SyntaxElement) -> bool {
    let key_value = element.parent().map(|e| {
        e.children().fold(0, |mut v, e| {
            if e.kind() == NODE_KEY_VALUE {
                v += 1;
                return v;
            } else {
                v += 0;
                return v;
            }
        })
    });
    key_value.map(|e| e >= 2).unwrap_or(false)
}

// special-cased to force a linebreak after `let` when `in` is not inline and key value >= 2
fn after_let_newline(element: &SyntaxElement) -> bool {
    header_has_multi_value(element)
}

fn in_body_newline(element: &SyntaxElement) -> bool {
    fn body_has_newline(element: &SyntaxElement) -> Option<bool> {
        next_non_whitespace_sibling(element)?.as_node().map(|e| has_newline(e))
    }

    fn header_is_multiline(element: &SyntaxElement) -> Option<bool> {
        next_token_sibling(&element.parent()?.first_child_or_token()?)
            .map(|e| e.text().contains('\n'))
    }

    body_has_newline(element) == Some(true)
        || (header_is_multiline(element) == Some(true) || header_has_multi_value(element))
}

fn let_inside_lambda_or_paren(element: &SyntaxElement) -> bool {
    fn find(element: &SyntaxElement) -> Option<bool> {
        let parent_let = element.parent()?;
        let inside_lambda_pattern =
            prev_non_whitespace_parent(element)?.into_token()?.text().contains(':');
        let inside_paren_pattern =
            prev_non_whitespace_parent(element)?.into_token()?.text().contains('(');
        let inside_assign_pattern =
            prev_non_whitespace_parent(element)?.into_token()?.text().contains('=');
        let prev_let_linebreak_pattern = match parent_let.prev_sibling_or_token()? {
            NodeOrToken::Node(_) => false,
            NodeOrToken::Token(it) => it.text().contains('\n'),
        };
        let between_let_in_newline = parent_let
            .children_with_tokens()
            .take_while(|e| e.kind() != T![in])
            .any(|t| t.as_token().map(|n| n.text().contains("\n")).unwrap_or(false));

        Some(
            prev_let_linebreak_pattern
                || inside_lambda_pattern
                || inside_paren_pattern
                || (inside_assign_pattern
                    && (header_has_multi_value(element) || between_let_in_newline)),
        )
    }

    find(element) == Some(true)
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
    let inline_let_in = element
        .as_node()
        .map(|n| {
            n.children_with_tokens()
                .take_while(|e| e.kind() != T![in])
                .any(|t| t.as_token().map(|n| n.text().contains("\n")).unwrap_or(false))
        })
        .unwrap_or(false);

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
