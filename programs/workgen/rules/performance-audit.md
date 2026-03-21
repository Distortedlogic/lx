# Performance Audit

Every item below is a binary check — a violation either exists or it does not. Each check targets a specific performance anti-pattern in general Rust code. The audit checks each item across all `.rs` files and build configuration files in all crates.

## Memory — Heap Allocations

- **Collect into Vec for iterator/slice consumer** — iterator collected into Vec solely to pass to a function that accepts an iterator or slice.
  Grep: `.collect::<Vec`, `.as_slice()`, `.iter()`
- **Intermediate collection for single-pass aggregate** — (sum, count, any, all) that could use iterator combinators directly.
  Grep: `.collect::<Vec`, `.sum()`, `.count()`, `.any(`, `.all(`
- **Allocation inside hot loop** — could be hoisted before the loop and reused via `.clear()`.
  Grep: `Vec::new()`, `String::new()`, `HashMap::new()`, `.clear()`, `for .+ in `, `loop `
- **Clone for serialization** — when the serializer accepts a shared reference.
  Grep: `.clone()`, `serialize(`, `bincode::encode`, `serde_json::to_`
- **Missing with_capacity** — on Vec, HashMap, or String when the size is known or estimable.
  Grep: `Vec::new()`, `HashMap::new()`, `String::new()`, `with_capacity`
- **String concatenation in loop** — via `+` or `push_str` instead of pre-sized String with `with_capacity` or `join`.
  Grep: `push_str`, `format!`, `.join(`, `+= `, `String::new()`
- **Repeated to_string/to_owned on static data** — could use a reference or be computed once.
  Grep: `.to_string()`, `.to_owned()`, `&str`

## Memory — Data Structure Choice

- **HashMap keyed by small fixed enum** — instead of an array indexed by the enum discriminant.
  Grep: `HashMap<`, `enum `, `.get(`, `.insert(`
- **Owned String keys cloned per HashMap op** — instead of borrowed keys, interned strings, or integer IDs.
  Grep: `HashMap<String`, `.clone()`, `.to_string()`, `.insert(`
- **Vec of indices for membership** — instead of a bitset or boolean array.
  Grep: `Vec<usize>`, `.contains(`, `BitVec`, `bitvec`
- **Full sort for partial result** — (top-k, percentile, trimmed subset) where `select_nth_unstable` or a heap suffices.
  Grep: `.sort(`, `.sort_by(`, `.sort_unstable(`, `select_nth`, `BinaryHeap`
- **Vec of tuples vs struct-of-arrays** — where struct-of-arrays layout would be more cache-friendly for columnar access patterns.
  Grep: `Vec<(`, `Vec<[`
- **Nested HashMap** — where a composite key in a flat HashMap would reduce indirection and hashing overhead.
  Grep: `HashMap<`, `HashMap<.+HashMap`
- **HashSet for very small sets** — (fewer than ~16 elements) where linear scan on a small Vec is faster due to cache locality.
  Grep: `HashSet<`, `HashSet::new()`, `.contains(`
- **Repeated HashMap lookup for same key** — instead of using the entry API or a let binding.
  Grep: `.get(`, `.insert(`, `.entry(`, `HashMap`

## Memory — Ownership & Copying

- **Unnecessary clone where borrow suffices** — `.clone()` where a reference or borrow would suffice and the lifetime allows it.
  Grep: `.clone()`, `&`, `&mut `
- **Unnecessary Box/Arc heap indirection** — where stack allocation or direct ownership works.
  Grep: `Box<`, `Arc<`, `Box::new(`, `Arc::new(`
- **Vec of Box** — (e.g., `Vec<Box<T>>`) where `Vec<T>` directly would eliminate per-element heap indirection.
  Grep: `Vec<Box<`, `Box<dyn`
- **Full structure clone instead of in-place mutation** — where in-place mutation would work and avoid the allocation.
  Grep: `.clone()`, `let mut .+ = .+.clone()`

## Concurrency — Lock Contention

- **Lock per parallel iteration** — instead of batch accumulate-then-insert.
  Grep: `.lock()`, `.write()`, `.par_iter(`, `for .+ in `
- **Expensive computation under write lock** — compute outside, lock only for the write.
  Grep: `.write()`, `.lock()`, `RwLock`, `Mutex`
