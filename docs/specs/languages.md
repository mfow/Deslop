# Language contracts

The parser interface is [PIPELINE-LANG-TRAIT]; shared normalization follows [PIPELINE-NORMALIZE-AST]. These contracts preserve the shipped language behavior formerly documented in the language roadmap. Grammar versions remain defined in `Cargo.toml`.

## [LANG-CAND-DART] Dart

Files ending in `.dart` use the Dart grammar. Identifier leaves collapse to `__ident__`; numeric, boolean, null, symbol, and string literals collapse to `__literal__`; comments disappear. Type wrappers, interpolation expressions, records, and patterns retain their structure. Renamed declaration signatures are scaffolding, not proof of copied bodies.

Implemented by `crates/deslop-core/src/lang/dart.rs`; exercised by `crates/deslop/tests/dart_signatures.rs`, `dart_issue_119_embedding_role_mismatch.rs`, and the Dart golden in `cli/cache_and_debug.rs`.

## [LANG-CAND-GO] Go

Files ending in `.go` use the Go grammar. Identifier leaves, including package, type, field, blank, and label identifiers, collapse to `__ident__`. Constants and string contents collapse to `__literal__`; comments disappear. Composite literals and function literals retain their structure. The mandatory package clause is excluded from whole-file clone views under [PIPELINE-FINGERPRINT-MERKLE-ROOT].

Implemented by `crates/deslop-core/src/lang/go.rs`; exercised by the Go CLI detection and AST-golden cases in `crates/deslop/tests/cli/`, including closure-body and package-clause coverage.

## [LANG-CAND-JAVASCRIPT] JavaScript and JSX

`.js`, `.mjs`, `.cjs`, and `.jsx` files use the JavaScript grammar. JavaScript shares its normalization table with TypeScript: identifier leaves collapse, literals and textual JSX content collapse, and comments and hashbangs disappear. Qualified names and template substitutions retain their structure. JSX uses the JavaScript grammar rather than the TypeScript grammar.

Implemented by `lang/javascript.rs` and `lang/ecmascript.rs` in `deslop-core`; exercised by `js_language_features.rs`, `js_ts_extensions.rs`, `jsx_tsx_components.rs`, and the JavaScript/JSX AST goldens in `deslop`'s CLI suite.

## [LANG-CAND-TYPESCRIPT] TypeScript and TSX

`.ts` files use the TypeScript grammar; `.tsx` files use its TSX entry point. Both share the JavaScript normalization table. Identifier and literal spelling is erased while type annotations, qualified names, JSX structure, and interpolated expressions remain structural.

Implemented by `lang/typescript.rs` and `lang/ecmascript.rs` in `deslop-core`; exercised by `typescript_features.rs`, `jsx_tsx_components.rs`, `js_ts_signatures.rs`, and the TypeScript/TSX AST goldens in `deslop`'s CLI suite.

## [LANG-CAND-KOTLIN] Kotlin is not registered

Kotlin is not a supported parser in [PIPELINE-LANG-TRAIT]. A live `find_similar` request naming Kotlin must return `UnsupportedLanguage`, rather than an empty match list. `crates/deslop-core/tests/live.rs` pins that refusal; adding Kotlin requires a parser and the same fixture coverage as a shipped language.

## [PARSE-FSHARP-NORMALIZE] F# normalization

`.fs` and `.fsx` use the F# source grammar. Identifier and operator-identifier leaves collapse to `__ident__`; numeric, character, boolean, unit, and string literals collapse to `__literal__`. Line comments, block comments, and XML documentation disappear. The grammar spells every infix and prefix operator as a named `infix_op` / `prefix_op` node, so each becomes one operator leaf carrying its own token (`__op__+`, `__op__|>`) under [PIPELINE-NORMALIZE-AST-OPERATOR]; `alpha + beta` and `alpha - beta` never hash alike. Qualified names, interpolation expressions, active patterns, computation expressions, pipelines, units of measure, and quotations retain their structure.

Implemented by `crates/deslop-core/src/lang/fsharp.rs`; the F# AST golden in `crates/deslop/tests/cli/cache_and_debug.rs` pins normalized output; `lang/fsharp_tests.rs` and `crates/deslop/tests/operator_drift_fsharp.rs` pin the operator leaves.

## [PARSE-OCAML-NORMALIZE] OCaml normalization

`.ml` files use the OCaml implementation grammar. Interface files (`.mli`) are not analysed, just as F# signature files are not: they declare types and nothing else, so every pair would match on scaffolding alone. Every name collapses to `__ident__`: values, record fields, labels, modules, types and type variables, classes and methods, constructors, and polymorphic-variant tags. Numbers, booleans, characters, unit, every string form (including `{|quoted|}` strings) and printf conversions collapse to `__literal__`; comments and doc comments disappear. The grammar names each operator by its precedence class (`add_operator` covers both `+` and `-`; `rel_operator` covers `|>` and `>>=`), so each becomes one operator leaf carrying its own token under [PIPELINE-NORMALIZE-AST-OPERATOR], including `!`, `:=`, `-.` and binding operators such as `let*`. Paths such as `List.length`, pattern matches, records and modules keep their structure. Files that only share `open` lines are hidden by [CLONE-NOISE-ML-IMPORTS].

Implemented by `crates/deslop-core/src/lang/ocaml.rs`; unit-tested by `lang/ocaml_tests.rs`; exercised by the OCaml AST golden and the OCaml detection cases in `crates/deslop/tests/cli/`, `crates/deslop/tests/operator_drift_ocaml.rs`, `import_scaffolding.rs` and `ocaml_literal_bindings.rs`.

## [PARSE-PHP-NORMALIZE] PHP normalization

`.php` files use the PHP grammar. Name leaves collapse to `__ident__`, including the name inside a variable. Numeric, boolean, null, and string-family nodes collapse to `__literal__`; their children still undergo normalization. Comments and documentation comments disappear; remaining named nodes retain their structure.

Implemented by `crates/deslop-core/src/lang/php.rs`; the PHP AST golden in `crates/deslop/tests/cli/cache_and_debug.rs` pins normalized output.
