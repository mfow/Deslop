//! OCaml language plugin.
//!
//! Implements [PIPELINE-LANG-TRAIT] for OCaml using the `tree-sitter-ocaml`
//! implementation grammar (`LANGUAGE_OCAML`, covering `.ml` files).
//! Interface files (`.mli`) are not analysed, matching the F# plugin's
//! treatment of signature files: they declare types and nothing else, so
//! every pair of them would match on scaffolding alone. Normalisation
//! follows the Type-2-invariance principle ([CLONE-TYPE-TAXONOMY]) and the
//! same identifier / literal / trivia collapse the other languages use
//! ([PARSE-OCAML-NORMALIZE]):
//!
//! - Every name leaf collapses to `"__ident__"` so renamed clones
//!   fingerprint identically: values and their binding patterns, record
//!   fields, labels, modules and module types, type constructors and type
//!   variables, classes, methods, instance variables, constructors,
//!   polymorphic-variant tags, kinds, and attribute ids. Paths
//!   (`value_path`, `module_path`, `constructor_path`) stay structural
//!   wrappers over those leaves, so `List.length` keeps a shape distinct
//!   from a bare name while each segment collapses — parity with the F#
//!   `long_identifier` handling in [`super::fsharp`].
//! - Every constant leaf collapses to `"__literal__"`: `number` and
//!   `signed_number`, `boolean`, `unit` (`()`), `character` and its
//!   content, and every string form — `string`, `quoted_string`
//!   (`{|...|}`), their contents, `escape_sequence`, and the printf
//!   `conversion_specification` / `pretty_printing_indication` inside a
//!   format string — so constant edits do not perturb the fingerprint.
//! - `comment` (block and doc comments alike) is dropped as trivia.
//! - The grammar names each operator by its precedence class
//!   (`add_operator` for both `+` and `-`), which the shared walk turns
//!   into operator leaves carrying their own token
//!   ([PIPELINE-NORMALIZE-AST-OPERATOR]).
//! - All other named node kinds pass through unchanged.
//!
//! Shared walking / interning plumbing lives in [`super::shared`].

use crate::{
    ast::NormalizedNode,
    error::CoreError,
    lang::{
        shared::{
            build_normalised_root_with_named_operators, normalise_kind_with, parse_source,
            NamedOperators,
        },
        LanguageParser,
    },
    state::FileId,
};

/// Stable language identifier reported by [`OCamlParser::id`].
const LANGUAGE_ID: &str = "ocaml";

/// OCaml implementation of [`LanguageParser`].
#[derive(Debug, Default)]
pub struct OCamlParser;

impl OCamlParser {
    /// Creates a new parser. Stateless — safe to share across threads.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl LanguageParser for OCamlParser {
    fn id(&self) -> &'static str {
        LANGUAGE_ID
    }

    fn file_extensions(&self) -> &'static [&'static str] {
        &["ml"]
    }

    fn grammar(&self) -> tree_sitter::Language {
        tree_sitter_ocaml::LANGUAGE_OCAML.into()
    }

    fn is_comment_kind(&self, kind: &str) -> bool {
        is_comment_kind(kind)
    }

    fn parse_and_normalize(
        &self,
        source: &[u8],
        file_id: FileId,
    ) -> Result<NormalizedNode, CoreError> {
        let tree = parse_source(LANGUAGE_ID, &self.grammar(), source)?;
        let named_operators = NamedOperators {
            source,
            kinds: NAMED_OPERATOR_KINDS,
        };
        build_normalised_root_with_named_operators(
            &tree,
            named_operators,
            file_id,
            normalise_kind,
            LANGUAGE_ID,
        )
    }
}

/// OCaml operator nodes, named by precedence class rather than token:
/// `add_operator` covers `+`, `-`, `+.` and `-.`; `rel_operator` covers
/// every comparison and user operator such as `|>` and `>>=`. The prefix
/// (`!`), sign (`-.`) and binding (`let*`, `and+`, `match*`) operators are
/// named nodes too ([PIPELINE-NORMALIZE-AST-OPERATOR]).
const NAMED_OPERATOR_KINDS: &[&str] = &[
    "add_operator",
    "mult_operator",
    "pow_operator",
    "rel_operator",
    "concat_operator",
    "and_operator",
    "or_operator",
    "assign_operator",
    "prefix_operator",
    "sign_operator",
    "hash_operator",
    "indexing_operator",
    "let_operator",
    "let_and_operator",
    "match_operator",
];

/// Maps a tree-sitter OCaml node kind to its normalised form. Covers the
/// identifier / literal / trivia families emitted by `tree-sitter-ocaml`
/// 0.26.x ([PARSE-OCAML-NORMALIZE]).
fn normalise_kind(raw: &str) -> Option<&'static str> {
    normalise_kind_with(raw, is_comment_kind, is_identifier_kind, is_literal_kind)
}

/// OCaml trivia.
fn is_comment_kind(raw: &str) -> bool {
    matches!(raw, "comment")
}

/// OCaml name leaves, collapsed for Type-2 renamed-clone detection.
fn is_identifier_kind(raw: &str) -> bool {
    matches!(
        raw,
        "value_name"
            | "value_pattern"
            | "field_name"
            | "label_name"
            | "module_name"
            | "module_type_name"
            | "type_constructor"
            | "type_variable"
            | "class_name"
            | "class_type_name"
            | "method_name"
            | "instance_variable_name"
            | "constructor_name"
            | "tag"
            | "kind_name"
            | "attribute_id"
    )
}

/// OCaml literal leaves, collapsed so constant edits do not perturb
/// fingerprints.
fn is_literal_kind(raw: &str) -> bool {
    matches!(
        raw,
        "number"
            | "signed_number"
            | "boolean"
            | "unit"
            | "character"
            | "character_content"
            | "string"
            | "string_content"
            | "quoted_string"
            | "quoted_string_content"
            | "escape_sequence"
            | "conversion_specification"
            | "pretty_printing_indication"
    )
}

#[cfg(test)]
#[path = "ocaml_tests.rs"]
mod tests;
