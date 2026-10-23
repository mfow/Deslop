//! [PIPELINE-NORMALIZE-AST-OPERATOR] — the operator-drift contract every
//! language's fixture is judged by.
//!
//! Two functions that differ only in their operators are not duplicates.
//! The contract is *not* "never cluster them": two functions that share a
//! shape and differ in an operator may well be worth a reader's attention.
//! The contract is that the report must not claim the content is
//! duplicated — no published cluster may pair occurrences covering both
//! sides of a changed operator.
//!
//! # Why a fixture cannot pass by going blind
//!
//! Every fixture carries a byte-identical control pair. Any normalisation
//! change wide enough to stop the operator families clustering also has to
//! leave that pair published as verbatim duplication, asserted in the same
//! run.

use serde_json::Value;

use super::{signals::*, *};

/// The namespace every operator leaf is emitted behind.
const OPERATOR_KIND_PREFIX: &str = "__op__";

/// One operator family: the pair, the operator leaves each member must
/// normalise to, and a source line carrying the changed computation.
pub(crate) struct Family {
    /// Human label used in every assertion message.
    pub(crate) label: &'static str,
    /// The member holding the first operator.
    pub(crate) left: &'static str,
    /// The member holding the second.
    pub(crate) right: &'static str,
    /// Every operator leaf `left` normalises to, in dump order.
    pub(crate) left_leaves: &'static [&'static str],
    /// Every operator leaf `right` normalises to, in dump order.
    pub(crate) right_leaves: &'static [&'static str],
    /// A line whose operator differs between the two members.
    pub(crate) changed_line: u64,
}

