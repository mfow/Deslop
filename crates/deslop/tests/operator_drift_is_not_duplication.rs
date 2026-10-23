//! [PIPELINE-NORMALIZE-AST] — an operator is behaviour, and two
//! functions that differ only in their operators are not duplicates.
//!
//! Normalisation walked `named_children` alone. In every grammar here
//! the operator of a binary expression is an *anonymous* token, so
//! `alpha + beta` and `alpha - beta` normalised to the same subtree with
//! the same identifier frontier and the same literals: nothing
//! downstream had any evidence they differed. The pair rendered
//! `structural = 1.00`, `token_jaccard = 1.00` and `pair_agreement =
//! 1.00` — the engine's strongest possible claim, made about
//! code that computes a different answer.
//!
//! That is a duplicate verdict, so a `find-similar`
//! consumer is told to reuse one where the other is meant, and an agent
//! following `docs/snippets/agents-md-recipe.md` deletes a subtraction
//! in favour of an addition. Sign errors, inverted comparisons and
//! inverted boolean guards are exactly the defects that survive review.
//!
//! Four families are pinned. Three cover the grammar productions that
//! carry the token — arithmetic (`binary_operator`), comparison
//! (`comparison_operator`) and boolean (`boolean_operator`) — and none
//! of them clusters once the operator reaches the digest. The fourth,
//! `ledger`, is a twelve-line body differing in exactly one `+`/`-`,
//! which is what a real sign error looks like. Its byte-identical regions
//! still cluster, but no published occurrence pair may cover the changed
//! operator line.
//!
//! The contract is *not* "never cluster them". Two functions that share
//! a shape and differ in an operator may well be worth a reader's
//! attention. The contract is that the report must not claim the
//! content is duplicated: the pair must stay out of the explicit
//! duplicate admission, and explicit pair evidence must remain honest.
//!
//! `control_alpha.py` / `control_beta.py` hold a byte-identical
//! function, judged in the same run. The shared contract lives in
//! `common::operator_drift`; `operator_drift_fsharp.rs` holds the F#
//! fixture judged by the same assertions.

use deslop_core::report::{ContentMeasurement, PairComparison, PairEndpoint};

use crate::common::{operator_drift::*, signals::*, *};

/// The operator families and one source line on which their computations
/// disagree.
const FAMILIES: [Family; 4] = [
    Family {
        label: "arithmetic + / -",
        left: "arithmetic_add.py",
        right: "arithmetic_sub.py",
        left_leaves: &["__op__+", "__op__+", "__op__+", "__op__+"],
        right_leaves: &["__op__-", "__op__-", "__op__-", "__op__-"],
        changed_line: 2,
    },
    Family {
        label: "comparison == / !=",
        left: "comparison_eq.py",
        right: "comparison_ne.py",
        left_leaves: &["__op__==", "__op__==", "__op__==", "__op__=="],
        right_leaves: &["__op__!=", "__op__!=", "__op__!=", "__op__!="],
        changed_line: 2,
    },
    Family {
        label: "boolean and / or",
        left: "boolean_and.py",
        right: "boolean_or.py",
        left_leaves: &["__op__and", "__op__and", "__op__and", "__op__and"],
        right_leaves: &["__op__or", "__op__or", "__op__or", "__op__or"],
        changed_line: 2,
    },
    LEDGER,
];

/// A twelve-line body differing in exactly one `+`/`-`.
const LEDGER: Family = Family {
    label: "ledger + / - inside a shared body",
    left: "ledger_credit.py",
    right: "ledger_debit.py",
    left_leaves: &["__op__*", "__op__+", "__op__+", "__op__/", "__op__-"],
    right_leaves: &["__op__*", "__op__-", "__op__+", "__op__/", "__op__-"],
    changed_line: 6,
};

/// The Python fixture: four operator pairs plus the control.
const PYTHON: DriftFixture = DriftFixture {
    dir: "operator-drift",
    min_nodes: 8,
    families: &FAMILIES,
    ledger: &LEDGER,
    control: ["control_alpha.py", "control_beta.py"],
    file_count: 10,
};

#[test]
fn an_operator_only_difference_never_claims_duplication() -> Result<()> {
    assert_operator_only_difference_never_claims_duplication(&PYTHON)
}

#[test]
fn the_byte_identical_control_survives_in_the_same_run() -> Result<()> {
    assert_byte_identical_control_survives(&PYTHON)
}

#[test]
fn every_cluster_ahead_of_the_control_is_proven_duplication() -> Result<()> {
    assert_every_cluster_ahead_of_control_is_proven(&PYTHON)
}

#[test]
fn each_family_member_normalises_to_its_own_operator_leaves() -> Result<()> {
    assert_each_member_normalises_to_its_own_operator_leaves(&PYTHON)
}

// [FUSED-CONTENT-GATE] Contradiction vetoes do not manufacture measured zero content.
const LEDGER_ENDPOINTS: [(&str, usize); 2] = [("ledger_credit.py", 458), ("ledger_debit.py", 457)];
const LEDGER_CONTENT_SHAPE_FLOOR: f64 = 0.9;

fn compare_operator_ledger() -> Result<PairComparison> {
    let scan_root = fixture(PYTHON.dir);
    let [left, right] = LEDGER_ENDPOINTS.map(|(file, end_byte)| PairEndpoint {
        path: scan_root.join(file),
        start_byte: 0,
        end_byte,
    });
    compare_endpoints(&scan_root, PYTHON.min_nodes, left, right)
}

#[test]
fn an_operator_veto_preserves_measured_content() -> Result<()> {
    let comparison = compare_operator_ledger()?;
    let evidence = &comparison.evidence;
    assert!(
        evidence.structural > LEDGER_CONTENT_SHAPE_FLOOR
            && evidence.agreement > LEDGER_CONTENT_SHAPE_FLOOR,
        "{comparison:#?}"
    );
    assert_eq!(evidence.content_measurement, ContentMeasurement::Measured);
    assert!(!evidence.content_ok, "{comparison:#?}");
    assert!(!evidence.admitted, "{comparison:#?}");
    assert_eq!(evidence.classification, None, "{comparison:#?}");
    let wire = serde_json::to_value(evidence)?;
    assert_eq!(
        field(&wire, "content_measurement").as_str(),
        Some("measured")
    );

    Ok(())
}
