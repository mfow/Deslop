//! [PARSE-OCAML-NORMALIZE] / [PIPELINE-NORMALIZE-AST-OPERATOR] — OCaml
//! names collapse, literals collapse, and operators survive carrying
//! their own token.
//!
//! `tree-sitter-ocaml` groups operators by precedence class rather than
//! by token: `+` and `-` are both `add_operator`, `|>` and `>>=` are both
//! `rel_operator`. The class alone cannot tell a sign error from correct
//! code, so each operator node must become a leaf naming its own token.

use std::path::PathBuf;

use super::OCamlParser;
use crate::{
    ast::NormalizedNode,
    lang::{
        shared::{IDENTIFIER_KIND, LITERAL_KIND, OPERATOR_KIND_PREFIX},
        LanguageParser,
    },
    state::FileRegistry,
};

/// Addition between two names.
const ADD: &str = "let f alpha beta = alpha + beta\n";
/// [`ADD`] with the operator swapped for subtraction.
const SUBTRACT: &str = "let f alpha beta = alpha - beta\n";
/// [`ADD`] with every name renamed — a Type-2 clone of it.
const ADD_RENAMED: &str = "let g left right = left + right\n";
/// A pipeline — `rel_operator`, the same class as [`BIND`].
const PIPE: &str = "let f alpha beta = alpha |> beta\n";
/// A monadic bind — `rel_operator`, the same class as [`PIPE`].
const BIND: &str = "let f alpha beta = alpha >>= beta\n";
/// Strictly-less-than comparison.
const LESS: &str = "let f alpha beta = alpha < beta\n";
/// [`LESS`] with the comparison inverted.
const GREATER_OR_EQUAL: &str = "let f alpha beta = alpha >= beta\n";
/// Boolean conjunction.
const AND: &str = "let f alpha beta = alpha && beta\n";
/// [`AND`] with the conjunction swapped for a disjunction.
const OR: &str = "let f alpha beta = alpha || beta\n";
/// Float multiplication.
const FLOAT_MULTIPLY: &str = "let f alpha beta = alpha *. beta\n";
/// A keyword operator, which the grammar wraps in `mult_operator`.
const MODULO: &str = "let f alpha beta = alpha mod beta\n";
/// Dereference, a prefix operator.
const DEREFERENCE: &str = "let f cell = !cell\n";
/// Float negation, a sign operator.
const NEGATE: &str = "let f alpha = -. alpha\n";
/// A binding operator: `let*` is bind, `let+` is map.
const LET_STAR: &str = "let f alpha = let* beta = alpha in beta\n";
/// [`LET_STAR`] with bind swapped for map.
const LET_PLUS: &str = "let f alpha = let+ beta = alpha in beta\n";
/// Every literal form the grammar spells, plus a comment and a doc comment.
const LITERALS: &str = "(* gone *)\n(** gone *)\nlet f () =\n  \
                        (1, -2, 3.5, 'x', \"text\", {|raw|}, true, ())\n";
/// [`LITERALS`] with every constant changed.
const LITERALS_CHANGED: &str = "(* gone *)\n(** gone *)\nlet f () =\n  \
                                (9, -8, 7.5, 'y', \"other\", {|other|}, false, ())\n";
/// Constructors, a polymorphic variant, a record field and a module path.
const NAMES: &str = "let f point = (Some point.x, `Red, List.length [])\n";
/// [`NAMES`] with every name renamed.
const NAMES_RENAMED: &str = "let g spot = (Ok spot.y, `Blue, Array.length [])\n";

/// Parses `source` as OCaml into a normalised tree.
fn normalise(source: &str) -> Result<NormalizedNode, String> {
    let mut registry = FileRegistry::new();
    let file_id = registry.register(PathBuf::from("sample.ml"));
    OCamlParser::new()
        .parse_and_normalize(source.as_bytes(), file_id)
        .map_err(|error| format!("the probe must parse: {error}"))
}

