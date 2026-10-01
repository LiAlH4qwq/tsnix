//! The `std` standard library: a pure-Nix attribute set registered as
//! `builtins.std` and injected as a bare `std` binding.
//!
//! The implementation lives in `stdlib.nix`; this module embeds it and carries
//! the machine-readable catalogue used by `tsnix libdoc` and `tsnix schema`.
//! A unit test parses `stdlib.nix` and asserts that its exported attribute
//! names match [`FUNCTIONS`] plus [`PRELUDE`], so the implementation and its
//! documentation cannot drift apart.

/// The Nix source of the standard library.
pub(crate) const STDLIB_SRC: &str = include_str!("stdlib.nix");

/// Metadata for one `std` function.
pub(crate) struct Function {
    /// The attribute name inside `std`.
    pub name: &'static str,
    /// A Nix-style signature.
    pub signature: &'static str,
    /// One-line description.
    pub about: &'static str,
    /// Grouping used by `libdoc`.
    pub group: &'static str,
    /// Closest nixpkgs `lib` name, or `""` when there is none.
    pub nixpkgs: &'static str,
}

/// Builtins re-exported under `std` so that `with std;` works as a prelude.
#[rustfmt::skip]
pub(crate) const PRELUDE: &[&str] = &[
    "map", "filter", "foldl'", "concatMap", "sort", "elem", "length", "head", "tail", "attrNames",
    "attrValues",
];

