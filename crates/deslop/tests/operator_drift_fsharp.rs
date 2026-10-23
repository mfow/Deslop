//! [PIPELINE-NORMALIZE-AST-OPERATOR] / [PARSE-FSHARP-NORMALIZE] — an F#
//! operator is behaviour, and two functions that differ only in their
//! operators are not duplicates.
//!
//! `tree-sitter-fsharp` spells every infix operator as a *named*
//! `infix_op` node rather than a fielded anonymous token, so the shared
//! operator rule never saw it: `alpha + beta` and `alpha - beta`
//! normalised to the same subtree and published as duplicates. The F#
//! AST golden recorded the opaque `infix_op` for as long as it existed.
//!
//! The families mirror the Python fixture — arithmetic, comparison and
//! boolean operators, plus a ledger body differing in one `+`/`-` — and
//! are judged by the same contract in `common::operator_drift`, beside a
//! byte-identical control in the same run.

use crate::common::{operator_drift::*, *};

/// The operator families and one source line on which their computations
/// disagree.
const FAMILIES: [Family; 4] = [
    Family {
        label: "F# arithmetic + / -",
        left: "arithmetic_add.fs",
        right: "arithmetic_sub.fs",
        left_leaves: &["__op__+", "__op__+", "__op__+", "__op__+"],
        right_leaves: &["__op__-", "__op__-", "__op__-", "__op__-"],
        changed_line: 4,
    },
    Family {
        label: "F# comparison < / >",
        left: "comparison_lt.fs",
        right: "comparison_gt.fs",
        left_leaves: &["__op__<", "__op__<", "__op__<", "__op__<", "__op__<"],
        right_leaves: &["__op__>", "__op__>", "__op__>", "__op__>", "__op__>"],
        changed_line: 4,
    },
    Family {
        label: "F# boolean && / ||",
        left: "boolean_and.fs",
        right: "boolean_or.fs",
        left_leaves: &["__op__&&", "__op__&&", "__op__&&", "__op__&&"],
        right_leaves: &["__op__||", "__op__||", "__op__||", "__op__||"],
        changed_line: 4,
    },
    LEDGER,
];

/// A twelve-line body differing in exactly one `+`/`-`. The mutation
/// arrow `<-` is assignment framing, not an operator, so it emits no leaf.
const LEDGER: Family = Family {
    label: "F# ledger + / - inside a shared body",
    left: "ledger_credit.fs",
    right: "ledger_debit.fs",
    left_leaves: &["__op__*", "__op__+", "__op__+", "__op__/", "__op__-"],
    right_leaves: &["__op__*", "__op__-", "__op__+", "__op__/", "__op__-"],
    changed_line: 8,
};

/// The F# fixture: four operator pairs plus the control.
const FSHARP: DriftFixture = DriftFixture {
    dir: "operator-drift-fsharp",
    min_nodes: 8,
    families: &FAMILIES,
    ledger: &LEDGER,
    control: ["control_alpha.fs", "control_beta.fs"],
    file_count: 10,
};

#[test]
fn an_fsharp_operator_only_difference_never_claims_duplication() -> Result<()> {
    assert_operator_only_difference_never_claims_duplication(&FSHARP)
}

#[test]
fn the_fsharp_byte_identical_control_survives_in_the_same_run() -> Result<()> {
    assert_byte_identical_control_survives(&FSHARP)
}

#[test]
fn every_cluster_ahead_of_the_fsharp_control_is_proven_duplication() -> Result<()> {
    assert_every_cluster_ahead_of_control_is_proven(&FSHARP)
}

#[test]
fn each_fsharp_family_member_normalises_to_its_own_operator_leaves() -> Result<()> {
    assert_each_member_normalises_to_its_own_operator_leaves(&FSHARP)
}
