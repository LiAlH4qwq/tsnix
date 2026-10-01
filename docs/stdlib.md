# Standard library

`tsnix` ships a small standard library under the namespace **`std`**. It is
written in pure Nix, uses only builtins, and touches neither a store nor the
filesystem, so it works in every build (including `--no-default-features` and
wasm).

- Available as `std` (a top-level binding) and as `builtins.std`.
- `with std;` works as a prelude: the core list/attr builtins are re-exported.
- Compiled in by default; disable it with `cargo build --no-default-features`
  (or by dropping the `stdlib` feature).
- Discover it with `tsnix libdoc` (see below).

## Usage

```console
$ tsnix eval -e 'std.merge { a = { x = 1; }; } { a = { y = 2; }; }'
{"a":{"x":1,"y":2}}

$ tsnix eval -e 'std.toSnakeCase "HTTPServerConfig"'
"http_server_config"

$ tsnix eval -e 'with std; join "," (map toString (sortBy (x: x) [ 3 1 2 ]))'
"1,2,3"
```

A user binding wins over the injected one, so `--arg std …` overrides the
namespace without disabling it (`builtins.std` still exists).

## Naming

Names follow common cross-language conventions (Clojure, Elixir, lodash,
Haskell) rather than nixpkgs, and the catalogue is flat. The `nixpkgs` column
below points at the closest nixpkgs `lib` equivalent, for readers migrating
from nixpkgs; it is not a compatibility guarantee.

Design rules:

- The collection/attribute set is the **last** argument, so functions curry and
  compose under `std.pipe`.
- Deep-path functions accept a list (`[ "a" "b" ]`) or a dotted string
  (`"a.b"`).
- Failure has an explicit default path: `getIn`/`getInOr`, `find`/`findOr` and
  `toIntOr`. `null` is the "absent" sentinel, so `getInOr`/`findOr` treat a real
  `null` as absent.
- `std` holds no functions that can be serialised into JSON, so its results are
  always safe under `builtins.toJSON`; predicates stay on the Nix side.

Because `snix-eval` does not implement the `|>` operator, `std.pipe` is the
chaining primitive:

```nix
std.pipe { host = "example.com"; }
  [ (std.assocIn "port" 8080)
    (std.renameKey "host" "hostname")
  ]
```

## Prelude

These builtins are re-exported so `with std;` is useful as a prelude:

`map`, `filter`, `foldl'`, `concatMap`, `sort`, `elem`, `length`, `head`,
`tail`, `attrNames`, `attrValues`.

## Catalogue

<!-- This table is generated from `tsnix libdoc`; keep it in sync. -->

### utilities

| function | signature | description | nixpkgs |
| --- | --- | --- | --- |
| `std.id` | `x -> x` | Return the argument unchanged. | `lib.id` |
| `std.const` | `x -> y -> x` | Return the first argument, ignoring the second. | `lib.const` |
| `std.flip` | `(a -> b -> c) -> b -> a -> c` | Flip the first two arguments of a function. | `lib.flip` |
| `std.pipe` | `value -> [ (a -> a) ] -> a` | Apply a list of functions left to right. | `lib.pipe` |

### conditional

| function | signature | description | nixpkgs |
| --- | --- | --- | --- |
| `std.optional` | `bool -> x -> [x]` | A singleton list when the condition holds, otherwise `[]`. | `lib.optional` |
| `std.optionals` | `bool -> [x] -> [x]` | The list when the condition holds, otherwise `[]`. | `lib.optionals` |
| `std.optionalAttrs` | `bool -> {..} -> {..}` | The attribute set when the condition holds, otherwise `{}`. | `lib.optionalAttrs` |
| `std.optionalString` | `bool -> string -> string` | The string when the condition holds, otherwise `""`. | `lib.optionalString` |

### merging

| function | signature | description | nixpkgs |
| --- | --- | --- | --- |
| `std.merge` | `{..} -> {..} -> {..}` | Deep merge, right side wins; non-attribute sets are replaced. | `lib.recursiveUpdate` |
| `std.mergeWith` | `(x -> y -> z) -> {..} -> {..} -> {..}` | Deep merge, resolving leaf conflicts with a function. | — |
| `std.defaults` | `{..} -> {..} -> {..}` | `merge base overrides`: fill in defaults. | — |