/// The native `std` functions, in catalogue order.
#[rustfmt::skip]
pub(crate) const FUNCTIONS: &[Function] = &[
    // utilities
    f("id", "x -> x", "Return the argument unchanged.", "utilities", "lib.id"),
    f("const", "x -> y -> x", "Return the first argument, ignoring the second.", "utilities", "lib.const"),
    f("flip", "(a -> b -> c) -> b -> a -> c", "Flip the first two arguments of a function.", "utilities", "lib.flip"),
    f("pipe", "value -> [ (a -> a) ] -> a", "Apply a list of functions left to right.", "utilities", "lib.pipe"),
    // conditional
    f("optional", "bool -> x -> [x]", "A singleton list when the condition holds, otherwise `[]`.", "conditional", "lib.optional"),
    f("optionals", "bool -> [x] -> [x]", "The list when the condition holds, otherwise `[]`.", "conditional", "lib.optionals"),
    f("optionalAttrs", "bool -> {..} -> {..}", "The attribute set when the condition holds, otherwise `{}`.", "conditional", "lib.optionalAttrs"),
    f("optionalString", "bool -> string -> string", "The string when the condition holds, otherwise `\"\"`.", "conditional", "lib.optionalString"),
    // merging
    f("merge", "{..} -> {..} -> {..}", "Deep merge, right side wins; non-attribute sets are replaced.", "merging", "lib.recursiveUpdate"),
    f("mergeWith", "(x -> y -> z) -> {..} -> {..} -> {..}", "Deep merge, resolving leaf conflicts with a function.", "merging", ""),
    f("defaults", "{..} -> {..} -> {..}", "`merge base overrides`: fill in defaults.", "merging", ""),
    // paths
    f("getIn", "path -> attrs -> value or null", "Deep lookup; `null` when absent. `path` may be a list or `\"a.b\"`.", "paths", "lib.attrByPath"),
    f("getInOr", "default -> path -> attrs -> value", "Deep lookup with a default (`null` counts as absent).", "paths", "lib.attrByPath"),
    f("hasIn", "path -> attrs -> bool", "Whether a deep path exists.", "paths", "lib.hasAttrByPath"),
    f("assocIn", "path -> value -> attrs -> attrs", "Immutable deep update, creating intermediate sets.", "paths", "lib.setAttrByPath"),
    f("updateIn", "path -> (value -> value) -> attrs -> attrs", "Apply a function at a deep path.", "paths", ""),
    f("removeIn", "path -> attrs -> attrs", "Deep delete; a missing path is a no-op.", "paths", ""),
    f("selectKeys", "[ path ] -> attrs -> attrs", "Project a whitelist of paths, preserving structure.", "paths", ""),
    f("omitKeys", "[ path ] -> attrs -> attrs", "Drop a blacklist of paths.", "paths", ""),
    f("renameKey", "from -> to -> attrs -> attrs", "Rename a top-level key.", "paths", ""),
    // attribute sets
    f("nameValuePair", "name -> value -> { name; value; }", "Build a `{ name; value; }` pair.", "attrs", "lib.nameValuePair"),
    f("mapEntries", "(name -> value -> { name; value; }) -> attrs -> attrs", "Map keys and values at once (later duplicates win).", "attrs", "lib.mapAttrs'"),
    f("mapAttrsToList", "(name -> value -> x) -> attrs -> [x]", "Map each entry to a list element.", "attrs", "lib.mapAttrsToList"),
    f("filterAttrs", "(name -> value -> bool) -> attrs -> attrs", "Keep entries for which the predicate holds.", "attrs", "lib.filterAttrs"),
    f("rejectAttrs", "(name -> value -> bool) -> attrs -> attrs", "Drop entries for which the predicate holds.", "attrs", ""),
    f("foldlAttrs", "(acc -> name -> value -> acc) -> acc -> attrs -> acc", "Left fold over attribute entries.", "attrs", "lib.foldlAttrs"),
    f("mapKeys", "(name -> name') -> attrs -> attrs", "Transform keys, keeping values.", "attrs", ""),
    f("compactAttrs", "attrs -> attrs", "Drop entries whose value is `null`.", "attrs", ""),
    // lists
    f("flatten", "[[..]] -> [..]", "Deeply flatten nested lists.", "lists", "lib.flatten"),
    f("flatMap", "(x -> [y]) -> [x] -> [y]", "Map then concatenate (alias of `builtins.concatMap`).", "lists", "lib.concatMap"),
    f("compact", "[x] -> [x]", "Drop `null` elements.", "lists", ""),
    f("unique", "[x] -> [x]", "Remove duplicates, keeping first occurrences.", "lists", "lib.unique"),
    f("uniqueBy", "(x -> k) -> [x] -> [x]", "Remove duplicates by a derived key.", "lists", ""),
    f("reverse", "[x] -> [x]", "Reverse a list.", "lists", "lib.reverseList"),
    f("range", "int -> int -> [int]", "Inclusive integer range (ascending or descending).", "lists", "lib.range"),
    f("repeat", "int -> x -> [x]", "A list containing `x` repeated `n` times.", "lists", "lib.replicate"),
    f("count", "(x -> bool) -> [x] -> int", "Count elements satisfying a predicate.", "lists", "lib.count"),
    f("find", "(x -> bool) -> [x] -> x or null", "First matching element, or `null`.", "lists", "lib.findFirst"),
    f("findOr", "default -> (x -> bool) -> [x] -> x", "First matching element, or a default.", "lists", "lib.findFirst"),
    f("last", "[x] -> x", "The last element (throws on `[]`).", "lists", "lib.last"),
    f("zip", "[a] -> [b] -> [{ fst; snd; }]", "Pair up two lists, truncating to the shorter.", "lists", "lib.zipLists"),
    f("zipWith", "(a -> b -> c) -> [a] -> [b] -> [c]", "Combine two lists element-wise.", "lists", "lib.zipListsWith"),
    f("enumerate", "[x] -> [{ index; value; }]", "Pair each element with its index.", "lists", ""),
    f("mapWithIndex", "(int -> x -> y) -> [x] -> [y]", "Map with the element index.", "lists", "lib.imap0"),
    f("sum", "[number] -> number", "Sum a list of numbers.", "lists", "lib.sum"),
    f("min", "[number] -> number", "Smallest element (throws on `[]`).", "lists", "lib.minimum"),
    f("max", "[number] -> number", "Largest element (throws on `[]`).", "lists", "lib.maximum"),
    f("sortBy", "(x -> k) -> [x] -> [x]", "Sort by a derived key.", "lists", "lib.sortOn"),
    f("take", "int -> [x] -> [x]", "The first `n` elements.", "lists", "lib.take"),
    f("drop", "int -> [x] -> [x]", "All but the first `n` elements.", "lists", "lib.drop"),
    f("slice", "int -> int -> [x] -> [x]", "`len` elements starting at `start`.", "lists", "lib.sublist"),
    f("intersperse", "x -> [x] -> [x]", "Insert a separator between elements.", "lists", "lib.intersperse"),
    f("chunk", "int -> [x] -> [[x]]", "Split into consecutive chunks of size `n`.", "lists", "lib.chunk"),
    // strings
    f("hasPrefix", "string -> string -> bool", "Whether a string starts with a prefix.", "strings", "lib.hasPrefix"),
    f("hasSuffix", "string -> string -> bool", "Whether a string ends with a suffix.", "strings", "lib.hasSuffix"),
    f("removePrefix", "string -> string -> string", "Remove a leading prefix when present.", "strings", "lib.removePrefix"),
    f("removeSuffix", "string -> string -> string", "Remove a trailing suffix when present.", "strings", "lib.removeSuffix"),
    f("trim", "string -> string", "Strip leading and trailing whitespace.", "strings", "lib.trim"),
    f("trimStart", "string -> string", "Strip leading whitespace.", "strings", ""),
    f("trimEnd", "string -> string", "Strip trailing whitespace.", "strings", ""),
    f("toUpper", "string -> string", "ASCII uppercase.", "strings", "lib.toUpper"),
    f("toLower", "string -> string", "ASCII lowercase.", "strings", "lib.toLower"),
    f("capitalize", "string -> string", "Uppercase the first character and lowercase the rest.", "strings", ""),
    f("splitString", "string -> string -> [string]", "Split on a literal separator.", "strings", "lib.splitString"),
    f("join", "string -> [string] -> string", "Join values with a separator.", "strings", "lib.concatStringsSep"),
    f("toIntOr", "int -> string -> int", "Parse an integer, falling back to a default.", "strings", "lib.toInt"),
    f("toCamelCase", "string -> string", "Normalise to `camelCase`.", "strings", ""),
    f("toSnakeCase", "string -> string", "Normalise to `snake_case`.", "strings", ""),
    f("toKebabCase", "string -> string", "Normalise to `kebab-case`.", "strings", ""),
    // diagnostics
    f("fail", "string -> a", "Abort evaluation with a message.", "diagnostics", "builtins.throw"),
    f("assertMsg", "bool -> string -> true", "Abort with a message unless the condition holds.", "diagnostics", "lib.assertMsg"),
];