/// Every normalised kind in the tree, depth first, in source order.
fn kinds(source: &str) -> Result<Vec<&'static str>, String> {
    let root = normalise(source)?;
    let mut found = Vec::new();
    let mut stack = vec![&root];
    while let Some(node) = stack.pop() {
        found.push(node.kind);
        stack.extend(node.children.iter().rev());
    }
    Ok(found)
}

/// The operator leaves `source` normalises to, in source order.
fn operator_leaves(source: &str) -> Result<Vec<&'static str>, String> {
    Ok(kinds(source)?
        .into_iter()
        .filter(|kind| kind.starts_with(OPERATOR_KIND_PREFIX))
        .collect())
}

#[test]
fn each_operator_normalises_to_a_leaf_naming_its_own_token() -> Result<(), String> {
    for (source, expected) in [
        (ADD, "__op__+"),
        (SUBTRACT, "__op__-"),
        (PIPE, "__op__|>"),
        (BIND, "__op__>>="),
        (LESS, "__op__<"),
        (GREATER_OR_EQUAL, "__op__>="),
        (AND, "__op__&&"),
        (OR, "__op__||"),
        (FLOAT_MULTIPLY, "__op__*."),
        (MODULO, "__op__mod"),
        (DEREFERENCE, "__op__!"),
        (NEGATE, "__op__-."),
        (LET_STAR, "__op__let*"),
        (LET_PLUS, "__op__let+"),
    ] {
        assert_eq!(
            operator_leaves(source)?,
            vec![expected],
            "`{}` must normalise to exactly one `{expected}` leaf; the \
             grammar's precedence class alone hashes every operator of that \
             class alike: {:?}",
            source.trim_end(),
            kinds(source)?,
        );
    }
    Ok(())
}

#[test]
fn an_operator_swap_changes_the_normalised_tree() -> Result<(), String> {
    for (left, right) in [
        (ADD, SUBTRACT),
        (PIPE, BIND),
        (LESS, GREATER_OR_EQUAL),
        (AND, OR),
        (LET_STAR, LET_PLUS),
    ] {
        assert_ne!(
            kinds(left)?,
            kinds(right)?,
            "`{}` and `{}` compute different answers, so their normalised \
             trees must differ",
            left.trim_end(),
            right.trim_end(),
        );
    }
    Ok(())
}

#[test]
fn a_consistent_rename_still_normalises_identically() -> Result<(), String> {
    assert_eq!(
        kinds(ADD)?,
        kinds(ADD_RENAMED)?,
        "a renamed copy keeps its operators, so Type-2 recall must survive"
    );
    assert_eq!(
        kinds(NAMES)?,
        kinds(NAMES_RENAMED)?,
        "constructors, polymorphic variants, record fields and module paths \
         are names, so renaming them must not change the tree"
    );
    let names = kinds(NAMES)?;
    for raw in [
        "value_name",
        "constructor_name",
        "field_name",
        "module_name",
        "tag",
    ] {
        assert!(
            !names.contains(&raw),
            "`{raw}` must collapse to {IDENTIFIER_KIND}: {names:?}"
        );
    }
    assert!(
        names.contains(&IDENTIFIER_KIND),
        "names must collapse to {IDENTIFIER_KIND}, not vanish: {names:?}"
    );
    Ok(())
}

#[test]
fn literals_collapse_and_comments_drop() -> Result<(), String> {
    assert_eq!(
        kinds(LITERALS)?,
        kinds(LITERALS_CHANGED)?,
        "a constant edit must not change the normalised tree"
    );
    let literal_kinds = kinds(LITERALS)?;
    for raw in [
        "number",
        "signed_number",
        "character",
        "string",
        "quoted_string",
        "boolean",
        "unit",
        "comment",
    ] {
        assert!(
            !literal_kinds.contains(&raw),
            "`{raw}` must collapse to {LITERAL_KIND} or drop as trivia: \
             {literal_kinds:?}"
        );
    }
    assert!(
        literal_kinds.contains(&LITERAL_KIND),
        "literals must collapse to {LITERAL_KIND}, not vanish: {literal_kinds:?}"
    );
    Ok(())
}