### paths

| function | signature | description | nixpkgs |
| --- | --- | --- | --- |
| `std.getIn` | `path -> attrs -> value or null` | Deep lookup; `null` when absent. `path` may be a list or `"a.b"`. | `lib.attrByPath` |
| `std.getInOr` | `default -> path -> attrs -> value` | Deep lookup with a default (`null` counts as absent). | `lib.attrByPath` |
| `std.hasIn` | `path -> attrs -> bool` | Whether a deep path exists. | `lib.hasAttrByPath` |
| `std.assocIn` | `path -> value -> attrs -> attrs` | Immutable deep update, creating intermediate sets. | `lib.setAttrByPath` |
| `std.updateIn` | `path -> (value -> value) -> attrs -> attrs` | Apply a function at a deep path. | — |
| `std.removeIn` | `path -> attrs -> attrs` | Deep delete; a missing path is a no-op. | — |
| `std.selectKeys` | `[ path ] -> attrs -> attrs` | Project a whitelist of paths, preserving structure. | — |
| `std.omitKeys` | `[ path ] -> attrs -> attrs` | Drop a blacklist of paths. | — |
| `std.renameKey` | `from -> to -> attrs -> attrs` | Rename a top-level key. | — |

### attrs

| function | signature | description | nixpkgs |
| --- | --- | --- | --- |
| `std.nameValuePair` | `name -> value -> { name; value; }` | Build a `{ name; value; }` pair. | `lib.nameValuePair` |
| `std.mapEntries` | `(name -> value -> { name; value; }) -> attrs -> attrs` | Map keys and values at once (later duplicates win). | `lib.mapAttrs'` |
| `std.mapAttrsToList` | `(name -> value -> x) -> attrs -> [x]` | Map each entry to a list element. | `lib.mapAttrsToList` |
| `std.filterAttrs` | `(name -> value -> bool) -> attrs -> attrs` | Keep entries for which the predicate holds. | `lib.filterAttrs` |
| `std.rejectAttrs` | `(name -> value -> bool) -> attrs -> attrs` | Drop entries for which the predicate holds. | — |
| `std.foldlAttrs` | `(acc -> name -> value -> acc) -> acc -> attrs -> acc` | Left fold over attribute entries. | `lib.foldlAttrs` |
| `std.mapKeys` | `(name -> name') -> attrs -> attrs` | Transform keys, keeping values. | — |
| `std.compactAttrs` | `attrs -> attrs` | Drop entries whose value is `null`. | — |

### lists

