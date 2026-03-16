# Collections — Reference

## Literals

```
[1 2 3 4 5]              -- list
{x: 3.0  y: 4.0}         -- record
%{"alice": 1  "bob": 2}  -- map (arbitrary keys)
(1 "hello" true)          -- tuple
```

No commas — whitespace separates.

## Lists

```
xs.0                  -- first (zero-based)
xs.-1                 -- last (negative index)
xs.1..3               -- [2 3] (slice)
xs.2..                -- from index 2 to end
xs...3                -- from start to index 3
[0 ..xs 6]            -- spread: [0 1 2 3 4 5 6]
[..a ..b]             -- concatenate
get 10 xs             -- None (safe access)
get 0 xs              -- Some 1
```

Immutable update via spread + slicing:

```
[..xs.0..2  99  ..xs.3..]     -- replace at index 2
[..xs.0..2  99  ..xs.2..]     -- insert at index 2
[..xs.0..2  ..xs.3..]         -- remove at index 2
```

## Records

```
p = {x: 3.0  y: 4.0}
p.x                          -- 3.0
{..p  x: 5.0}               -- spread + update
{..a  ..b}                   -- merge (b wins)
x = 3.0; y = 4.0
{x  y}                       -- shorthand for {x: x  y: y}
```

## Maps

```
m = %{"alice": 1  "bob": 2}
m."alice"                    -- 1
get "carol" m                -- None
%{..m  "carol": 3}          -- spread + merge
keys m       values m        entries m
has_key? "alice" m           remove "bob" m           merge m1 m2
```

Maps iterate over `(key value)` tuples with `map`/`filter`/etc.

## Tuples

```
t = (1 "hello" true)
t.0                          -- 1
(a b c) = get_triple ()      -- destructure
```

## Record vs Map

**Records**: fixed ident keys, `.field` access, structural typing.
**Maps**: dynamic expression keys, `."key"` access, `%` prefix. JSON-parsed data returns maps.

## Conversions

```
to_map record       to_map entries_list       to_record map       to_list map
```

`to_record` fails if map keys are not valid identifiers.

## Concatenation

`[1 2] ++ [3 4]` / `"a" ++ "b"`. Prefer spread for literals.
