//! [PARSE-FSHARP-NORMALIZE] / [PIPELINE-NORMALIZE-AST-OPERATOR] — F#
//! operators survive normalisation carrying their own token.
//!
//! `tree-sitter-fsharp` spells the operator of an infix or prefix
//! expression as a *named* `infix_op` / `prefix_op` node, never as a
//! fielded anonymous token, so the operator never reached the digest:
//! `alpha + beta` and `alpha - beta` normalised to the same subtree.

use std::path::PathBuf;

use super::FSharpParser;
use crate::{
    ast::NormalizedNode,
    lang::{shared::OPERATOR_KIND_PREFIX, LanguageParser},
    state::FileRegistry,
};

/// Addition between two identifiers.
const ADD: &str = "let f alpha beta = alpha + beta\n";
/// [`ADD`] with the operator swapped for subtraction.
const SUBTRACT: &str = "let f alpha beta = alpha - beta\n";
/// [`ADD`] with every identifier renamed — a Type-2 clone of it.
const ADD_RENAMED: &str = "let g left right = left + right\n";
/// Multiplication, the operator in the F# AST golden.
const MULTIPLY: &str = "let f alpha beta = alpha * beta\n";
/// Strictly-less-than comparison.
const LESS: &str = "let f alpha beta = alpha < beta\n";
/// [`LESS`] with the comparison inverted.
const GREATER_OR_EQUAL: &str = "let f alpha beta = alpha >= beta\n";
/// Boolean conjunction, which the grammar aliases straight to `infix_op`.
const AND: &str = "let f alpha beta = alpha && beta\n";
/// [`AND`] with the conjunction swapped for a disjunction.
const OR: &str = "let f alpha beta = alpha || beta\n";
/// A pipeline into a function — the most common F# infix operator.
const PIPE: &str = "let f alpha beta = alpha |> beta\n";

/// Parses `source` as F# into a normalised tree.
fn normalise(source: &str) -> Result<NormalizedNode, String> {
    let mut registry = FileRegistry::new();
    let file_id = registry.register(PathBuf::from("Sample.fs"));
    FSharpParser::new()
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
fn each_infix_operator_normalises_to_a_leaf_naming_its_own_token() -> Result<(), String> {
    for (source, expected) in [
        (ADD, "__op__+"),
        (SUBTRACT, "__op__-"),
        (MULTIPLY, "__op__*"),
        (LESS, "__op__<"),
        (GREATER_OR_EQUAL, "__op__>="),
        (AND, "__op__&&"),
        (OR, "__op__||"),
        (PIPE, "__op__|>"),
    ] {
        assert_eq!(
            operator_leaves(source)?,
            vec![expected],
            "`{}` must normalise to exactly one `{expected}` leaf; a missing \
             leaf means the operator never reaches the digest and every \
             operator at this position hashes alike: {:?}",
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
        (ADD, MULTIPLY),
        (LESS, GREATER_OR_EQUAL),
        (AND, OR),
    ] {
        assert_ne!(
            kinds(left)?,
            kinds(right)?,
            "`{}` and `{}` compute different answers, so their normalised \
             trees must differ — equal trees hash alike and publish as \
             duplicates",
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
        "a renamed copy keeps its operators, so Type-2 recall must survive \
         operator leaves reaching the digest"
    );
    assert!(
        !kinds(ADD)?.contains(&"infix_op"),
        "the opaque `infix_op` kind must not survive beside the operator \
         leaf that replaces it: {:?}",
        kinds(ADD)?,
    );
    Ok(())
}
