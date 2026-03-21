# Backend Trait Design Patterns

## 1. Backend Trait Design

### What Goes in the Trait

A backend trait should define the **minimal surface area** needed to abstract over implementations. Guidelines from the ecosystem:

**Methods**: Only operations that differ between backends. Common logic stays in the caller or a wrapper type (Go's `sql.DB` handles connection pooling; the `driver.Driver` interface only handles raw connections). Tower's `Service` has exactly two methods: `poll_ready` and `call`.

**Associated types**: Use them for backend-specific return types. Diesel's `Backend` trait has `QueryBuilder`, `RawValue`, and `BindCollector` as associated types -- each backend provides its own types for these. wgpu's `Api` trait uses associated types for `Instance`, `Adapter`, `Device`, `Queue`, etc.

**Error types**: Three approaches ranked by flexibility:
1. **Concrete error enum** (lx's approach): `Result<Value, LxError>`. Simple, unified, but backends can't express backend-specific errors without string encoding.
2. **Associated error type**: `type Error: std::error::Error + Send + Sync`. Each backend defines its own error. Tower uses this -- `Service::Error` is an associated type.
3. **Trait object error**: `Result<T, Box<dyn std::error::Error + Send + Sync>>`. Maximum flexibility, minimum type information for the caller.

**Supertraits**: `Send + Sync` are required if the trait object will be shared across threads (lx's case -- all 9 traits require `Send + Sync`). The wgpu HAL traits additionally require `'static`.

- Source: [tower Service trait](https://docs.rs/tower-service/latest/tower_service/trait.Service.html)
- Source: [Diesel Backend trait](https://docs.rs/diesel/latest/diesel/backend/trait.Backend.html)

### What Stays Concrete

- Connection pooling, retry logic, timeout management (Go's `sql.DB`)
- Validation and resource tracking (wgpu-core sits between the public API and the HAL)
- Caching, buffering, batching (these are cross-cutting concerns, not backend-specific)
- Configuration parsing (backends receive parsed config, don't parse it themselves)

### lx's Current Trait Design

lx's 9 backend traits are well-designed for their purpose:

```rust
pub trait ShellBackend: Send + Sync {
    fn exec(&self, cmd: &str, span: Span) -> Result<Value, LxError>;
    fn exec_capture(&self, cmd: &str, span: Span) -> Result<Value, LxError>;
}
```

Each trait is narrow (1-4 methods), uses a unified error type (`LxError`), and passes a `Span` for error attribution. The `&self` receiver with `Send + Sync` bounds enables shared ownership via `Arc<dyn Backend>`.

---

## 2. Default Implementations

### The "Works Out of the Box" Pattern

Every pluggable system needs a default backend that provides useful behavior without configuration. Patterns from the ecosystem:

**Rust log crate**: No default logger. If no logger is set, `log!` macros are no-ops (effectively a built-in NullHandler). Libraries must not set a logger -- only the binary entry point should.

**SLF4J 2.x**: If no provider is found on the classpath, defaults to NOP (no-operation) logger. This is a deliberate design choice -- silence is preferable to crashes.

**Go database/sql**: No default driver. You must explicitly import a driver package.

**lx's approach**: `RuntimeCtx::default()` provides a fully-functional set of backends:

| Trait | Default Implementation | Behavior |
|-------|----------------------|-----------|
| AiBackend | ClaudeCodeAiBackend | Delegates to Claude Code CLI |
| EmitBackend | StdoutEmitBackend | Prints to stdout |
| HttpBackend | ReqwestHttpBackend | Makes HTTP requests via reqwest |
| ShellBackend | ProcessShellBackend | Executes shell commands via std::process |
| YieldBackend | StdinStdoutYieldBackend | Reads from stdin, writes to stdout |
| LogBackend | StderrLogBackend | Logs to stderr |
| UserBackend | NoopUserBackend | No-op (no user interaction in CLI mode) |
| PaneBackend | YieldPaneBackend | Delegates to yield mechanism |
| EmbedBackend | VoyageEmbedBackend | Uses Voyage API for embeddings |

This is the right approach for an opinionated language runtime: the defaults work for the CLI use case, and other host environments (desktop, mobile) swap in their own implementations.

### Relationship Between Default and Specialized Backends

The key design principle: **defaults handle the common case; specialized backends handle host-specific or restricted cases**. The default should never be the "lowest common denominator" -- it should be the best implementation for the primary use case.

SQLAlchemy demonstrates this well: the default dialect for a PostgreSQL URL uses psycopg2 (the most popular driver), but users can specify `postgresql+asyncpg://` to use an async driver instead. The default is opinionated, not minimal.

---

## 3. Deny/Restrict Backends

### Capability-Based Restriction

lx implements capability-based security through backend trait implementations that refuse operations:

```rust
pub struct DenyShellBackend;

impl ShellBackend for DenyShellBackend {
    fn exec(&self, _cmd: &str, _span: Span) -> Result<Value, LxError> {
        Ok(Value::Err(Box::new(Value::Str(Arc::from(
            "shell access denied by sandbox policy",
        )))))
    }
}
```

Five Deny backends exist: `DenyShellBackend`, `DenyHttpBackend`, `DenyAiBackend`, `DenyPaneBackend`, `DenyEmbedBackend`. They return `Value::Err` with descriptive messages rather than panicking.

### The RestrictedShellBackend Pattern

lx also has a partial-deny backend that allows specific commands:

```rust
pub struct RestrictedShellBackend {
    pub inner: Arc<dyn ShellBackend>,
    pub allowed_cmds: Vec<String>,
}
```

This is a **decorator** that wraps a real backend and filters operations. The inner backend handles allowed commands; denied commands return an error. This is structurally identical to:
- Deno's `--allow-run=git,npm` flag (allow specific subprocesses)
- Java's SecurityManager (deprecated but conceptually similar)
- Firewall rules (default-deny with allowlist)

### Comparison with Deno's Permission Model

Deno implements capability-based security at the runtime level with flags:

| Deno Flag | lx Equivalent |
|-----------|---------------|
| `--allow-net` / `--deny-net` | `ReqwestHttpBackend` / `DenyHttpBackend` |
| `--allow-run` / `--deny-run` | `ProcessShellBackend` / `DenyShellBackend` |
| `--allow-read` / `--deny-read` | No filesystem backend (shell handles it) |
| `--allow-env` / `--deny-env` | No env backend (shell handles it) |

Key differences:
- **Deno**: Permissions are CLI flags checked at runtime by the engine. Deny flags take precedence over allow flags. Granularity is per-domain/per-path.
- **lx**: Permissions are backend implementations set at construction time. The sandbox scope replaces backends wholesale. Granularity is per-capability (shell, HTTP, AI, etc.) with `RestrictedShellBackend` allowing per-command granularity.

lx's approach is more flexible because backends are first-class objects -- a sandboxed subprocess can have different permissions than its parent by constructing a different `RuntimeCtx`.

- Source: [Deno Security and Permissions](https://docs.deno.com/runtime/fundamentals/security/)
- Source: [Capability-based security](https://en.wikipedia.org/wiki/Capability-based_security)

### Design Recommendations

1. **Return errors, don't panic**: Deny backends should return typed errors the program can handle, not crash the runtime. lx does this correctly with `Value::Err`.
2. **Consistent error messages**: Include which capability was denied and why. lx's messages ("shell access denied by sandbox policy") are good.
3. **Deny by default for security-sensitive operations**: In sandboxed contexts, start with all Deny backends and selectively enable capabilities. lx's `sandbox_scope.rs` does this.

---

## 4. Backend Composition

### Decorator/Wrapper Pattern

The most powerful composition pattern is wrapping a backend with additional behavior. Examples:

**Tower's Layer + Service**: The canonical Rust example. Each Layer wraps a Service, adding behavior before/after the inner call:

```
Request -> Timeout -> RateLimit -> LoadBalance -> InnerService
                                                      |
Response <- Timeout <- RateLimit <- LoadBalance <- Response
```

`ServiceBuilder::new().timeout(Duration::from_secs(5)).rate_limit(100, Duration::from_secs(1)).service(inner)` composes layers.

- Source: [Tower docs](https://docs.rs/tower)

**lx's RestrictedShellBackend**: Already demonstrates this pattern -- it wraps an inner `ShellBackend` and adds access control.

**Potential lx extensions**:
- `LoggingShellBackend { inner: Arc<dyn ShellBackend>, log: Arc<dyn LogBackend> }` -- logs every command before/after execution
- `CachingHttpBackend { inner: Arc<dyn HttpBackend>, cache: HashMap<String, Value> }` -- caches GET responses
- `RetryAiBackend { inner: Arc<dyn AiBackend>, max_retries: u32 }` -- retries on transient failures
- `MeteredBackend<B: Backend> { inner: B, metrics: Arc<Metrics> }` -- counts calls, measures latency

### Layered Backend Architecture

wgpu demonstrates a three-layer composition:
1. **Public API** (safe, validated) -- wgpu
2. **Core logic** (validation, tracking) -- wgpu-core
3. **Backend interface** (unsafe, raw) -- wgpu-hal

Go's `database/sql` demonstrates a two-layer composition:
1. **User-facing type** (pooling, retries) -- `sql.DB`
2. **Backend interface** (raw connections) -- `driver.Driver`

lx currently has a single-layer design: the backend trait is both the public interface and the implementation interface. If backend composition grows complex, consider splitting into:
1. **Runtime API** (what lx programs call) -- the current trait definitions
2. **Backend SPI** (what implementors provide) -- potentially simpler traits with less lx-specific types

---

## 5. Backend Discovery and Registration

### Compile-Time Discovery

**Rust generics**: The backend is a type parameter resolved at compile time. Diesel uses `Backend` as a trait bound. wgpu uses feature flags to conditionally compile backends. Zero runtime overhead but requires recompilation to switch backends.

**Rust feature flags**: `cfg(feature = "vulkan")` gates backend compilation. wgpu, Diesel, and SQLx all use this approach. The Cargo.toml specifies which backends to include.

### Runtime Discovery

**Java SPI**: `ServiceLoader` scans `META-INF/services/` files at runtime. JDBC, SLF4J 2.x, and many frameworks use this. Providers can be added by dropping JARs on the classpath.

**Go init() registration**: Drivers call `sql.Register("postgres", &Driver{})` in their `init()` function, triggered by blank imports. The global registry is a `map[string]driver.Driver`.

**Python entry points**: `setuptools` entry points allow packages to register themselves. SQLAlchemy discovers third-party dialects via the `sqlalchemy.dialects` entry point group. `importlib.metadata.entry_points()` provides discovery.

### Configuration-Driven Selection

**Spring profiles**: `@Profile("production")` annotations and `spring.profiles.active` property select which beans are instantiated.

**Terraform**: The `provider` block in HCL configuration names the provider and version. The Terraform Registry resolves it to a binary download.

**SQLAlchemy**: The database URL scheme (`postgresql://`, `mysql://`, `sqlite:///`) determines which dialect to instantiate.

### lx's Approach: Constructor Injection

lx uses neither discovery nor registration. Backends are passed directly at construction time via `RuntimeCtx`:

```rust
RuntimeCtx {
    ai: Arc::new(ClaudeCodeAiBackend),
    shell: Arc::new(DenyShellBackend),
    http: Arc::new(ReqwestHttpBackend),
    // ...
}
```

This is the simplest correct approach for lx's use case: the host environment (CLI, desktop app, mobile app) knows which backends to use at compile time. There's no need for plugin discovery because lx is embedded in Rust applications, not loaded as a standalone interpreter with arbitrary plugins.

---

## 6. Testing with Mock Backends

### Why Backend Traits Enable Testing

Backend traits are the primary mechanism for testing without real I/O. Every trait in lx's system can be implemented with a mock that:
- Records calls for assertion
- Returns predetermined values
- Simulates errors
- Tracks call order and arguments

### Mock Implementation Patterns

**Recording backend**: Stores all calls for later assertion:

```rust
struct RecordingShellBackend {
    calls: Mutex<Vec<String>>,
}

impl ShellBackend for RecordingShellBackend {
    fn exec(&self, cmd: &str, _span: Span) -> Result<Value, LxError> {
        self.calls.lock().push(cmd.to_string());
        Ok(Value::Str(Arc::from("mock output")))
    }
}
```

**Scripted backend**: Returns predetermined responses in order:

```rust
struct ScriptedAiBackend {
    responses: Mutex<VecDeque<Value>>,
}

impl AiBackend for ScriptedAiBackend {
    fn prompt(&self, _text: &str, _opts: &AiOpts, _span: Span) -> Result<Value, LxError> {
        Ok(self.responses.lock().pop_front().unwrap_or(Value::Nil))
    }
}
```

**Error-injecting backend**: Simulates failures for error handling tests:

```rust
struct FailingHttpBackend {
    fail_count: AtomicU32,
    inner: Arc<dyn HttpBackend>,
}
```

### Ecosystem Mock Tools

**mockall** (Rust): Procedural macro that generates mock implementations from trait definitions. Supports expectations, return value scripting, call counting, and argument matching. Works with `#[async_trait]` via `#[automock]`.

**double** (Rust): Simpler mock library inspired by googlemock. Tracks call arguments and lets you set return values at test time.

**Manual mocks**: For backend traits with few methods (like lx's), hand-written mocks are often simpler than macro-generated ones. The `DenyShellBackend` is already a production mock -- it's a test double that happens to be useful in production for sandboxing.

- Source: [mockall docs](https://docs.rs/mockall/latest/mockall/)
- Source: [Mocking in Rust](https://blog.logrocket.com/mocking-rust-mockall-alternatives/)

### lx's Testing Advantage

lx's backend trait design is well-suited for testing because:
1. All backends are behind `Arc<dyn Trait>`, so mocks are trivially substitutable
2. `RuntimeCtx` holds all backends in one struct, making it easy to construct test configurations
3. The `Deny*Backend` types serve double duty as test mocks (for testing sandbox enforcement) and production security mechanisms
4. No global state -- backends are passed through the context, so tests don't interfere with each other

---

## 7. Dependency Injection Patterns

### Constructor Injection (lx's Approach)

lx uses constructor injection: backends are passed to `RuntimeCtx` at construction time. This is the simplest and most explicit pattern:

```rust
let ctx = RuntimeCtx {
    shell: Arc::new(ProcessShellBackend),
    http: Arc::new(DenyHttpBackend),
    // ... all backends specified explicitly
};
```

**Advantages**: No hidden dependencies, no global state, easy to test, clear ownership.
**Disadvantages**: Verbose when many backends need to be specified (mitigated by `Default` impl).

Tower uses the same pattern: `ServiceBuilder` constructs a service by wrapping layers around an inner service. Spring's constructor injection (`@Autowired` on constructors) is the same concept with framework support.

### Service Locator

A global registry of backends, looked up by name or type at runtime:

- **Java JNDI**: `InitialContext.lookup("java:comp/env/jdbc/myDB")` retrieves a DataSource
- **Go database/sql**: `sql.Open("postgres", connStr)` looks up a registered driver by name
- **Python importlib**: `importlib.import_module("my_backend")` loads a module dynamically

Service locators hide dependencies (the caller doesn't declare what it needs) and make testing harder (must set up the global registry). They're appropriate when the set of backends is open-ended (plugin systems) but not for a fixed set of well-known capabilities (lx's case).

### Inversion of Control Containers

Spring's IoC container, Guice, and Dagger 2 manage object creation and wiring:

1. **Registration**: Backends are registered as beans/modules/components
2. **Resolution**: The container resolves the dependency graph and creates objects in the correct order
3. **Injection**: Dependencies are passed to constructors, setters, or fields

Spring detects circular dependencies at startup. Dagger 2 detects them at compile time. Wire (Go) generates code that makes the dependency graph explicit.

### Compile-Time DI: Generics vs Dynamic Dispatch

**Static dispatch (generics)**:
```rust
struct Runtime<S: ShellBackend, H: HttpBackend> {
    shell: S,
    http: H,
}
```
Zero-cost abstraction via monomorphization. The compiler generates a specialized `Runtime` for each combination of backends. No vtable overhead. But: every function that uses `Runtime` must be generic over backend types, leading to generic parameter explosion and longer compile times.

**Dynamic dispatch (trait objects)**:
```rust
struct Runtime {
    shell: Arc<dyn ShellBackend>,
    http: Arc<dyn HttpBackend>,
}
```
One function handles all backend combinations. Vtable lookup per call (~2-3ns overhead). lx uses this approach, which is correct for an interpreter where the overhead of vtable dispatch is negligible compared to interpretation overhead.

Performance comparison: benchmarks show static dispatch can be 3x faster than dynamic dispatch in tight loops (64ms vs 216ms for 20M iterations). But for backend calls that involve I/O (HTTP requests, shell commands, AI prompts), the vtable overhead is unmeasurable.

- Source: [Rust Static vs Dynamic Dispatch](https://softwaremill.com/rust-static-vs-dynamic-dispatch/)
- Source: [Zero-Cost Abstractions](https://monomorph.is/posts/zero-cost-abstractions/)

### Runtime DI

Selecting backends based on configuration or environment:

```rust
let shell: Arc<dyn ShellBackend> = match config.shell_policy {
    ShellPolicy::Allow => Arc::new(ProcessShellBackend),
    ShellPolicy::AllowList(cmds) => Arc::new(RestrictedShellBackend {
        inner: Arc::new(ProcessShellBackend),
        allowed_cmds: cmds,
    }),
    ShellPolicy::Deny => Arc::new(DenyShellBackend),
};
```

lx's `sandbox_scope.rs` already does this -- it reads the sandbox policy and constructs the appropriate backend. This is runtime DI without a framework, which is the right level of complexity for lx.

---

## 8. Cross-Cutting Concerns

### Backend Lifecycle

**Initialization**: Backends should be cheap to construct and defer expensive setup (connection establishment, authentication) to first use. SQLAlchemy's Engine creates connections lazily via a pool. Tokio's Runtime spawns worker threads on `new()` but the I/O driver starts on first use.

**Connection pooling**: For backends that manage connections (HTTP, database, AI API), the backend should own the pool. lx's `ReqwestHttpBackend` likely holds a `reqwest::Client`, which maintains a connection pool internally. This is the right approach -- the backend owns its resources.

**Graceful shutdown**: Backends that hold resources (open connections, background tasks) need cleanup. Patterns:
- **Drop trait**: Rust's RAII handles resource cleanup when the backend is dropped
- **Explicit shutdown method**: `shutdown(&self) -> Result<()>` for async cleanup that can't happen in `Drop`
- **Cancellation tokens**: `tokio_util::sync::CancellationToken` signals background tasks to stop
- **Shutdown timeout**: `tokio::runtime::Runtime::shutdown_timeout()` waits up to a duration for tasks to complete

lx's backends take `&self` (shared reference), so they're designed for long-lived shared use. Shutdown happens when the `RuntimeCtx` is dropped and all `Arc` reference counts reach zero.

- Source: [Tokio Graceful Shutdown](https://tokio.rs/tokio/topics/shutdown)
- Source: [async_shutdown crate](https://docs.rs/async-shutdown)

### Error Handling Across Backends

Three strategies for unifying errors across backend implementations:

**1. Single error type (lx's approach)**:
```rust
fn exec(&self, cmd: &str, span: Span) -> Result<Value, LxError>;
```
All backends return `LxError`. Backend-specific errors are converted at the boundary. This is simple and consistent but loses type information.

**2. thiserror enum with variants per backend**:
```rust
#[derive(thiserror::Error)]
enum BackendError {
    #[error("shell: {0}")] Shell(#[source] ShellError),
    #[error("http: {0}")] Http(#[source] reqwest::Error),
    #[error("ai: {0}")] Ai(#[source] AiError),
}
```
Preserves error provenance. Callers can match on variants. Used by many Rust libraries.

**3. anyhow::Error for erasure**:
```rust
fn exec(&self, cmd: &str) -> Result<Value, anyhow::Error>;
```
Maximum flexibility. Any error type is accepted. Callers use `downcast_ref()` to recover specific types. Best for application code, not library code.

lx's choice of a single `LxError` type is appropriate because the interpreter needs to present errors to lx programs as `Value::Err`, not as Rust types. The error type is a presentation boundary, not a programmatic boundary.

- Source: [Error Handling in Rust](https://lpalmieri.com/posts/error-handling-rust/)
- Source: [thiserror](https://docs.rs/thiserror/latest/thiserror/)

### Async Backend Traits

lx's backend traits are synchronous (`fn exec(&self, ...) -> Result<...>`), not async. This is a deliberate design choice with tradeoffs:

**Current approach (sync traits + blocking)**:
- Backend methods block the calling thread
- The `tokio_runtime` field in `RuntimeCtx` allows backends to internally use `block_on()` for async operations
- Simple to implement and reason about
- Works well because lx's interpreter is single-threaded per program

**Alternative (async traits)**:
```rust
#[async_trait]
pub trait ShellBackend: Send + Sync {
    async fn exec(&self, cmd: &str, span: Span) -> Result<Value, LxError>;
}
```

`async_trait` transforms async methods into `Pin<Box<dyn Future + Send>>`, adding a heap allocation per call. Rust 1.75+ supports async fn in traits natively, but trait objects with async methods require the `trait_variant` crate or manual desugaring.

The key consideration: if lx's interpreter becomes async (to support concurrent agent spawning without thread-per-agent), all backend traits would need to become async. This is a one-way door -- going from sync to async traits is a breaking change for all implementations.

- Source: [async_trait docs](https://docs.rs/async-trait/latest/async_trait/)
- Source: [Rust RFC 3185 - Static async fn in trait](https://rust-lang.github.io/rfcs/3185-static-async-fn-in-trait.html)

### Configuration

How backends receive their configuration:

**Environment variables**: `env_logger` reads `RUST_LOG`. Simple, works everywhere, but limited expressiveness.

**Config struct**: Diesel's `Backend` trait doesn't handle config -- it's passed to `Connection::establish()`. This separates "what the backend does" from "how it's configured."

**URL scheme**: SQLAlchemy's `create_engine("postgresql+asyncpg://host/db")` encodes both the dialect and driver in the URL.

**Builder pattern**: Tokio's `Runtime::builder()` provides a fluent API for configuration. Reqwest's `Client::builder()` does the same.

lx's backends receive no configuration through the trait -- they're constructed with their config and the trait methods receive only per-call parameters. This is the cleanest separation. If a backend needs config, it takes it as constructor arguments:

```rust
RestrictedShellBackend {
    inner: Arc::new(ProcessShellBackend),
    allowed_cmds: vec!["git".into(), "npm".into()],
}
```

### Observability

Adding logging, metrics, and tracing to backends without changing the trait:

**Decorator pattern**: Wrap backends with instrumentation:
```rust
struct InstrumentedBackend<B> {
    inner: B,
    metrics: Arc<Metrics>,
}
```

**Tower's approach**: `tower-http`'s `TraceLayer` wraps any Service with OpenTelemetry-compatible tracing. The inner service is unaware of instrumentation.

**Go's approach**: `http.Handler` middleware wraps handlers with logging/metrics. Standard library provides `httputil.ReverseProxy` for wrapping.

**Aspect-oriented**: Spring AOP can add logging/metrics to any bean method via annotations (`@Timed`, `@Logged`) without modifying the bean class.

For lx, the decorator pattern is the right choice. A `LoggingShellBackend` wrapping `ProcessShellBackend` could log every command without changing either the trait or the inner backend.

### Versioning and Compatibility

How backend trait changes are managed:

**Additive evolution**: Add new methods with default implementations. Existing backends continue to work. Go's `database/sql/driver` added `QueryerContext` alongside `Queryer` -- new drivers implement both, old drivers only implement `Queryer`, and the `sql.DB` wrapper checks for the new interface at runtime.

**Protocol versioning**: Terraform's plugin protocol has versions 5 and 6. Providers declare which version they implement. Core supports multiple versions simultaneously.

**Sealed traits**: Prevent external implementations to preserve freedom to add methods. Diesel's `Backend` trait is effectively sealed by requiring implementation of internal types.

**Deprecation path**: SLF4J moved from `StaticLoggerBinder` (1.x) to `ServiceLoader` (2.x) over years, maintaining backward compatibility through the transition.

**lx's situation**: Since lx's backends are all defined in the same codebase and there are no external implementors, trait evolution is straightforward -- change the trait and update all implementations. No backward compatibility concerns needed (per CLAUDE.md: "Do not worry about backward compatibility").

---

## 9. Case Studies

### Case Study 1: Tokio's Runtime

Tokio's `Runtime` struct bundles an I/O driver, task scheduler, and timer:

```rust
let rt = tokio::runtime::Builder::new_multi_thread()
    .worker_threads(4)
    .enable_all()
    .build()?;
```

Key design decisions:
- **Single entry point**: `Runtime::new()` creates all subsystems. No separate driver/scheduler/timer initialization.
- **Handle for sharing**: `rt.handle()` returns a `Handle` that can be cloned and sent to other threads for spawning tasks, without preventing runtime shutdown.
- **Context via thread-local**: `rt.enter()` sets a thread-local so `tokio::spawn()` works without explicit runtime references.
- **Graceful shutdown**: `rt.shutdown_timeout(Duration::from_secs(5))` signals all tasks and waits. `rt.shutdown_background()` detaches without waiting.

Relevance to lx: `RuntimeCtx` is analogous to Tokio's `Runtime` -- a single struct holding all subsystems. lx already stores `tokio_runtime: Arc<tokio::runtime::Runtime>` in `RuntimeCtx`, using Tokio as the async execution backend while providing its own backend traits for domain-specific capabilities.

- Source: [tokio::runtime::Runtime docs](https://docs.rs/tokio/latest/tokio/runtime/struct.Runtime.html)

### Case Study 2: SLF4J + Logback/Log4j2

SLF4J is the gold standard of facade + backend separation:

**Architecture**:
1. `slf4j-api` defines `Logger`, `LoggerFactory`, `Marker`, `MDC` -- the API surface
2. A **provider** JAR implements `SLF4JServiceProvider` and provides a `LoggerFactory` that creates backend-specific `Logger` instances
3. Logback implements `org.slf4j.Logger` directly (zero-overhead native binding)
4. Log4j2 provides `log4j-slf4j2-impl` as a bridge JAR

**Binding evolution**:
- SLF4J 1.x: `StaticLoggerBinder` class looked up via classloader. Only one binding allowed on classpath.
- SLF4J 2.x: `ServiceLoader<SLF4JServiceProvider>` for standard discovery. Multiple providers warned, first one used.

**Design principles**:
- Libraries depend on `slf4j-api` only, never on a binding
- Binary/application chooses the binding at deployment time
- Switching from Logback to Log4j2: swap one JAR, change no code
- No-binding defaults to NOP (not an error)

This maps directly to lx's architecture: `LogBackend` is lx's logging facade, `StderrLogBackend` is the default binding, and a desktop app could provide a `UiLogBackend` that routes to a log panel.

- Source: [SLF4J Manual](https://www.slf4j.org/manual.html)

### Case Study 3: Rust's log + env_logger / tracing

The `log` crate follows SLF4J's pattern exactly:

```rust
// Library code -- uses the facade
log::info!("Processing request {}", id);

// Binary code -- sets the backend
env_logger::init();  // or: tracing_subscriber::fmt::init();
```

The `Log` trait has three methods: `enabled()`, `log()`, `flush()`. A single global logger is set via `log::set_logger()` (returns error if already set) or `log::set_boxed_logger()`.

`tracing` extends this with spans (enter/exit) and structured fields. The `Subscriber` trait replaces `Log`. Layers compose subscribers:

```rust
tracing_subscriber::registry()
    .with(fmt_layer)       // formatting
    .with(filter_layer)    // level filtering
    .with(opentelemetry)   // export to Jaeger/Zipkin
    .init();
```

The `tracing-log` compatibility layer allows `log` events to be captured by `tracing` subscribers and vice versa.

- Source: [log crate](https://crates.io/crates/log)
- Source: [tracing crate](https://docs.rs/tracing)

### Case Study 4: SQLAlchemy

SQLAlchemy's dialect system demonstrates full database backend abstraction:

```python
engine = create_engine("postgresql://user:pass@localhost/mydb")
```

The URL determines:
1. **Dialect**: `postgresql` -> `PGDialect`
2. **DBAPI driver**: Default `psycopg2`, or explicit: `postgresql+asyncpg://`
3. **Connection parameters**: Host, port, database, credentials

The Dialect class hierarchy:
- `Dialect` (abstract base)
  - `PGDialect` (PostgreSQL)
    - `PGDialect_psycopg2` (psycopg2 driver)
    - `PGDialect_asyncpg` (asyncpg driver)
  - `MySQLDialect` (MySQL)
  - `SQLiteDialect` (SQLite)

Each dialect handles SQL compilation, type mapping, schema reflection, transaction control, and DBAPI integration. The same ORM code works across all backends -- only the URL changes.

Third-party dialects register via Python entry points:
```python
# setup.py or pyproject.toml
[project.entry-points."sqlalchemy.dialects"]
snowflake = "snowflake.sqlalchemy:dialect"
```

- Source: [SQLAlchemy Dialects](https://docs.sqlalchemy.org/en/20/dialects/)
- Source: [SQLAlchemy Engine Configuration](https://docs.sqlalchemy.org/en/20/core/engines.html)

### Case Study 5: React Renderers

React's reconciler/renderer split demonstrates backend abstraction for UI rendering:

**Reconciler** (shared): Manages the component tree, diffs virtual trees, computes minimal updates, handles state/effects/lifecycle. Uses the Fiber architecture for incremental rendering.

**Renderer** (pluggable): Implements a `HostConfig` object with ~30 methods:
- `createInstance(type, props)` -- create a host element
- `appendChild(parent, child)` -- attach child to parent
- `commitMount(instance, type, props)` -- post-mount side effects
- `commitUpdate(instance, type, oldProps, newProps)` -- apply prop changes
- `removeChild(parent, child)` -- detach child

**Production renderers**:
- `react-dom`: Creates/modifies DOM elements
- `react-native`: Creates/modifies native iOS/Android views
- `react-three-fiber`: Creates/modifies Three.js 3D scene objects
- `ink`: Creates/modifies terminal UI elements via Yoga layout + ANSI output

The reconciler doesn't know or care what the host elements are. It works with opaque "instances" that the renderer creates and manages. This is the same abstraction pattern as wgpu's core/HAL split and lx's interpreter/backend split.

- Source: [react-reconciler npm](https://www.npmjs.com/package/react-reconciler)
- Source: [Building a Custom React Renderer](https://blog.openreplay.com/building-a-custom-react-renderer/)
