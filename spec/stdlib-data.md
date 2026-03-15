# Standard Library — Data Ecosystem (Future)

**Status: Not implemented.** These modules are planned for future phases. None of this code works today.

Data processing, persistence, numerical computation, ML inference, and visualization modules. These are stdlib modules — no language changes required. All follow data-last conventions and compose naturally with pipes and sections.

See [stdlib-modules.md](stdlib-modules.md) for core modules (fs, http, json, time, etc.).

## std/df — Dataframes (Polars)

Columnar data processing backed by Polars. Dataframes are opaque types — created by `df.read_*` or `df.from_*`, consumed by `df.*` operations. Lazy by default (like Polars LazyFrame). `df.collect` forces evaluation.

```
read_csv path                 -- DataFrame ^ IoErr
read_csv_with opts path       -- opts: {delimiter: Str  header: Bool  skip: Int  schema: %{}}
read_parquet path             -- DataFrame ^ IoErr
read_json path                -- DataFrame ^ IoErr (newline-delimited JSON)
from_records records          -- DataFrame (from [{...}] list of records)
from_map map                  -- DataFrame (from %{col: [values]} column map)

filter pred df                -- DataFrame (pred is a column expression)
select cols df                -- DataFrame (cols: [Str] or column expressions)
drop cols df                  -- DataFrame (remove columns)
rename mapping df             -- DataFrame (mapping: %{old: new})
sort_by cols df               -- DataFrame (cols: [Str] or column expressions)
rev df                        -- DataFrame (reverse row order)
take n df                     -- DataFrame (first n rows)
drop_rows n df                -- DataFrame (skip first n rows)
head n df                     -- DataFrame (alias for take)
tail n df                     -- DataFrame (last n rows)

group_by cols df              -- GroupedFrame
agg exprs grouped             -- DataFrame (exprs: {name: agg_expr} record)

join how right_df on left_df  -- DataFrame. how: "inner" | "left" | "outer" | "cross"
concat dfs                    -- DataFrame (vertical stack of [DataFrame])
melt id_cols df               -- DataFrame (wide to long)
pivot index cols values df    -- DataFrame (long to wide)

col name                      -- ColumnExpr (reference a column by name)
lit val                        -- ColumnExpr (literal value as column)
sum col_expr                  -- AggExpr
mean col_expr                 -- AggExpr
median col_expr               -- AggExpr
std_dev col_expr              -- AggExpr
min_agg col_expr              -- AggExpr
max_agg col_expr              -- AggExpr
count                         -- AggExpr (row count)
n_unique col_expr             -- AggExpr

shape df                      -- (Int Int) (rows, cols)
columns df                    -- [Str]
dtypes df                     -- [Str]
describe df                   -- DataFrame (summary statistics)
null_count df                 -- DataFrame (null count per column)
unique col df                 -- DataFrame (unique values in column)

to_records df                 -- [{...}] list of records
to_map df                     -- %{col: [values]} column map
write_csv path df             -- () ^ IoErr
write_parquet path df         -- () ^ IoErr
write_json path df            -- () ^ IoErr

collect df                    -- DataFrame (force lazy evaluation)
each f df                     -- () (iterate rows as records, side effects)
```

Sections work as column selectors: `df.filter (.amount > 1000)` creates a column predicate from the section. `df.select [(.name) (.age)]` selects columns via field sections.

```
use std/df

df.read_csv "sales.csv" ^
  | df.filter (.amount > 1000)
  | df.group_by [(.region)]
  | df.agg {total: df.sum (.amount)  avg: df.mean (.amount)  n: df.count}
  | df.sort_by [(.total)] | rev
  | df.take 10
  | df.each (row) $echo "{row.region}: {row.total}"
```

Backend: `polars` (in reference/).

## std/db — Embedded Database (SQLite + DuckDB)

Embedded databases for persistence and analytical queries. Two backends: SQLite for transactional/OLTP, DuckDB for analytical/OLAP.

```
open path                     -- Conn ^ DbErr (SQLite)
open_duck path                -- Conn ^ DbErr (DuckDB, also reads CSV/Parquet directly)
close conn                    -- () ^ DbErr

exec conn sql                 -- Int ^ DbErr (affected row count)
query conn sql                -- lazy [{...}] ^ DbErr (rows as records)
query_one conn sql            -- Maybe {...} ^ DbErr (first row or None)

prepare conn sql              -- Stmt ^ DbErr (prepared statement)
bind params stmt              -- Stmt (bind positional params: [val] or named: %{})
run stmt                      -- Int ^ DbErr
fetch stmt                    -- lazy [{...}] ^ DbErr

begin conn                    -- () ^ DbErr
commit conn                   -- () ^ DbErr
rollback conn                 -- () ^ DbErr
transaction conn f            -- a ^ DbErr (run f in transaction, auto commit/rollback)

tables conn                   -- [Str] ^ DbErr
table_info conn name          -- [{name col_type nullable}] ^ DbErr
```

SQL interpolation uses `{expr}` for parameterized values (safe from injection):

```
use std/db

conn = db.open "app.db" ^
defer () db.close conn
db.exec conn "CREATE TABLE IF NOT EXISTS logs (ts TEXT, level TEXT, msg TEXT)" ^
db.exec conn "INSERT INTO logs VALUES ({time.now () | to_str}, 'info', {msg})" ^
recent = db.query conn "SELECT * FROM logs ORDER BY ts DESC LIMIT 10" ^
recent | each (row) $echo "[{row.level}] {row.msg}"
```

DuckDB for analytics (reads CSV/Parquet directly, no ETL):

