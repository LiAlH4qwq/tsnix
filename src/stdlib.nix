# tsnix standard library (`std`).
#
# Pure Nix, no store and no I/O: every function is deterministic and works in
# any build. It is written to be used as `builtins.std` and as a bare `std`
# binding. Names follow cross-language conventions (Clojure / Elixir / lodash /
# Haskell) rather than nixpkgs; `tsnix libdoc` carries the nixpkgs mapping.
let
  inherit (builtins)
    map filter elem length head tail attrNames attrValues sort concatMap
    isAttrs isList isString hasAttr removeAttrs substring stringLength
    replaceStrings genList elemAt foldl' match split
    throw;

  mod = a: b: a - b * (a / b);

  digitValue = {
    "0" = 0; "1" = 1; "2" = 2; "3" = 3; "4" = 4;
    "5" = 5; "6" = 6; "7" = 7; "8" = 8; "9" = 9;
  };

  # ------------------------------------------------------------------ #
  # private helpers
  # ------------------------------------------------------------------ #

  # A path may be a list of keys, or a dotted string (`"a.b"`).
  toPath = p:
    if isString p
    then (if p == "" then [ ] else filter (s: isString s && s != "") (split "\\." p))
    else p;

  hasInRaw = path: attrs:
    if path == [ ] then true
    else
      isAttrs attrs
      && hasAttr (head path) attrs
      && hasInRaw (tail path) attrs.${head path};

  escapeRegex = replaceStrings
    [ "\\" "^" "$" "." "|" "?" "*" "+" "(" ")" "[" "]" "{" "}" ]
    [ "\\\\" "\\^" "\\$" "\\." "\\|" "\\?" "\\*" "\\+" "\\(" "\\)" "\\[" "\\]" "\\{" "\\}" ];

  stringToCharacters = s:
    genList (i: substring i 1 s) (stringLength s);

  isUpperChar = c: match "[A-Z]" c != null;
  isLowerChar = c: match "[a-z]" c != null;
  isDigitChar = c: match "[0-9]" c != null;
  isSeparator = c: c == "-" || c == "_" || c == " " || c == ".";

  lowerChars = [
    "a" "b" "c" "d" "e" "f" "g" "h" "i" "j" "k" "l" "m"
    "n" "o" "p" "q" "r" "s" "t" "u" "v" "w" "x" "y" "z"
  ];
  upperChars = [
    "A" "B" "C" "D" "E" "F" "G" "H" "I" "J" "K" "L" "M"
    "N" "O" "P" "Q" "R" "S" "T" "U" "V" "W" "X" "Y" "Z"
  ];

  splitWords = s: filter (w: isString w && w != "") (split "[^A-Za-z0-9]+" s);

  # ------------------------------------------------------------------ #
  # utilities
  # ------------------------------------------------------------------ #

  id = x: x;
  const = x: _: x;
  flip = f: a: b: f b a;
  pipe = value: fs: foldl' (acc: f: f acc) value fs;

  # ------------------------------------------------------------------ #
  # conditional construction
  # ------------------------------------------------------------------ #

  optional = cond: x: if cond then [ x ] else [ ];
  optionals = cond: xs: if cond then xs else [ ];
  optionalAttrs = cond: attrs: if cond then attrs else { };
  optionalString = cond: s: if cond then s else "";

  # ------------------------------------------------------------------ #
  # merging
  # ------------------------------------------------------------------ #

  mergeWith = f: a: b:
    if isAttrs a && isAttrs b then
      foldl' (acc: name:
        if hasAttr name a
        then acc // { ${name} = mergeWith f a.${name} b.${name}; }
        else acc // { ${name} = b.${name}; }
      ) a (attrNames b)
    else f a b;

  merge = mergeWith (_: b: b);
  defaults = base: overrides: merge base overrides;

  # ------------------------------------------------------------------ #
  # deep paths (get-in / assoc-in style)
  # ------------------------------------------------------------------ #

  getIn = path: attrs:
    let
      go = acc: p:
        if p == [ ] then acc
        else if isAttrs acc && hasAttr (head p) acc
        then go acc.${head p} (tail p)
        else null;
    in go attrs (toPath path);

  getInOr = default: path: attrs:
    let value = getIn path attrs; in if value == null then default else value;

  hasIn = path: attrs: hasInRaw (toPath path) attrs;

  assocIn = path: value: attrs:
    let p = toPath path; in
    if p == [ ] then value
    else attrs // { ${head p} = assocIn (tail p) value (attrs.${head p} or { }); };

  updateIn = path: f: attrs: assocIn path (f (getIn path attrs)) attrs;

  removeIn = path: attrs:
    let p = toPath path; in
    if p == [ ] then attrs
    else if length p == 1 then removeAttrs attrs p
    else if isAttrs attrs && hasAttr (head p) attrs && isAttrs attrs.${head p}
    then attrs // { ${head p} = removeIn (tail p) attrs.${head p}; }
    else attrs;

  selectKeys = paths: attrs:
    foldl' (acc: path:
      let p = toPath path; in
      if hasIn p attrs then assocIn p (getIn p attrs) acc else acc
    ) { } paths;

  omitKeys = paths: attrs: foldl' (acc: path: removeIn path acc) attrs paths;

  renameKey = from: to: attrs:
    if hasAttr from attrs
    then (removeAttrs attrs [ from ]) // { ${to} = attrs.${from}; }
    else attrs;

  # ------------------------------------------------------------------ #
  # attribute sets
  # ------------------------------------------------------------------ #

  nameValuePair = name: value: { inherit name value; };

  mapEntries = f: attrs:
    foldl' (acc: name:
      let entry = f name attrs.${name}; in
      acc // { ${entry.name} = entry.value; }
    ) { } (attrNames attrs);

  mapAttrsToList = f: attrs: map (name: f name attrs.${name}) (attrNames attrs);

  filterAttrs = pred: attrs:
    foldl' (acc: name:
      if pred name attrs.${name} then acc // { ${name} = attrs.${name}; } else acc
    ) { } (attrNames attrs);

  rejectAttrs = pred: attrs: filterAttrs (name: value: !(pred name value)) attrs;

  foldlAttrs = f: init: attrs:
    foldl' (acc: name: f acc name attrs.${name}) init (attrNames attrs);

  mapKeys = f: attrs:
    foldl' (acc: name: acc // { ${f name} = attrs.${name}; }) { } (attrNames attrs);

  compactAttrs = attrs: filterAttrs (_: value: value != null) attrs;

  # ------------------------------------------------------------------ #
  # lists
  # ------------------------------------------------------------------ #

  flatten = foldl' (acc: x: acc ++ (if isList x then flatten x else [ x ])) [ ];

  flatMap = concatMap;

  compact = xs: filter (x: x != null) xs;

  unique = foldl' (acc: x: if elem x acc then acc else acc ++ [ x ]) [ ];

  uniqueBy = f: foldl' (acc: x:
    if any (y: f x == f y) acc then acc else acc ++ [ x ]
  ) [ ];

  any = pred: xs: foldl' (acc: x: acc || pred x) false xs;

  reverse = xs:
    genList (i: elemAt xs (length xs - i - 1)) (length xs);

  range = first: last:
    let
      count = if last >= first then last - first + 1 else first - last + 1;
      step = if last >= first then (i: first + i) else (i: first - i);
    in genList step count;

  repeat = n: x: genList (_: x) (max [ 0 n ]);

  count = pred: xs: length (filter pred xs);

  find = pred: xs:
    let matches = filter pred xs; in
    if matches == [ ] then null else head matches;

  findOr = default: pred: xs:
    let value = find pred xs; in if value == null then default else value;

  last = xs:
    if xs == [ ] then throw "std.last: empty list"
    else elemAt xs (length xs - 1);

  zipWith = f: a: b:
    genList (i: f (elemAt a i) (elemAt b i)) (min [ (length a) (length b) ]);

  zip = zipWith (a: b: { fst = a; snd = b; });

  enumerate = xs:
    genList (i: { index = i; value = elemAt xs i; }) (length xs);

  mapWithIndex = f: xs:
    genList (i: f i (elemAt xs i)) (length xs);

  sum = xs: foldl' (a: b: a + b) 0 xs;

  min = xs:
    if xs == [ ] then throw "std.min: empty list"
    else foldl' (a: b: if a < b then a else b) (head xs) (tail xs);

  max = xs:
    if xs == [ ] then throw "std.max: empty list"
    else foldl' (a: b: if a > b then a else b) (head xs) (tail xs);

  sortBy = f: xs: sort (a: b: f a < f b) xs;

  take = n: xs: genList (i: elemAt xs i) (min [ n (length xs) ]);

  drop = n: xs:
    let
      start = max [ 0 n ];
      remaining = max [ 0 (length xs - start) ];
    in genList (i: elemAt xs (start + i)) remaining;

  slice = start: len: xs:
    let
      from = max [ 0 start ];
      remaining = max [ 0 (min [ len (length xs - from) ]) ];
    in genList (i: elemAt xs (from + i)) remaining;

  intersperse = sep: xs:
    let n = length xs; in
    if n <= 1 then xs
    else genList
      (i: if i == 0 then elemAt xs 0
          else if mod i 2 == 0 then elemAt xs (i / 2)
          else sep)
      (2 * n - 1);

  chunk = n: xs:
    if n <= 0 then throw "std.chunk: size must be positive"
    else
      let
        len = length xs;
        chunks = (len + n - 1) / n;
      in genList
        (i: genList (j: elemAt xs (i * n + j)) (min [ n (len - i * n) ]))
        chunks;

  # ------------------------------------------------------------------ #
  # strings
  # ------------------------------------------------------------------ #

  hasPrefix = prefix: s:
    substring 0 (stringLength prefix) s == prefix;

  hasSuffix = suffix: s:
    let l = stringLength suffix; in
    stringLength s >= l && substring (stringLength s - l) l s == suffix;

  removePrefix = prefix: s:
    if hasPrefix prefix s then substring (stringLength prefix) (stringLength s) s else s;

  removeSuffix = suffix: s:
    if hasSuffix suffix s then substring 0 (stringLength s - stringLength suffix) s else s;

  trim = s:
    let m = match "[[:space:]]*(.*[^[:space:]])?[[:space:]]*" s; in
    if m == null || head m == null then "" else head m;

  trimStart = s:
    let m = match "[[:space:]]*(.*)" s; in
    if m == null then s else head m;

  trimEnd = s:
    let m = match "(.*[^[:space:]])?[[:space:]]*" s; in
    if m == null || head m == null then "" else head m;

  toUpper = s: replaceStrings lowerChars upperChars s;
  toLower = s: replaceStrings upperChars lowerChars s;

  capitalize = s:
    if s == "" then s
    else toUpper (substring 0 1 s) + substring 1 (stringLength s) s;

  splitString = sep: s:
    if sep == "" then stringToCharacters s
    else filter (x: x != "") (filter isString (split (escapeRegex sep) s));

  join = sep: xs: foldl' (acc: x: if acc == "" then x else acc + sep + x) "" (map toString' xs);

  toString' = x:
    if isAttrs x || isList x then throw "std.join: cannot join a list or attribute set"
    else builtins.toString x;

  toIntOr = default: s:
    let m = match "[[:space:]]*([+-]?[0-9]+)[[:space:]]*" s; in
    if m == null then default
    else
      let
        text = head m;
        first = substring 0 1 text;
        negative = first == "-";
        digits = if first == "-" || first == "+" then substring 1 (stringLength text) text else text;
        value = foldl' (acc: c: acc * 10 + digitValue.${c}) 0 (stringToCharacters digits);
      in if negative then -value else value;

  toSnakeCase = s:
    let
      chars = stringToCharacters s;
      n = length chars;
      endsWithUnderscore = acc:
        acc != "" && substring (stringLength acc - 1) 1 acc == "_";
      go = acc: i:
        let
          c = elemAt chars i;
          prev = if i == 0 then "" else elemAt chars (i - 1);
          next = if i + 1 < n then elemAt chars (i + 1) else "";
          upper = isUpperChar c;
          separator = isSeparator c;
          needSep = !separator && upper && !(endsWithUnderscore acc)
            && (isLowerChar prev || isDigitChar prev
                || (isUpperChar prev && isLowerChar next));
          sep = if separator then (if endsWithUnderscore acc then "" else "_")
                else if needSep then "_" else "";
        in if separator then acc + sep
           else acc + sep + (if upper then toLower c else c);
    in foldl' go "" (genList (i: i) n);

  toCamelCase = s:
    let words = splitWords (toSnakeCase s); in
    if words == [ ] then ""
    else head words + foldl' (acc: w: acc + capitalize w) "" (tail words);

  toKebabCase = s: replaceStrings [ "_" ] [ "-" ] (toSnakeCase s);

  # ------------------------------------------------------------------ #
  # diagnostics
  # ------------------------------------------------------------------ #

  fail = throw;
  assertMsg = cond: msg: if cond then true else throw msg;
in
{
  inherit
    id const flip pipe
    optional optionals optionalAttrs optionalString
    merge mergeWith defaults
    getIn getInOr hasIn assocIn updateIn removeIn selectKeys omitKeys renameKey
    nameValuePair mapEntries mapAttrsToList filterAttrs rejectAttrs foldlAttrs mapKeys compactAttrs
    flatten flatMap compact unique uniqueBy reverse range repeat count find findOr last
    zip zipWith enumerate mapWithIndex sum min max sortBy take drop slice intersperse chunk
    hasPrefix hasSuffix removePrefix removeSuffix trim trimStart trimEnd toUpper toLower
    capitalize splitString join toIntOr toCamelCase toSnakeCase toKebabCase
    fail assertMsg
    # core builtins, re-exported so `with std;` works as a prelude
    map filter foldl' concatMap sort elem length head tail attrNames attrValues;
}
