//! [CLONE-NOISE-ML-IMPORTS] Import-dominated scaffolding is not a clone,
//! while a copied implementation remains a reported finding — judged
//! identically for F# and OCaml, whose files open the same libraries the
//! same way.

use anyhow::Result;
use serde_json::Value;

use crate::common::*;

/// One language's import-scaffolding fixture and the copy it must keep.
struct ScaffoldFixture {
    /// Fixture directory under `tests/fixtures`.
    dir: &'static str,
    /// Every source file in the fixture.
    files_analysed: u64,
    /// The two files holding a copied function.
    copy_files: [&'static str; 2],
    /// The copy's clone kind.
    copy_kind: &'static str,
    /// The copy's mass.
    copy_mass: u64,
    /// The two provider files whose different implementations sit under
    /// near-identical import lists.
    provider_files: [&'static str; 2],
    /// Each scaffold file and the last line of its import prologue.
    scaffold_last_lines: [(&'static str, u64); 4],
}

const MIN_NODES: u32 = 20;
/// Only the copied function survives.
const REPORTED_CLUSTERS: usize = 1;
const COPY_OCCURRENCES: u64 = 2;
const COPY_RANK: u64 = 1;

/// The copy's `module` headers differ, so it is a near-identical copy.
const FSHARP: ScaffoldFixture = ScaffoldFixture {
    dir: "fsharp-import-scaffolding",
    files_analysed: 6,
    copy_files: ["copy_a.fs", "copy_b.fs"],
    copy_kind: "nearly_identical",
    copy_mass: 81,
    provider_files: ["JsonProvider.fs", "XmlProvider.fs"],
    scaffold_last_lines: [
        ("JsonProvider.fs", 16),
        ("XmlProvider.fs", 17),
        ("InferenceTests.fs", 10),
        ("HtmlRuntimeTypes.fs", 10),
    ],
};

/// An OCaml file is its own module, so the two copies are byte-identical.
const OCAML: ScaffoldFixture = ScaffoldFixture {
    dir: "ocaml-import-scaffolding",
    files_analysed: 6,
    copy_files: ["copy_a.ml", "copy_b.ml"],
    copy_kind: "identical",
    copy_mass: 76,
    provider_files: ["json_provider.ml", "xml_provider.ml"],
    scaffold_last_lines: [
        ("json_provider.ml", 13),
        ("xml_provider.ml", 15),
        ("inference_tests.ml", 7),
        ("html_runtime_types.ml", 7),
    ],
};

#[test]
fn fsharp_import_lists_are_not_clones_of_implementation() -> Result<()> {
    assert_import_lists_are_not_clones(&FSHARP)
}

#[test]
fn ocaml_import_lists_are_not_clones_of_implementation() -> Result<()> {
    assert_import_lists_are_not_clones(&OCAML)
}

fn assert_import_lists_are_not_clones(scaffold: &ScaffoldFixture) -> Result<()> {
    let report = run_report(&fixture(scaffold.dir), MIN_NODES)?;
    assert_eq!(
        field(&report, "files_analysed").as_u64(),
        Some(scaffold.files_analysed)
    );
    assert_eq!(clusters(&report).len(), REPORTED_CLUSTERS, "{report:#}");

    for cluster in clusters(&report) {
        assert!(
            !import_only_cluster(scaffold, cluster),
            "import-only prologues have no duplicated implementation: {cluster:#}"
        );
        assert!(
            !provider_only_cluster(scaffold, cluster),
            "different provider implementations are not a clone: {cluster:#}"
        );
    }
    assert_copy(scaffold, &report)
}

fn assert_copy(scaffold: &ScaffoldFixture, report: &Value) -> Result<()> {
    let copy = expect_cluster_spanning(report, &scaffold.copy_files)?;
    assert_eq!(
        field(copy, "occurrence_count").as_u64(),
        Some(COPY_OCCURRENCES)
    );
    assert_eq!(field(copy, "kind").as_str(), Some(scaffold.copy_kind));
    assert_eq!(field(copy, "rank").as_u64(), Some(COPY_RANK));
    assert_eq!(field(copy, "mass").as_u64(), Some(scaffold.copy_mass));
    Ok(())
}

fn provider_only_cluster(scaffold: &ScaffoldFixture, cluster: &Value) -> bool {
    let members = occurrences(cluster);
    members.len() == scaffold.provider_files.len()
        && scaffold.provider_files.iter().all(|file| {
            members
                .iter()
                .any(|member| occurrence_path(member).ok() == Some(*file))
        })
}

/// An AST-delimited import run cannot gain implementation from a later line.
fn import_only_cluster(scaffold: &ScaffoldFixture, cluster: &Value) -> bool {
    let members = occurrences(cluster);
    members.len() >= 2
        && members
            .iter()
            .all(|member| import_only_member(scaffold, member))
}

fn import_only_member(scaffold: &ScaffoldFixture, member: &Value) -> bool {
    occurrence_path(member).is_ok_and(|path| {
        scaffold
            .scaffold_last_lines
            .iter()
            .find(|(name, _)| *name == path)
            .is_some_and(|(_, last)| occurrence_line_span(member).1 <= *last)
    })
}
