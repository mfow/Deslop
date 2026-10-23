//! [CLONE-NOISE-ML-IMPORTS] ML-family import-dominated prologues are
//! scaffolding.
//!
//! Tree-sitter proves the matched range is imports and optional short,
//! structurally distinct implementation. A copied function remains visible.
//! The rule is one rule; only the grammar of a file's scaffolding is
//! per-language, described once per language by a [`ScaffoldGrammar`].

use tree_sitter::Node;

use super::{
    constant_table::{fsharp_is_container, fsharp_is_trivia, fsharp_module_name},
    language_cluster_shapes, node_intersects_range, parse_for, trimmed_snippet_range, Snippet,
};
use crate::{ast::named_children, ast::ByteRange};

/// How one ML-family grammar spells a file's scaffolding.
pub(super) struct ScaffoldGrammar {
    /// Language id the grammar parses under.
    language: &'static str,
    /// Kind of the parse root.
    root: &'static str,
    /// Containers whose children may be imports or one function.
    is_container: fn(&str) -> bool,
    /// Whether a container's child only names the container.
    is_header: fn(Node<'_>, Node<'_>) -> bool,
    /// Comments and documentation.
    is_trivia: fn(&str) -> bool,
    /// The import declaration (`open`).
    import: &'static str,
    /// The top-level definition that may carry one short function.
    definition: &'static str,
    /// Compiler directives and floating attributes.
    directives: &'static [&'static str],
    /// The expression body of a simple function definition.
    function_body: fn(Node<'_>) -> Option<Node<'_>>,
}

/// F#: `open` lists under a named module or namespace.
pub(super) const FSHARP: ScaffoldGrammar = ScaffoldGrammar {
    language: "fsharp",
    root: "file",
    is_container: fsharp_is_container,
    is_header: fsharp_module_name,
    is_trivia: fsharp_is_trivia,
    import: "import_decl",
    definition: "function_or_value_defn",
    directives: &["compiler_directive_decl", "fsi_directive_decl"],
    function_body: fsharp_function_body,
};

/// OCaml: `open` lists at file level or inside a `struct ... end` submodule.
pub(super) const OCAML: ScaffoldGrammar = ScaffoldGrammar {
    language: "ocaml",
    root: "compilation_unit",
    is_container: ocaml_is_container,
    is_header: ocaml_module_name,
    is_trivia: ocaml_is_trivia,
    import: "open_module",
    definition: "value_definition",
    directives: &[
        "floating_attribute",
        "toplevel_directive",
        "line_number_directive",
    ],
    function_body: ocaml_function_body,
};

/// A single `open` could be an incidental part of a meaningful match.
const MIN_IMPORT_DECLARATIONS: usize = 2;

/// What implementation shape, if any, accompanies the imports.
enum MemberShape {
    /// The matched range contains only file scaffolding.
    ImportsOnly,
    /// A short function accompanies the imports; this is its expression kind.
    Expression(u16),
}

/// Import lists repeated across ML modules have no extractable implementation.
pub(super) fn is_import_scaffolding_cluster(
    snippets: &[Snippet<'_>],
    grammar: &ScaffoldGrammar,
) -> bool {
    let Some(shapes) = language_cluster_shapes(snippets, grammar.language, |snippet| {
        member_shape(snippet, grammar)
    }) else {
        return false;
    };
    if shapes
        .iter()
        .all(|shape| matches!(shape, MemberShape::ImportsOnly))
    {
        return true;
    }
    let kinds: Option<Vec<_>> = shapes
        .iter()
        .map(|shape| match shape {
            MemberShape::ImportsOnly => None,
            MemberShape::Expression(kind) => Some(*kind),
        })
        .collect();
    kinds.is_some_and(|kinds| all_unique(&kinds))
}

/// Rejects a member with multiple or substantial implementation nodes.
fn member_shape(snippet: &Snippet<'_>, grammar: &ScaffoldGrammar) -> Option<MemberShape> {
    let tree = parse_for(snippet)?;
    let root = tree.root_node();
    (root.kind() == grammar.root).then_some(())?;
    let range = trimmed_snippet_range(snippet).unwrap_or(snippet.range);
    let (imports, function) = scan_nodes(root, range, grammar)?;
    covered_body_shape(function, imports, grammar)
}

/// The walker records only the syntax needed for this narrow proof.
fn scan_nodes<'tree>(
    root: Node<'tree>,
    range: ByteRange,
    grammar: &ScaffoldGrammar,
) -> Option<(usize, Option<Node<'tree>>)> {
    let mut walk = ScaffoldWalk {
        range,
        grammar,
        pending: vec![root],
        imports: 0,
        function: None,
    };
    while let Some(node) = walk.pending.pop() {
        walk.inspect(node)?;
    }
    let ScaffoldWalk {
        imports, function, ..
    } = walk;
    (imports >= MIN_IMPORT_DECLARATIONS).then_some(())?;
    Some((imports, function))
}