```
duck = db.open_duck ":memory:" ^
results = db.query duck "SELECT region, SUM(amount) as total FROM read_csv('sales.csv') GROUP BY region ORDER BY total DESC" ^
```

Backend: `rusqlite` for SQLite, `duckdb` crate for DuckDB.

## std/num — Numerical Arrays (ndarray)

Contiguous, typed numerical arrays for vectorized computation. Distinct from lists — these are SIMD-friendly, homogeneous, and support element-wise operations.

```
from_list xs                  -- Array (from [Int] or [Float])
zeros n                       -- Array (n zeros)
ones n                        -- Array (n ones)
range start stop step         -- Array (evenly spaced values)
linspace start stop n         -- Array (n evenly spaced values)

add a b                       -- Array (element-wise addition)
sub a b                       -- Array
mul a b                       -- Array
div a b                       -- Array
scale s arr                   -- Array (multiply all by scalar)

dot a b                       -- Float (dot product)
norm arr                      -- Float (L2 norm)
normalize arr                 -- Array (unit vector)

sum arr                       -- Float
mean arr                      -- Float
median arr                    -- Float
std_dev arr                   -- Float
variance arr                  -- Float
min_val arr                   -- Float
max_val arr                   -- Float
percentile p arr              -- Float (p in 0..100)
histogram n arr               -- [(Float Float)] (n bins, (edge count) pairs)

sort arr                      -- Array
argsort arr                   -- [Int] (indices that would sort)
cumsum arr                    -- Array (cumulative sum)
diff arr                      -- Array (pairwise differences)
rolling_mean n arr            -- Array (rolling average, window n)
correlation a b               -- Float (Pearson correlation)

len arr                       -- Int
get i arr                     -- Float
slice from to arr             -- Array
to_list arr                   -- [Float]
```

```
use std/num

signal = num.from_list [1.0 3.0 2.0 5.0 4.0 6.0]
smoothed = signal | num.rolling_mean 3
trend = signal | num.diff | num.mean
corr = num.correlation signal1 signal2
pct = num.percentile 95 latencies
```

Backend: `ndarray`.

## std/ml — ML Inference (candle)

Local model inference for embeddings, classification, and generation. No training — inference only. Models loaded from ONNX or safetensors format.

```
load path                     -- Model ^ MlErr (load ONNX or safetensors model)
load_with opts path           -- Model ^ MlErr (opts: {device: "cpu"|"cuda"  threads: Int})

embed model text              -- Array ^ MlErr (text -> embedding vector)
embed_batch model texts       -- [Array] ^ MlErr (batch embedding)
similarity a b                -- Float (cosine similarity between two Arrays)

classify model text           -- {label: Str  score: Float} ^ MlErr
classify_batch model texts    -- [{label score}] ^ MlErr

generate model prompt         -- Str ^ MlErr
generate_with opts model prompt -- Str ^ MlErr (opts: {max_tokens temperature top_p})

tokenize model text           -- [Str] ^ MlErr
token_count model text        -- Int ^ MlErr
```

```
use std/ml
use std/num

model = ml.load "models/all-MiniLM-L6-v2.onnx" ^
docs = ["error in auth module" "login failed" "disk space low" "auth timeout"]
embeddings = docs | pmap (d) ml.embed model d

query = ml.embed model "authentication problems" ^
ranked = docs
  | zip embeddings
  | map (doc emb) {doc  score: num.similarity query emb}
  | sort_by (.score) | rev
  | take 3
ranked | each (r) $echo "{r.score | fmt.fixed 3}: {r.doc}"
```

Backend: `candle-core` + `candle-transformers` (Hugging Face), or `ort` (ONNX Runtime bindings).

## std/plot — Terminal/SVG Charts (charming)

Visualization for the observe-iterate workflow. Two output modes: terminal (Unicode block charts for quick inspection) and SVG (for reports/sharing).

```
bar labels values             -- Chart
bar_h labels values           -- Chart (horizontal)
line xs ys                    -- Chart
scatter xs ys                 -- Chart
hist n data                   -- Chart (histogram with n bins)
pie labels values             -- Chart
heatmap matrix                -- Chart

title text chart              -- Chart (add title)
x_label text chart            -- Chart
y_label text chart            -- Chart
legend pos chart              -- Chart (pos: "top" | "bottom" | "right")
size w h chart                -- Chart (set dimensions)

render chart                  -- Str (terminal Unicode output)
render_svg chart              -- Str (SVG markup)
write_svg path chart          -- () ^ IoErr
show chart                    -- () (print to terminal)
```

```
use std/plot
use std/df

data = df.read_csv "metrics.csv" ^
by_day = data
  | df.group_by [(.date)]
  | df.agg {requests: df.sum (.count)  errors: df.sum (.errors)}
  | df.to_records

plot.bar (by_day | map (.date)) (by_day | map (.requests))
  | plot.title "Daily Requests"
  | plot.x_label "Date"
  | plot.show
```

Backend: `charming` (in reference/) for SVG, custom terminal renderer for Unicode block output.

## Cross-References

- Agent ecosystem modules: [stdlib-agents.md](stdlib-agents.md) (std/agent, std/mcp, std/ctx, std/md)
- Core stdlib modules: [stdlib-modules.md](stdlib-modules.md)
- Built-in functions and conventions: [stdlib.md](stdlib.md)
- Stdlib loader design: [impl-stdlib.md](../design/impl-stdlib.md)
- Implementation phases: [implementation-phases.md](../design/implementation-phases.md) (Phase 11)
- Polars reference: `reference/polars/`
- Charming reference: `reference/charming/`