- **One-by-one insertion under lock** — instead of collecting into a local buffer and batch-inserting.
  Grep: `.lock()`, `.write()`, `.insert(`, `.push(`
- **Lock held across I/O or expensive computation** — hold the lock only for the critical section.
  Grep: `.lock()`, `.write()`, `.read()`, `await`, `File::`, `std::io`
- **Mutex where RwLock allows concurrent readers** — Mutex used where RwLock would allow concurrent readers.
  Grep: `Mutex<`, `RwLock<`, `parking_lot::Mutex`, `parking_lot::RwLock`
- **Arc\<Mutex\> where lock-free fits** — where a lock-free structure (DashMap, atomics, channels) fits the access pattern.
  Grep: `Arc<Mutex`, `Arc<RwLock`, `DashMap`, `AtomicU`, `AtomicI`, `channel(`

## Concurrency — Atomics & Channels

- **SeqCst where weaker ordering suffices** — Relaxed or Acquire/Release would be sufficient and faster.
  Grep: `SeqCst`, `Ordering::`, `Relaxed`, `Acquire`, `Release`, `AcqRel`
- **Unbounded channel without backpressure** — where bounded would provide backpressure and prevent unbounded memory growth.
  Grep: `unbounded_channel`, `bounded`, `channel(`, `mpsc::`
- **Locks where channels would eliminate contention** — shared mutable state protected by locks where message passing via channels would eliminate contention.
  Grep: `Arc<Mutex`, `Arc<RwLock`, `mpsc::`, `channel(`, `Sender`, `Receiver`

## Serialization — I/O Efficiency

- **Full buffer before deserialization** — instead of streaming directly from the reader.
  Grep: `read_to_end`, `read_to_string`, `bincode::decode`, `serde_json::from_str`, `BufReader`
- **Encoding to intermediate buffer** — before writing to output instead of serializing directly to the writer.
  Grep: `bincode::encode`, `serde_json::to_vec`, `serde_json::to_string`, `BufWriter`, `.write_all(`
- **Re-serialization of unchanged data** — on every save instead of caching the serialized form.
  Grep: `serialize(`, `encode(`, `to_vec(`, `to_string(`
- **Clone for serializable variant** — instead of deriving Serialize on the original type.
  Grep: `.clone()`, `Serialize`, `#[derive(`, `impl Serialize`
- **Missing BufReader/BufWriter** — on file I/O causing excessive syscalls.
  Grep: `File::open`, `File::create`, `BufReader`, `BufWriter`, `std::io`
- **Untuned compression level** — default level used without considering latency vs. size tradeoff.
  Grep: `zstd`, `compression_level`, `Encoder::new`, `Decoder::new`, `flate2`

## Serialization — Format

- **Text format for internal data exchange** — (JSON, TOML, CSV) where a binary format (bincode, MessagePack) would be faster and smaller.
  Grep: `serde_json`, `toml`, `serde_yaml`, `bincode`, `rmp_serde`, `ciborium`
- **Full structure deserialization for partial use** — when only a subset of fields is needed.
  Grep: `deserialize`, `decode(`, `from_reader`, `from_str`, `#[serde(skip`

## Algorithmic Complexity

- **Recursive computation without memoization** — on stable or immutable input structures.
  Grep: `fn .+\(.+\) ->`, `HashMap`, `memo`, `cache`, `recursive`
- **Linear scan for repeated membership checks** — (`.contains()`, `.find()`, `.position()`) where a HashSet or sorted collection with binary search would be O(1) or O(log n).
  Grep: `.contains(`, `.find(`, `.position(`, `HashSet`, `binary_search`
- **Collect for existence/count query** — full collection materialization (`.collect()`) to answer `.any()` or `.count()` that iterator combinators handle lazily.
  Grep: `.collect::<Vec`, `.any(`, `.count()`, `.all(`, `.find(`
- **Vec of indices for contiguous span** — where a Range represents the same contiguous span.
  Grep: `Vec<usize>`, `0..`, `Range`, `.len()`
- **O(n²) nested loop** — where an O(n log n) or O(n) algorithm exists for the same result.
  Grep: `for .+ in `, `.iter()`, `.contains(`, `nested`
