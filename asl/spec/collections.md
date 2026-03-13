# Collections

Lists, records, maps, sets, and tuples — their literals, access patterns, and operations.

## Literals

```
xs = [1 2 3 4 5]              -- list (ordered, homogeneous)
p = {x: 3.0  y: 4.0}          -- record (ordered name-value pairs)
m = %{"alice": 1  "bob": 2}   -- map (arbitrary keys)
s = #{1 2 3}                   -- set (unordered, unique)
t = (1 "hello" true)           -- tuple (fixed-size, heterogeneous)
```

No commas between elements — whitespace separates. Two spaces between record fields for visual clarity (convention, not required).

## Records

Records are name-value pairs that preserve insertion order for field access and iteration. Equality is order-independent (see [runtime.md](runtime.md)). Field names are identifiers (lowercase).

```
p = {x: 3.0  y: 4.0}
```

Field access with `.`:

```
p.x                   -- 3.0
p.y                   -- 4.0
```

Spread to copy and update:

```
{..p  x: 5.0}        -- {x: 5.0  y: 4.0}
{..a  ..b}            -- merge (b wins on conflict)
```

Shorthand when variable name matches field name:

```
x = 3.0
y = 4.0
p = {x  y}            -- same as {x: x  y: y}
```

## Lists

Ordered, homogeneous sequences.

```
xs = [1 2 3 4 5]
```

Access by index with `.` (zero-based):

```
xs.0                  -- 1 (first)
xs.4                  -- 5 (last)
xs.-1                 -- 5 (last, negative index)
xs.-2                 -- 4 (second to last)
```

Slicing with `..`:

```
xs.1..3               -- [2 3]
xs.2..                -- [3 4 5] (from index 2 to end)
xs...3                -- [1 2 3] (from start to index 3)
```

Spread in list literals:

```
[0 ..xs 6]            -- [0 1 2 3 4 5 6]
[..a ..b]             -- concatenate
```

Direct access (`.0`, `.-1`) errors on out-of-bounds. Use `get` for safe access:

```
xs.10                 -- runtime error
get 10 xs             -- None
get 0 xs              -- Some 1
```

## Maps

Arbitrary key-value pairs. Keys are expressions (usually strings), prefixed with `%`. Keys must be comparable values — functions cannot be map keys (since functions are not comparable).

```
m = %{"alice": 1  "bob": 2}
```

Access:

```
m."alice"             -- 1
get "carol" m         -- None
```

Spread works like records:

```
%{..m  "carol": 3}   -- merge
```

## Sets

Unordered, unique values. Prefixed with `#`.

```
s = #{1 2 3 4 5}
contains? 3 s         -- true
#{..a ..b}            -- union via spread
```

Set operations:

```
a = #{1 2 3}
b = #{2 3 4}
intersect a b         -- #{2 3}
difference a b        -- #{1}
sym_diff a b          -- #{1 4}
is_subset? a b        -- false
is_superset? a b      -- false
```

Sets are iterable — they work with `map`, `filter`, `fold`, and other pipeline functions. The iteration order is not guaranteed.

## Tuples

Fixed-size, heterogeneous. Use parens.

```
t = (1 "hello" true)
t.0                   -- 1
t.1                   -- "hello"
```

Destructure with pattern matching:

```
(a b c) = get_triple ()
```

## Concatenation

`++` concatenates lists and strings at runtime:

```
[1 2] ++ [3 4]         -- [1 2 3 4]
"hello" ++ " world"    -- "hello world"
```

For literals, spread is preferred: `[..a ..b]`. Use `++` for computed values in pipelines.

## Immutable List Updates

Lists are immutable by default. To "update" an element, construct a new list using spread and slicing:

```
xs = [10 20 30 40 50]

-- replace element at index 2
[..xs.0..2  99  ..xs.3..]     -- [10 20 99 40 50]

-- insert at index 2
[..xs.0..2  99  ..xs.2..]     -- [10 20 99 30 40 50]

-- remove element at index 2
[..xs.0..2  ..xs.3..]         -- [10 20 40 50]
```

For frequent updates, use a mutable list:

```
xs := [10 20 30 40 50]
xs <- [..xs.0..2  99  ..xs.3..]
```

## Map Operations

```
m = %{"alice": 1  "bob": 2  "carol": 3}
keys m                -- ["alice" "bob" "carol"]
values m              -- [1 2 3]
entries m             -- [("alice" 1) ("bob" 2) ("carol" 3)]
has_key? "alice" m    -- true
remove "bob" m        -- %{"alice": 1  "carol": 3}
merge m1 m2           -- combine (m2 wins on conflict)
```

Maps are iterable — `map`, `filter`, etc. iterate over `(key value)` tuples:

```
%{"a": 1  "b": 2} | filter (kv) kv.1 > 1    -- [("b" 2)]
%{"a": 1  "b": 2} | map (kv) { (kv.0 kv.1 * 2) } | to_map  -- %{"a": 2  "b": 4}
```

`to_map` converts a list of `(key value)` tuples into a map. `to_list` converts a map to `[(key value)]`.

## Record vs Map

Records and maps both store key-value pairs but serve different purposes:

**Records** — fixed, known fields. Keys are identifiers. Access with `.field`. Structural typing applies.

```
p = {x: 3.0  y: 4.0}
p.x                          -- 3.0
```

**Maps** — dynamic keys. Keys are arbitrary expressions (usually strings). Prefixed with `%`. Access with `."key"` or `get`.

```
m = %{"alice": 1  "bob": 2}
m."alice"                    -- 1
get "carol" m                -- None
```

JSON-parsed data returns maps (dynamic keys), not records. Use `."key"` for map field access.

## Conversions

```
to_map record         -- {x: 1  y: 2} -> %{"x": 1  "y": 2}
to_map entries        -- [("a" 1) ("b" 2)] -> %{"a": 1  "b": 2}
to_record map         -- %{"x": 1  "y": 2} -> {x: 1  y: 2} (keys must be valid identifiers)
to_list map           -- %{"a": 1} -> [("a" 1)]
to_list set           -- #{1 2 3} -> [1 2 3] (order not guaranteed)
to_set list           -- [1 2 2 3] -> #{1 2 3}
```

`to_record` fails at runtime if any map key is not a valid identifier (starts with `[a-z_]`, contains only `[a-z0-9_]`).

## Collection Size Limits

No artificial limits. Lists, maps, and sets grow as needed, bounded only by available memory. For large datasets that don't fit in memory, use lazy sequences with streaming pipelines.

## Cross-References

- Implementation: [impl-interpreter.md](../impl/impl-interpreter.md) (Value representation), [impl-builtins.md](../impl/impl-builtins.md) (collection functions)
- Test suite: [06_collections.lx](../suite/06_collections.lx)