/// One language's operator-drift fixture.
pub(crate) struct DriftFixture {
    /// Fixture directory under `tests/fixtures`.
    pub(crate) dir: &'static str,
    /// Node floor low enough that each function body is a candidate window.
    pub(crate) min_nodes: u32,
    /// The operator families the fixture poses.
    pub(crate) families: &'static [Family],
    /// The family whose members share a long body around one changed
    /// operator — the shape a real sign error takes.
    pub(crate) ledger: &'static Family,
    /// The byte-identical pair that must survive whatever separates the
    /// families.
    pub(crate) control: [&'static str; 2],
    /// Every file the fixture holds, asserted per run so "the family
    /// published nothing" can only ever mean the pair was analysed and
    /// excluded.
    pub(crate) file_count: u64,
}

/// Renders the fixture.
pub(crate) fn render(drift: &DriftFixture) -> Result<Value> {
    run_report(&fixture(drift.dir), drift.min_nodes)
}

fn occurrence_covers(occurrence: &Value, file: &str, line: u64) -> bool {
    let path_matches = field(occurrence, "path")
        .as_str()
        .and_then(|path| std::path::Path::new(path).file_name())
        .is_some_and(|name| name == file);
    let start = field(occurrence, "start_line").as_u64().unwrap_or(0);
    let end = field(occurrence, "end_line").as_u64().unwrap_or(0);
    path_matches && start <= line && line <= end
}

/// Whether a published cluster pairs both sides of a family's changed line.
/// A file-pair lookup is insufficient: the ledger files also contain
/// genuine byte-identical subregions, and mistaking one of those for the
/// operator-bearing enclosure condemned correct duplication.
fn covers_operator_pair(cluster: &Value, family: &Family) -> bool {
    [family.left, family.right].iter().all(|file| {
        occurrences(cluster)
            .iter()
            .any(|occurrence| occurrence_covers(occurrence, file, family.changed_line))
    })
}

/// An operator-only difference is never published as duplicated content.
///
/// Every family is judged in the same run as the byte-identical control.
/// Absence is a legitimate disposition for these pairs — they compute
/// different answers — but absence alone proves nothing: a detector that
/// had stopped producing candidates, or a normalisation change that
/// dropped the whole fixture, would show the same empty result. The
/// control is what separates "considered and excluded" from "never
/// looked".
pub(crate) fn assert_operator_only_difference_never_claims_duplication(
    drift: &DriftFixture,
) -> Result<()> {
    let report = render(drift)?;
    assert_eq!(
        field(&report, "files_analysed").as_u64(),
        Some(drift.file_count),
        "{dir}: every operator pair must reach the pipeline before its absence \
         from the report can mean anything: {published:#?}",
        dir = drift.dir,
        published = published_with_mass(&report),
    );
    let control = expect_cluster_spanning(&report, &drift.control)?;
    assert!(
        has_verbatim_pair(&fixture(drift.dir), control)?,
        "{dir}: the byte-identical control must still be published as \
         duplication in this very run — without it, every assertion below is \
         satisfied just as well by a detector that produced no candidates: \
         {published:#?}",
        dir = drift.dir,
        published = published_with_mass(&report),
    );
    assert_structural_only_contract(control, drift.dir);
    for family in drift.families {
        let offenders: Vec<&Value> = clusters(&report)
            .iter()
            .filter(|cluster| covers_operator_pair(cluster, family))
            .collect();
        assert!(
            offenders.is_empty(),
            "{label}: {left} and {right} compute different answers on line \
             {line}. No duplicate cluster may pair occurrences covering both \
             changed operators: {offenders:#?}\nall clusters: {published:#?}",
            label = family.label,
            left = family.left,
            right = family.right,
            line = family.changed_line,
            published = published_with_mass(&report),
        );
    }
    Ok(())
}

/// The false-negative control, in the same run: a real byte-identical
/// clone still ranks and shows both copies.
pub(crate) fn assert_byte_identical_control_survives(drift: &DriftFixture) -> Result<()> {
    let report = render(drift)?;
    let control = expect_cluster_spanning(&report, &drift.control)?;
    let dump = signal_dump(control);
    assert!(
        has_verbatim_pair(&fixture(drift.dir), control)?,
        "the control is copied byte for byte — {dump}"
    );
    assert_eq!(
        cluster_size(control),
        2,
        "both copies of the control must be shown — {dump}"
    );
    Ok(())
}

/// A genuine exact subregion may outrank the small control. Every cluster
/// ahead of it must be byte-identical and must exclude the ledger's changed
/// operator line; otherwise ranking would still elevate the false positive.
pub(crate) fn assert_every_cluster_ahead_of_control_is_proven(drift: &DriftFixture) -> Result<()> {
    let report = render(drift)?;
    let control = expect_cluster_spanning(&report, &drift.control)?;
    let control_rank = rank_of(&report, control)?;
    let scan_root = fixture(drift.dir);
    for cluster in clusters(&report).iter().take(control_rank) {
        assert_eq!(
            distinct_texts(&scan_root, cluster)?.len(),
            1,
            "a cluster may outrank the control only with byte-identical \
             source proof: {}",
            signal_dump(cluster),
        );
        assert!(
            !covers_operator_pair(cluster, drift.ledger),
            "the changed ledger operator must never enter a higher-ranked \
             duplicate cluster: {}",
            signal_dump(cluster),
        );
    }
    Ok(())
}

/// The `--debug-ast` dump for one fixture file, as the exact sequence of
/// normalised node kinds it names. The dump prints one node per line as
/// `<indent><kind> [start..end]`, so the kind is everything between the
/// indent and the span. Reading it this way keeps the assertions exact: a
/// `contains` test for `"__op__+"` is also satisfied by `__op__++` and
/// `__op__+=`.
fn debug_ast_kinds(drift: &DriftFixture, file_name: &str) -> Result<Vec<String>> {
    let output = assert_cmd::Command::cargo_bin("deslop")?
        .arg("--debug-ast")
        .arg(fixture(drift.dir).join(file_name))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    Ok(String::from_utf8(output)?
        .lines()
        .filter_map(|line| line.trim_start().split_once(" ["))
        .map(|(kind, _span)| kind.to_owned())
        .collect())
}

/// Every operator leaf one file normalises to, in dump order.
fn operator_leaves(drift: &DriftFixture, file_name: &str) -> Result<Vec<String>> {
    Ok(debug_ast_kinds(drift, file_name)?
        .into_iter()
        .filter(|kind| kind.starts_with(OPERATOR_KIND_PREFIX))
        .collect())
}

/// The operator-specific non-vacuity proof, on the pairs themselves.
///
/// The disposition pin says what each family reaches the report as, and
/// the control says the run was still detecting — but neither shows that
/// these files exercised *operator* normalisation. Both would hold just as
/// well if operators had stopped reaching the digest altogether, which is
/// the original defect. This reads each member's normalised tree and pins
/// its operator leaves *exactly*, in dump order, then requires the two
/// sides to differ: identical leaf lists would mean the fixture had stopped
/// posing the question.
pub(crate) fn assert_each_member_normalises_to_its_own_operator_leaves(
    drift: &DriftFixture,
) -> Result<()> {
    for family in drift.families {
        for (file, expected) in [
            (family.left, family.left_leaves),
            (family.right, family.right_leaves),
        ] {
            assert_eq!(
                operator_leaves(drift, file)?,
                expected,
                "{label}: {file} must normalise to exactly these operator \
                 leaves under {OPERATOR_KIND_PREFIX}, in this order. Fewer \
                 means an operator stopped reaching the digest and this file \
                 now hashes closer to its sibling; a different spelling means \
                 normalisation is telling the reader about a token nobody \
                 wrote.",
                label = family.label,
            );
        }
        assert_ne!(
            family.left_leaves,
            family.right_leaves,
            "{label}: the two members carry the same operator leaves, so \
             the pair no longer differs in an operator and the family \
             proves nothing about operator drift",
            label = family.label,
        );
    }
    Ok(())
}
