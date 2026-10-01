# 标准库

`tsnix` 内置一个精简的标准库，命名空间为 **`std`**。它完全用纯 Nix 编写，
只使用 builtins，既不访问 store 也不访问文件系统，因此在任何构建下都可用
（包括 `--no-default-features` 与 wasm）。

- 以 `std`（顶层绑定）和 `builtins.std` 两种形式提供。
- `with std;` 可作为 prelude：核心 list/attr builtin 会被再导出。
- 默认编译进产物；`cargo build --no-default-features`（或去掉 `stdlib`
  feature）可关闭。
- 用 `tsnix libdoc` 查看完整目录（见下）。

## 用法

```console
$ tsnix eval -e 'std.merge { a = { x = 1; }; } { a = { y = 2; }; }'
{"a":{"x":1,"y":2}}

$ tsnix eval -e 'std.toSnakeCase "HTTPServerConfig"'
"http_server_config"

$ tsnix eval -e 'with std; join "," (map toString (sortBy (x: x) [ 3 1 2 ]))'
"1,2,3"
```

用户绑定优先于注入的绑定，因此 `--arg std …` 会覆盖该命名空间（`builtins.std`
仍然存在）。

## 命名约定

命名参考了各语言的通用惯例（Clojure、Elixir、lodash、Haskell），而不是
nixpkgs，且目录是扁平的。下表 `nixpkgs` 一列给出最接近的 nixpkgs `lib`
对应项，方便从 nixpkgs 迁移的读者参考，并不代表兼容性保证。

设计规则：

- 被操作的集合/属性集是**最后一个**参数，便于柯里化与 `std.pipe` 组合。
- 深路径函数接受列表（`[ "a" "b" ]`）或点号字符串（`"a.b"`）。
- 失败场景有显式的默认值路径：`getIn`/`getInOr`、`find`/`findOr`、`toIntOr`。
  `null` 表示“缺失”，因此 `getInOr`/`findOr` 会把真正的 `null` 视作缺失。
- `std` 中不含任何可被序列化为 JSON 的函数，因此其结果在 `builtins.toJSON`
  下始终安全；谓词留在 Nix 侧。

由于 `snix-eval` 尚未实现 `|>` 运算符，`std.pipe` 是链式组合的原语：

```nix
std.pipe { host = "example.com"; }
  [ (std.assocIn "port" 8080)
    (std.renameKey "host" "hostname")
  ]
```

## Prelude

以下 builtin 被再导出，使 `with std;` 可作为 prelude：

`map`、`filter`、`foldl'`、`concatMap`、`sort`、`elem`、`length`、`head`、
`tail`、`attrNames`、`attrValues`。

## 函数目录

<!-- 该表由 `tsnix libdoc` 生成，请保持同步。 -->

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

`tsnix libdoc` 以 JSON 打印目录（即生成上表的数据），`tsnix schema` 会将其嵌入
`"stdlib"` 字段：

```console
$ tsnix libdoc --pretty
$ tsnix schema | jq .stdlib.available
```

目录与实现共享同一份 Rust 元数据，并有单元测试解析 `src/stdlib.nix` 断言导出名
与目录完全一致，因此二者不会漂移。
