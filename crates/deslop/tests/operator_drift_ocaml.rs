//! [PIPELINE-NORMALIZE-AST-OPERATOR] / [PARSE-OCAML-NORMALIZE] — an OCaml
//! operator is behaviour, and two functions that differ only in their
//! operators are not duplicates.
//!
//! `tree-sitter-ocaml` names an operator by its precedence class —
//! `add_operator` for both `+` and `-` — so the class alone hashes a
//! sign error exactly like the correct code. The families mirror the
//! Python and F# fixtures and are judged by the same contract in
//! `common::operator_drift`, beside a byte-identical control.

use crate::common::{operator_drift::*, *};

/// The operator families and one source line on which their computations
/// disagree.
const FAMILIES: [Family; 4] = [
    Family {
        label: "OCaml arithmetic + / -",
        left: "arithmetic_add.ml",
        right: "arithmetic_sub.ml",
        left_leaves: &["__op__+", "__op__+", "__op__+", "__op__+"],
        right_leaves: &["__op__-", "__op__-", "__op__-", "__op__-"],
        changed_line: 2,
    },
    Family {
        label: "OCaml comparison < / >",
        left: "comparison_lt.ml",
        right: "comparison_gt.ml",
        left_leaves: &["__op__<", "__op__<", "__op__<", "__op__<", "__op__<"],
        right_leaves: &["__op__>", "__op__>", "__op__>", "__op__>", "__op__>"],
        changed_line: 2,
    },
    Family {
        label: "OCaml boolean && / ||",
        left: "boolean_and.ml",
        right: "boolean_or.ml",
        left_leaves: &["__op__&&", "__op__&&", "__op__&&", "__op__&&"],
        right_leaves: &["__op__||", "__op__||", "__op__||", "__op__||"],
        changed_line: 2,
    },
    LEDGER,
];

/// A twelve-line body differing in exactly one `+.`/`-.`. Every `:=`
/// assignment and `!` dereference is an operator leaf too; the list
/// cons `::` is a fixed production with no operator to vary.
const LEDGER: Family = Family {
    label: "OCaml ledger +. / -. inside a shared body",
    left: "ledger_credit.ml",
    right: "ledger_debit.ml",
    left_leaves: &[
        "__op__*.", "__op__+.", "__op__:=", "__op__!", "__op__:=", "__op__!", "__op__+.",
        "__op__!", "__op__/.", "__op__!", "__op__!", "__op__-.", "__op__!", "__op__!",
    ],
    right_leaves: &[
        "__op__*.", "__op__-.", "__op__:=", "__op__!", "__op__:=", "__op__!", "__op__+.",
        "__op__!", "__op__/.", "__op__!", "__op__!", "__op__-.", "__op__!", "__op__!",
    ],
    changed_line: 7,
};

/// The OCaml fixture: four operator pairs plus the control.
const OCAML: DriftFixture = DriftFixture {
    dir: "operator-drift-ocaml",
    min_nodes: 8,
    families: &FAMILIES,
    ledger: &LEDGER,
    control: ["control_alpha.ml", "control_beta.ml"],
    file_count: 10,
};

#[test]
fn an_ocaml_operator_only_difference_never_claims_duplication() -> Result<()> {
    assert_operator_only_difference_never_claims_duplication(&OCAML)
}

#[test]
fn the_ocaml_byte_identical_control_survives_in_the_same_run() -> Result<()> {
    assert_byte_identical_control_survives(&OCAML)
}

#[test]
fn every_cluster_ahead_of_the_ocaml_control_is_proven_duplication() -> Result<()> {
    assert_every_cluster_ahead_of_control_is_proven(&OCAML)
}

#[test]
fn each_ocaml_family_member_normalises_to_its_own_operator_leaves() -> Result<()> {
    assert_each_member_normalises_to_its_own_operator_leaves(&OCAML)
}