/// Helper to declare a catalogue entry compactly.
const fn f(
    name: &'static str,
    signature: &'static str,
    about: &'static str,
    group: &'static str,
    nixpkgs: &'static str,
) -> Function {
    Function {
        name,
        signature,
        about,
        group,
        nixpkgs,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rnix::ast::{Entry, HasEntry};

    /// Collect the attribute names of the top-level attribute set in
    /// `stdlib.nix`.
    fn exported_names() -> Vec<String> {
        let parsed = rnix::ast::Root::parse(STDLIB_SRC);
        assert!(
            parsed.errors().is_empty(),
            "stdlib.nix has parse errors: {:?}",
            parsed.errors()
        );
        let expr = parsed.tree().expr().expect("stdlib.nix has an expression");
        // `stdlib.nix` is `let … in { inherit …; }`; unwrap the body.
        let expr = match expr {
            rnix::ast::Expr::LetIn(let_in) => let_in.body().expect("let-in has a body"),
            other => other,
        };
        let rnix::ast::Expr::AttrSet(set) = expr else {
            panic!("stdlib.nix must evaluate to an attribute set");
        };
        let mut names = Vec::new();
        for entry in set.entries() {
            match entry {
                Entry::Inherit(inherit) => {
                    for attr in inherit.attrs() {
                        names.push(attr.to_string());
                    }
                }
                Entry::AttrpathValue(value) => {
                    if let Some(attr) = value.attrpath().and_then(|p| p.attrs().next()) {
                        names.push(attr.to_string());
                    }
                }
            }
        }
        names.sort();
        names
    }

    fn expected_names() -> Vec<String> {
        let mut names: Vec<String> = FUNCTIONS
            .iter()
            .map(|function| function.name.to_string())
            .chain(PRELUDE.iter().map(|name| name.to_string()))
            .collect();
        names.sort();
        names
    }

    #[test]
    fn catalogue_matches_source() {
        assert_eq!(exported_names(), expected_names());
    }

    #[test]
    fn catalogue_has_no_duplicates() {
        let mut names = expected_names();
        let before = names.len();
        names.dedup();
        assert_eq!(names.len(), before, "duplicate names in the catalogue");
    }

    #[test]
    fn catalogue_entries_are_complete() {
        for function in FUNCTIONS {
            assert!(!function.name.is_empty());
            assert!(
                !function.signature.is_empty(),
                "{} lacks a signature",
                function.name
            );
            assert!(
                !function.about.is_empty(),
                "{} lacks a description",
                function.name
            );
            assert!(
                !function.group.is_empty(),
                "{} lacks a group",
                function.name
            );
        }
    }
}
