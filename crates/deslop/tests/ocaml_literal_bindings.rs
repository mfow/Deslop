//! [CLONE-NOISE-CONSTANT-TABLE] [ACCURACY-RECOVERY-PLAN-LITERAL-TABLE] —
//! OCaml tables of module-level `let NAME = <literal>` bindings are data,
//! not duplication, exactly as the F# rule holds them.
//!
//! Two unrelated constant modules share only the grammar of a binding, so
//! they must not cluster. A copied function beside them proves the run is
//! still detecting, and a copied table inside differently named submodules
//! proves the rule compares binding names and values, not shape alone.

use std::path::Path;

use anyhow::Result;
use serde_json::Value;

use crate::common::{corpora::*, signals::has_verbatim_pair, *};

/// The genuine clone: two byte-identical OCaml files.
const CLONE_FILES: [&str; 2] = ["parse_a.ml", "parse_b.ml"];
/// Distinct HTTP method names and World Bank payloads share only the
/// grammar of module-level bindings.
const UNRELATED_BINDING_FILES: [&str; 2] = ["http_methods.ml", "world_bank_mocks.ml"];
const BINDING_MIN_NODES: u32 = 20;
const BINDING_FILE_COUNT: u64 = 4;
const COPIED_OCCURRENCES: u64 = 2;
const NO_DUPLICATED_LINES: u64 = 0;
const HTTP_METHOD_BINDINGS: &str = r#"(** Retrieves properties of an HTTP resource. *)
let prop_find = "PROPFIND"
(** Updates the properties on an HTTP resource. *)
let prop_patch = "PROPPATCH"
(** Creates a collection resource. *)
let mk_col = "MKCOL"
(** Copies an HTTP resource. *)
let copy = "COPY"
(** Moves an HTTP resource. *)
let move = "MOVE"
(** Acquires an HTTP resource lock. *)
let lock = "LOCK"
(** Releases an HTTP resource lock. *)
let unlock = "UNLOCK"
(** Applies partial changes to a resource. *)
let patch = "PATCH"
"#;
const WORLD_BANK_MOCKS: &str = r#"let mock_indicator_response = {|
[
  { "page": 1, "pages": 1, "per_page": 1000, "total": 2 },
  [
    { "id": "AG.AGR.TRAC.NO", "name": "Agricultural machinery, tractors",
      "source": { "id": "2", "value": "World Development Indicators" },
      "topics": [{ "id": "9", "value": "Infrastructure" }] },
    { "id": "AG.CON.FERT.ZS", "name": "Fertilizer consumption",
      "source": { "id": "2", "value": "World Development Indicators" },
      "topics": [{ "id": "6", "value": "Environment" }] }
  ]
]
|}

let mock_country_response = {|
[
  { "page": 1, "pages": 1, "per_page": 1000, "total": 3 },
  [
    { "id": "ABW", "name": "Aruba", "capitalCity": "Oranjestad" },
    { "id": "AFG", "name": "Afghanistan", "capitalCity": "Kabul" }
  ]
]
|}

let mock_data_response = {|
[
  { "page": 1, "pages": 1, "per_page": 1000, "total": 3 },
  [
    { "date": "2020", "value": "65894.86" },
    { "date": "2019", "value": "62888.17" },
    { "date": "2018", "value": null }
  ]
]
|}
"#;

/// Wraps the shared HTTP method bindings in a named submodule.
fn http_methods_module(module_name: &str) -> String {
    format!("module {module_name} = struct\n{HTTP_METHOD_BINDINGS}end\n")
}

#[test]
fn unrelated_ocaml_literal_bindings_stay_out_of_report() -> Result<()> {
    let mut files = vec![
        (
            UNRELATED_BINDING_FILES[0].to_owned(),
            HTTP_METHOD_BINDINGS.to_owned(),
        ),
        (
            UNRELATED_BINDING_FILES[1].to_owned(),
            WORLD_BANK_MOCKS.to_owned(),
        ),
    ];
    files.extend(genuine_pair(
        CLONE_FILES[0],
        CLONE_FILES[1],
        OCAML_GENUINE_CLONE,
    ));
    let (_workspace, root, report) = report_for_with_root(&files, BINDING_MIN_NODES)?;
    assert_eq!(
        field(&report, "files_analysed").as_u64(),
        Some(BINDING_FILE_COUNT)
    );
    assert_copied_ocaml_function(&root, &report)?;
    assert_unrelated_bindings_absent(&report);
    Ok(())
}

/// A copied table is real duplication even when its enclosing submodule
/// name changes.
#[test]
fn copied_ocaml_literal_bindings_survive_module_rename() -> Result<()> {
    const COPY_FILES: [&str; 2] = ["methods_a.ml", "methods_b.ml"];
    const OTHER_FILE: &str = "world_bank_mocks.ml";
    const EXPECTED_FILES: u64 = 3;
    let files = [
        (
            COPY_FILES[0].to_owned(),
            http_methods_module("First_methods"),
        ),
        (
            COPY_FILES[1].to_owned(),
            http_methods_module("Second_methods"),
        ),
        (OTHER_FILE.to_owned(), WORLD_BANK_MOCKS.to_owned()),
    ];
    let (_workspace, _root, report) = report_for_with_root(&files, BINDING_MIN_NODES)?;
    assert_eq!(
        field(&report, "files_analysed").as_u64(),
        Some(EXPECTED_FILES)
    );
    let copy = expect_cluster_spanning(&report, &COPY_FILES)?;
    assert_eq!(cluster_size(copy), COPIED_OCCURRENCES, "{copy:#}");
    assert!(!cluster_file_set(copy).contains(OTHER_FILE), "{copy:#}");
    for path in COPY_FILES {
        assert!(duplicated_loc_for(&report, path) > 0, "{path}: {report:#}");
    }
    assert_eq!(duplicated_loc_for(&report, OTHER_FILE), NO_DUPLICATED_LINES);
    Ok(())
}

/// A genuine byte-identical function must remain visible and counted.
fn assert_copied_ocaml_function(root: &Path, report: &Value) -> Result<()> {
    let copy = expect_cluster_spanning(report, &CLONE_FILES)?;
    assert_eq!(
        cluster_size(copy),
        COPIED_OCCURRENCES,
        "both copies: {copy:#}"
    );
    assert!(has_verbatim_pair(root, copy)?, "copied function: {copy:#}");
    for path in CLONE_FILES {
        assert!(duplicated_loc_for(report, path) > 0, "{path}: {report:#}");
    }
    Ok(())
}

/// Neither unrelated table may surface in a visible cluster or LOC metric.
fn assert_unrelated_bindings_absent(report: &Value) {
    for cluster in clusters(report) {
        let paths = cluster_file_set(cluster);
        assert!(
            UNRELATED_BINDING_FILES
                .iter()
                .all(|path| !paths.contains(*path)),
            "unrelated literal bindings share no extractable code: {cluster:#}"
        );
    }
    for path in UNRELATED_BINDING_FILES {
        assert_eq!(
            duplicated_loc_for(report, path),
            NO_DUPLICATED_LINES,
            "{path}: {report:#}"
        );
    }
}