/// A short, fully covered expression can be compared at its root kind.
fn covered_body_shape(
    function: Option<Node<'_>>,
    imports: usize,
    grammar: &ScaffoldGrammar,
) -> Option<MemberShape> {
    let Some(function) = function else {
        return Some(MemberShape::ImportsOnly);
    };
    let body = (grammar.function_body)(function)?;
    body_shorter_than_import_run(body, imports).then_some(MemberShape::Expression(body.kind_id()))
}

/// The state of one member's scaffolding proof.
struct ScaffoldWalk<'tree, 'grammar> {
    /// The matched range being proven.
    range: ByteRange,
    /// The grammar the member is written in.
    grammar: &'grammar ScaffoldGrammar,
    /// Containers and declarations still to inspect.
    pending: Vec<Node<'tree>>,
    /// Import declarations seen so far.
    imports: usize,
    /// The one fully matched definition, if any.
    function: Option<Node<'tree>>,
}

impl<'tree> ScaffoldWalk<'tree, '_> {
    /// Module containers can contribute imports and at most one full function.
    fn inspect(&mut self, node: Node<'tree>) -> Option<()> {
        if !node_intersects_range(node, self.range) {
            return Some(());
        }
        let kind = node.kind();
        if (self.grammar.is_container)(kind) {
            self.queue_children(node);
        } else if kind == self.grammar.import {
            self.imports = self.imports.checked_add(1)?;
        } else if kind == self.grammar.definition {
            self.record_function(node)?;
        } else if !(self.grammar.is_trivia)(kind) && !self.grammar.directives.contains(&kind) {
            return None;
        }
        Some(())
    }

    /// Module names are headers, not implementation children.
    fn queue_children(&mut self, node: Node<'tree>) {
        let (range, is_header) = (self.range, self.grammar.is_header);
        self.pending.extend(
            named_children(node)
                .into_iter()
                .filter(|child| node_intersects_range(*child, range) && !is_header(node, *child)),
        );
    }

    /// A second or partially matched function invalidates the shape proof.
    fn record_function(&mut self, node: Node<'tree>) -> Option<()> {
        if self.function.is_some()
            || node.start_byte() < self.range.start
            || node.end_byte() > self.range.end
        {
            return None;
        }
        self.function = Some(node);
        Some(())
    }
}

/// Only a simple F# definition with one expression body qualifies.
fn fsharp_function_body(function: Node<'_>) -> Option<Node<'_>> {
    let children = named_children(function);
    match children.as_slice() {
        [left, body] if left.kind() == "function_declaration_left" => Some(*body),
        _ => None,
    }
}

/// More import statements than implementation lines prove the file match is scaffold-dominated.
fn body_shorter_than_import_run(body: Node<'_>, imports: usize) -> bool {
    body.end_position()
        .row
        .checked_sub(body.start_position().row)
        .and_then(|lines| lines.checked_add(1))
        .is_some_and(|lines| lines < imports)
}

/// A shared expression shape may be a copied implementation, so keep it.
fn all_unique(kinds: &[u16]) -> bool {
    kinds
        .iter()
        .enumerate()
        .all(|(index, kind)| kinds.iter().take(index).all(|previous| previous != kind))
}

/// Only a single OCaml `let` binding that takes parameters is a function;
/// its `body` field is the expression compared.
fn ocaml_function_body(function: Node<'_>) -> Option<Node<'_>> {
    let children = named_children(function);
    let [binding] = children.as_slice() else {
        return None;
    };
    (binding.kind() == "let_binding").then_some(())?;
    named_children(*binding)
        .iter()
        .any(|child| child.kind() == "parameter")
        .then_some(())?;
    binding.child_by_field_name("body")
}

/// OCaml containers: the file, and a `module Name = struct ... end`.
fn ocaml_is_container(kind: &str) -> bool {
    matches!(
        kind,
        "compilation_unit" | "module_definition" | "module_binding" | "structure"
    )
}

/// A submodule's name labels its container; it is not one of its entries.
fn ocaml_module_name(parent: Node<'_>, child: Node<'_>) -> bool {
    parent.kind() == "module_binding" && child.kind() == "module_name"
}

/// OCaml block and documentation comments.
fn ocaml_is_trivia(kind: &str) -> bool {
    kind == "comment"
}