| function | signature | description | nixpkgs |
| --- | --- | --- | --- |
| `std.flatten` | `[[..]] -> [..]` | Deeply flatten nested lists. | `lib.flatten` |
| `std.flatMap` | `(x -> [y]) -> [x] -> [y]` | Map then concatenate (alias of `builtins.concatMap`). | `lib.concatMap` |
| `std.compact` | `[x] -> [x]` | Drop `null` elements. | — |
| `std.unique` | `[x] -> [x]` | Remove duplicates, keeping first occurrences. | `lib.unique` |
| `std.uniqueBy` | `(x -> k) -> [x] -> [x]` | Remove duplicates by a derived key. | — |
| `std.reverse` | `[x] -> [x]` | Reverse a list. | `lib.reverseList` |
| `std.range` | `int -> int -> [int]` | Inclusive integer range (ascending or descending). | `lib.range` |
| `std.repeat` | `int -> x -> [x]` | A list containing `x` repeated `n` times. | `lib.replicate` |
| `std.count` | `(x -> bool) -> [x] -> int` | Count elements satisfying a predicate. | `lib.count` |
| `std.find` | `(x -> bool) -> [x] -> x or null` | First matching element, or `null`. | `lib.findFirst` |
| `std.findOr` | `default -> (x -> bool) -> [x] -> x` | First matching element, or a default. | `lib.findFirst` |
| `std.last` | `[x] -> x` | The last element (throws on `[]`). | `lib.last` |
| `std.zip` | `[a] -> [b] -> [{ fst; snd; }]` | Pair up two lists, truncating to the shorter. | `lib.zipLists` |
| `std.zipWith` | `(a -> b -> c) -> [a] -> [b] -> [c]` | Combine two lists element-wise. | `lib.zipListsWith` |
| `std.enumerate` | `[x] -> [{ index; value; }]` | Pair each element with its index. | — |
| `std.mapWithIndex` | `(int -> x -> y) -> [x] -> [y]` | Map with the element index. | `lib.imap0` |
| `std.sum` | `[number] -> number` | Sum a list of numbers. | `lib.sum` |
| `std.min` | `[number] -> number` | Smallest element (throws on `[]`). | `lib.minimum` |
| `std.max` | `[number] -> number` | Largest element (throws on `[]`). | `lib.maximum` |
| `std.sortBy` | `(x -> k) -> [x] -> [x]` | Sort by a derived key. | `lib.sortOn` |
| `std.take` | `int -> [x] -> [x]` | The first `n` elements. | `lib.take` |
| `std.drop` | `int -> [x] -> [x]` | All but the first `n` elements. | `lib.drop` |
| `std.slice` | `int -> int -> [x] -> [x]` | `len` elements starting at `start`. | `lib.sublist` |
| `std.intersperse` | `x -> [x] -> [x]` | Insert a separator between elements. | `lib.intersperse` |
| `std.chunk` | `int -> [x] -> [[x]]` | Split into consecutive chunks of size `n`. | `lib.chunk` |

### strings

| function | signature | description | nixpkgs |
| --- | --- | --- | --- |
| `std.hasPrefix` | `string -> string -> bool` | Whether a string starts with a prefix. | `lib.hasPrefix` |
| `std.hasSuffix` | `string -> string -> bool` | Whether a string ends with a suffix. | `lib.hasSuffix` |
| `std.removePrefix` | `string -> string -> string` | Remove a leading prefix when present. | `lib.removePrefix` |
| `std.removeSuffix` | `string -> string -> string` | Remove a trailing suffix when present. | `lib.removeSuffix` |
| `std.trim` | `string -> string` | Strip leading and trailing whitespace. | `lib.trim` |
| `std.trimStart` | `string -> string` | Strip leading whitespace. | — |
| `std.trimEnd` | `string -> string` | Strip trailing whitespace. | — |
| `std.toUpper` | `string -> string` | ASCII uppercase. | `lib.toUpper` |
| `std.toLower` | `string -> string` | ASCII lowercase. | `lib.toLower` |
| `std.capitalize` | `string -> string` | Uppercase the first character and lowercase the rest. | — |
| `std.splitString` | `string -> string -> [string]` | Split on a literal separator. | `lib.splitString` |
| `std.join` | `string -> [string] -> string` | Join values with a separator. | `lib.concatStringsSep` |
| `std.toIntOr` | `int -> string -> int` | Parse an integer, falling back to a default. | `lib.toInt` |
| `std.toCamelCase` | `string -> string` | Normalise to `camelCase`. | — |
| `std.toSnakeCase` | `string -> string` | Normalise to `snake_case`. | — |
| `std.toKebabCase` | `string -> string` | Normalise to `kebab-case`. | — |

### diagnostics

| function | signature | description | nixpkgs |
| --- | --- | --- | --- |
| `std.fail` | `string -> a` | Abort evaluation with a message. | `builtins.throw` |
| `std.assertMsg` | `bool -> string -> true` | Abort with a message unless the condition holds. | `lib.assertMsg` |

## `tsnix libdoc`

`tsnix libdoc` prints the catalogue as JSON (the same data that generated the
tables above), and `tsnix schema` embeds it under `"stdlib"`:

```console
$ tsnix libdoc --pretty
$ tsnix schema | jq .stdlib.available
```

The catalogue is compiled from the same Rust metadata as the implementation,
and a unit test parses `src/stdlib.nix` to assert that the exported names match
the catalogue exactly, so the two cannot drift.