- **Recomputation of derived values** — on every access that could be cached or precomputed once.
  Grep: `fn .+(&self)`, `.len()`, `.iter()`, `lazy_static`, `once_cell`, `LazyLock`
- **Stable sort where stability unnecessary** — `.sort_unstable()` is faster.
  Grep: `.sort()`, `.sort_by(`, `.sort_unstable(`, `.sort_unstable_by(`
- **Linear search in sorted data** — where binary search (`.binary_search()`, `partition_point`) would be O(log n).
  Grep: `.binary_search(`, `partition_point`, `.find(`, `.position(`, `.iter()`

## Numeric & Floating Point

- **Float equality without epsilon** — fragile and often incorrect.
  Grep: `== `, `f64`, `f32`, `abs()`, `EPSILON`, `epsilon`
- **Naive float summation** — without compensated summation (Kahan/Neumaier) accumulates rounding error.
  Grep: `+= `, `.sum()`, `f64`, `f32`, `kahan`, `compensated`
- **Division by zero without guard** — produces Infinity or panics on integer division.
  Grep: `/ `, `/ 0`, `.is_finite()`, `checked_div`
- **NaN/Infinity propagation** — without explicit handling, silently corrupts downstream computations.
  Grep: `.is_nan()`, `.is_finite()`, `.is_infinite()`, `NaN`, `f64`, `f32`
- **f64 where f32 suffices** — doubles memory bandwidth and cache footprint for no benefit.
  Grep: `f64`, `f32`, `as f64`, `as f32`

## Build Configuration

- **opt-level mismatch** — size-optimized (`opt-level = "z"` or `"s"`) on CPU-bound numerical work where `opt-level = 3` would be faster.
  Grep: `opt-level`, `[profile.release]`, `[profile.dev]`
- **Missing target-cpu=native** — in RUSTFLAGS for platform-specific deployment builds, misses SIMD and microarchitecture optimizations.
  Grep: `target-cpu`, `RUSTFLAGS`, `.cargo/config`
- **Unused heavy dependencies via feature flags** — increases compile time and binary size for no benefit.
  Grep: `features`, `[dependencies`, `default-features`
- **codegen-units > 1 in release** — prevents cross-module inlining and optimization.
  Grep: `codegen-units`, `[profile.release]`, `lto`

## Logging & Tracing

- **Expensive formatting in disabled log levels** — use lazy formatting or guard with `enabled!`.
  Grep: `tracing::debug!`, `tracing::trace!`, `log::debug!`, `log::trace!`, `format!(`, `enabled!`
- **Tracing spans in hot inner loops** — span creation and drop overhead dominates the loop body.
  Grep: `#[instrument`, `tracing::span!`, `info_span!`, `debug_span!`, `for .+ in `, `loop `

## Async Runtime

- **Blocking I/O on async thread pool** — without `spawn_blocking`.
  Grep: `spawn_blocking`, `block_on`, `std::fs::`, `std::io::`, `tokio::spawn`, `.await`
- **Lock held across .await** — blocks the async runtime thread.
  Grep: `.lock()`, `.write()`, `.read()`, `.await`, `MutexGuard`, `RwLockGuard`
- **Excessive task spawning** — for trivial work where batching or a single task with a loop would reduce scheduler overhead.
  Grep: `tokio::spawn`, `spawn(`, `JoinSet`, `FuturesUnordered`

## Dynamic Dispatch

- **dyn Trait in hot loops** — where enum dispatch or monomorphization via generics would eliminate vtable overhead.
  Grep: `dyn `, `Box<dyn`, `&dyn`, `impl Trait`, `enum`
- **Box\<dyn Trait\> allocated per call** — where a reusable allocation or enum variant would avoid repeated heap allocation.
  Grep: `Box<dyn`, `Box::new(`, `enum`, `impl `

## Random Number Generation

- **RNG init inside hot loop** — instead of creating once outside and reusing.
  Grep: `thread_rng()`, `from_entropy()`, `StdRng`, `ChaCha`, `SmallRng`, `for .+ in `, `loop `
- **Shared RNG contention across threads** — instead of per-thread or split RNG instances.
  Grep: `Arc<Mutex`, `thread_rng`, `SmallRng`, `SeedableRng`, `split(`, `Rng`
