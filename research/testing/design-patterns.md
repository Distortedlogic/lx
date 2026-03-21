# Testing Patterns and Design

How testing frameworks implement discovery, fixtures, assertions, mocking, property testing, snapshots, fuzzing, parametrization, isolation, BDD, and coverage.

## 1. Test Discovery

How frameworks find tests without manual registration.

### Naming conventions

The simplest discovery mechanism. The framework scans for files and functions matching a pattern.

| Framework | File pattern | Function/method pattern | Class pattern |
|-----------|-------------|------------------------|---------------|
| pytest | `test_*.py`, `*_test.py` | `test_*` | `Test*` (no `__init__`) |
| unittest | `test*.py` | `test*` methods | `TestCase` subclasses |
| Go testing | `*_test.go` | `func TestXxx`, `func BenchmarkXxx`, `func FuzzXxx`, `func ExampleXxx` | N/A |
| Rust | Any file in `tests/` (integration), any file in `src/` (unit) | `#[test]` annotated functions | N/A |
| Jest/Vitest | `*.test.js`, `*.spec.js`, `__tests__/*.js` | `test()`, `it()` calls | N/A |
| RSpec | `*_spec.rb` | `it` blocks inside `describe` | N/A |
| minitest | `test_*.rb` or any loaded file | `test_*` methods | `Minitest::Test` subclasses |
| ExUnit | `*_test.exs` | `test` macro invocations | Modules with `use ExUnit.Case` |
| busted | `*_spec.lua`, `*_test.lua` | `it()` blocks | N/A |

### Decorators and attributes

Some languages use explicit markers that double as discovery mechanisms:

- **Rust `#[test]`:** The compiler collects all functions with this attribute and generates a test harness. The attribute is removed from non-test builds via `#[cfg(test)]`.
- **Python `@pytest.mark.*`:** Not for discovery (naming handles that) but for filtering and categorization. `@pytest.mark.slow`, `@pytest.mark.parametrize`, etc.
- **C++ Google Test:** `TEST()` and `TEST_F()` macros register tests at static initialization time, before `main()` runs. No naming convention needed — the macro handles registration.

### Registration-based discovery

Some frameworks require explicit registration:

- **Lua (luaunit):** Tests are collected from the global table or passed explicitly to `LuaUnit.run()`
- **Go's `TestMain`:** Optional `func TestMain(m *testing.M)` intercepts the test runner, allowing custom setup before `m.Run()` executes discovered tests
- **Rust custom harness:** With `harness = false` in Cargo.toml, you write your own `main()` and manually invoke test functions

### Directory-based conventions

- **Rust:** `tests/` directory for integration tests (each file = separate crate), `src/` for unit tests
- **Go:** Test files live alongside source files in the same package (same directory), or in `_test` package for black-box testing
- **Elixir:** `test/` directory, `test/test_helper.exs` for bootstrap
- **pytest:** Configurable; typically `tests/` directory
- **Jest:** `__tests__/` directories or co-located `.test.js` files

---

## 2. Fixture Systems

Mechanisms for test setup, teardown, and shared state.

### xUnit setup/teardown (unittest, minitest, Go testify suite)

The original pattern from SUnit (Smalltalk) through JUnit:

```
setUpClass    → runs once before all tests in the class
setUp         → runs before each test
test_method   → the actual test
tearDown      → runs after each test
tearDownClass → runs once after all tests
```

**Strengths:** Simple mental model. Clear lifecycle.
**Weaknesses:** No composition — if tests need different combinations of resources, you end up with inheritance hierarchies or duplicated setup code. No dependency injection.

### pytest fixtures (dependency injection with scoping)

pytest's fixture system is the most sophisticated in any mainstream testing framework.

**Dependency injection:** Test functions declare what they need by naming fixtures as parameters. pytest inspects the signature, resolves the dependency graph, and injects values.

```python
@pytest.fixture
def db_connection():
    conn = create_connection()
    yield conn
    conn.close()

@pytest.fixture
def user(db_connection):
    return db_connection.create_user("test")

def test_user_name(user):
    assert user.name == "test"
```

**Scope hierarchy:**
- `function` (default): new instance per test
- `class`: shared across tests in a class
- `module`: shared across tests in a file
- `package`: shared across tests in a package
- `session`: shared across the entire test run

Higher-scoped fixtures execute before lower-scoped ones. A `function`-scoped fixture cannot request a `function`-scoped fixture from a different test — scoping enforces isolation.

**Yield fixtures:** The fixture yields its value, and code after `yield` runs as teardown. pytest executes teardown in reverse initialization order, guaranteeing correct cleanup sequencing.

**conftest.py:** Fixtures defined in `conftest.py` are available to all tests in the same directory and below without imports. Multiple `conftest.py` files at different directory levels create a hierarchy of available fixtures.

**Autouse:** `@pytest.fixture(autouse=True)` applies the fixture to every test in scope without requiring it as a parameter. Useful for things like database transaction rollback.

**Fixture factories:** Return a callable from the fixture for tests that need multiple instances with different configurations.

Sources: [pytest fixtures](https://docs.pytest.org/en/stable/how-to/fixtures.html), [Advanced fixture patterns](https://www.inspiredpython.com/article/five-advanced-pytest-fixture-patterns)

### Rust's test harness

Rust has no fixture system in the language. Common patterns:

- **Helper functions:** Called at the start of each test. No automatic teardown.
- **RAII guards:** Structs whose `Drop` implementation performs cleanup. Rust's ownership system guarantees teardown runs even on panic (unlike most languages where exceptions can skip teardown).
- **`#[ctor]`/`#[dtor]` crates:** For one-time global setup (e.g., initializing a logger).
- **rstest fixtures:** The `rstest` crate provides pytest-like `#[fixture]` functions injected into `#[rstest]` tests.

The absence of a built-in fixture system is deliberate: Rust's type system and RAII provide resource management guarantees that fixture systems in GC'd languages exist to approximate.

### ExUnit callbacks

```elixir
setup do
  user = create_user()
  {:ok, user: user}
end

test "user exists", %{user: user} do
  assert user.name == "test"
end
```

The map returned from `setup` is merged into the test context, which is pattern-matched in the test's argument. `setup_all` runs once per module for expensive resources.

### RSpec let/before/after

- `let` is lazily evaluated and memoized per example. Not assigned until first referenced.
- `let!` is eagerly evaluated before each example.
- `before(:each)` / `after(:each)` run around every example.
- `before(:all)` / `after(:all)` run once per group.
- `around(:each)` receives the example as a block, enabling patterns like wrapping each test in a database transaction.

### Mocha/Jest hooks

```javascript
describe('Feature', () => {
  beforeAll(() => { /* once before suite */ });
  beforeEach(() => { /* before each test */ });
  afterEach(() => { /* after each test */ });
  afterAll(() => { /* once after suite */ });

  it('works', () => { /* test */ });
});
```

Hooks execute in lifecycle order: `beforeAll` -> (`beforeEach` -> test -> `afterEach`) x N -> `afterAll`. Nested `describe` blocks create nested hook scopes.

---

## 3. Assertion Design

How frameworks report what went wrong when a test fails.

### Assert functions (unittest, Go, luaunit)

The traditional approach: named functions that check a condition and report failure.

```python
self.assertEqual(actual, expected)          # unittest
self.assertIn(element, collection)
self.assertRaises(ValueError, func, arg)
```

```go
if got != want {
    t.Errorf("Add(%d, %d) = %d, want %d", a, b, got, want)  // Go: no assertion library
}
```

**Advantages:** Explicit, no magic. Each assertion knows what it's checking and can produce a targeted error message.
**Disadvantages:** Verbose. Proliferation of assertion methods (`assertEqual`, `assertNotEqual`, `assertAlmostEqual`, `assertGreater`, ...). Go's approach is the most minimal: no assertions at all, just if-statements with error reporting.

### Matchers and expect chains (RSpec, Jest, Chai)

```ruby
expect(result).to eq(42)              # RSpec
expect(list).to include(3)
expect { risky }.to raise_error(IOError)
```

```javascript
expect(result).toBe(42);              // Jest
expect(list).toContain(3);
expect(() => risky()).toThrow(/error/);
```

**Advantages:** Reads like English. Extensible: custom matchers plug in naturally. The `expect` wrapper captures the subject, and matchers define the check.
**Disadvantages:** More complex implementation. Error messages depend on matcher quality.

### pytest's AST-rewritten assertions

pytest takes a unique approach: plain `assert` statements are rewritten at import time via AST transformation to produce rich error messages without any special assertion API.

```python
assert response.status_code == 200
```

On failure, produces:
```
AssertionError: assert 404 == 200
  where 404 = <Response>.status_code
```

**How the rewriting works:**

1. A PEP 302 import hook intercepts test module imports
2. The source is parsed into an AST
3. An `ast.NodeVisitor` subclass walks the tree, finding `Assert` nodes via `visit_Assert()`
4. The assert's test expression is decomposed into subexpressions
5. New AST nodes capture each subexpression's value into temporary variables (named `@py_assert0`, `@py_assert1`, etc. — the `@` prefix is invalid Python, preventing name collisions)
6. A format template is generated from the expression structure, filled with captured intermediate values
7. The `assert` becomes `if not <expr>: raise AssertionError(<formatted message>)`
8. The rewritten AST is compiled to bytecode and cached as `.pyc`

This is the only mainstream testing framework that rewrites the language's assertion mechanism via compiler-level transformation.

Sources: [pytest assertion rewrite source](https://github.com/pytest-dev/pytest/blob/main/src/_pytest/assertion/rewrite.py), [Python Insight: assertion rewriting](https://www.pythoninsight.com/2018/02/assertion-rewriting-in-pytest-part-3-the-ast/)

### Rust's assert macros

```rust
assert_eq!(left, right);
assert_eq!(left, right, "custom message: {}", detail);
```

On failure:
```
thread 'test' panicked at 'assertion `left == right` failed
  left: 3
  right: 5'
```

`assert_eq!` requires both values to implement `Debug` (for display) and `PartialEq` (for comparison). The macro expands to code that evaluates both sides, compares them, and panics with a formatted message showing both values. This is a compile-time transformation (macro expansion) rather than the runtime AST rewriting pytest uses.

### Custom assertion messages

Every serious framework supports custom messages on failure:
- pytest: `assert x == y, f"expected {y} but got {x}"`
- Rust: `assert_eq!(a, b, "for input {}", input)`
- Go: `t.Errorf("got %v, want %v", got, want)` (messages are the only mechanism)
- Jest: `expect(x).toBe(y)` shows both values; `.toBe` diff output is automatic

---

## 4. Mocking and Test Doubles

### Terminology

- **Stub:** Returns canned responses. No verification.
- **Mock:** Records calls and verifies expectations.
- **Spy:** Wraps real object, records calls, delegates to real implementation.
- **Fake:** Working implementation unsuitable for production (e.g., in-memory database).
- **Dummy:** Passed around but never actually used. Satisfies a parameter list.

### Python: unittest.mock

```python
from unittest.mock import Mock, patch, MagicMock

mock = Mock()
mock.method.return_value = 42
result = mock.method("arg")
mock.method.assert_called_once_with("arg")
```

`@patch('module.ClassName')` replaces an object at its lookup path for the duration of a test. Uses Python's dynamic nature to swap objects in `sys.modules`.

`MagicMock` extends `Mock` with default implementations of magic methods (`__len__`, `__iter__`, etc.).

### Rust: mockall

Rust's type system makes mocking fundamentally different from dynamic languages. You cannot monkey-patch at runtime. Instead, mockall uses procedural macros to generate mock struct implementations at compile time.

```rust
#[automock]
trait Database {
    fn get(&self, key: &str) -> Option<String>;
}

#[test]
fn test_lookup() {
    let mut mock = MockDatabase::new();
    mock.expect_get()
        .with(eq("key"))
        .times(1)
        .returning(|_| Some("value".to_string()));

    assert_eq!(service_using_db(&mock), "value");
}
```

Key design constraint: the code under test must accept a trait (not a concrete type) for mocking to work. This forces dependency injection at the type level, which is arguably better design but requires more upfront architecture.

**Expectation matching:** Multiple expectations on the same method evaluate in FIFO order. First match wins. Expectations support:
- Argument predicates: `with(eq(x))`, `withf(|arg| arg > 5)`
- Call counts: `times(1)`, `times(2..5)`, `never()`
- Ordered sequences across mocks via `Sequence`
- Checkpoints: `checkpoint()` validates all pending expectations

Sources: [mockall docs](https://docs.rs/mockall/latest/mockall/), [Mocking in Rust](https://blog.logrocket.com/mocking-rust-mockall-alternatives/)

### JavaScript: jest.fn() and module mocking

```javascript
const callback = jest.fn();
callback(1, 2);
expect(callback).toHaveBeenCalledWith(1, 2);

// Module mocking
jest.mock('./database', () => ({
  query: jest.fn().mockResolvedValue([{ id: 1 }])
}));
```

- `jest.fn()`: Creates a mock function recording all calls. Returns `undefined` unless configured.
- `jest.spyOn(obj, 'method')`: Wraps existing method, preserving original implementation by default. `mockRestore()` restores the original.
- `jest.mock('module')`: Replaces an entire module. Uses Jest's custom module resolver to intercept `require`/`import`. Auto-mocking generates mock implementations for all exports.
- `jest.unstable_mockModule()`: ESM-compatible variant (still experimental in Jest 30).

Jest's module mocking is possible because it controls the module resolution pipeline. Vitest provides equivalent `vi.mock()` using Vite's module graph.

### Dependency injection patterns for testability

The common thread across all languages: mocking requires indirection. The strategies differ:

| Language | Indirection mechanism | Mocking approach |
|----------|----------------------|------------------|
| Rust | Trait objects / generics | Compile-time mock generation from traits |
| Python | Dynamic dispatch, `sys.modules` | Runtime patching of any object |
| JavaScript | Module system, closures | Module replacement at resolution time |
| Go | Interfaces | Manual mock structs implementing interfaces |
| Java | Interfaces, subclassing | Byte-code generation (Mockito/cglib) |

Languages with dynamic dispatch (Python, JS, Ruby) allow mocking anything at runtime. Languages with static dispatch (Rust, Go) require architectural decisions upfront — interfaces/traits must exist before mocking is possible.

---

## 5. Property-Based Testing

Instead of specific input-output examples, declare properties that should hold for all inputs matching a description. The framework generates random inputs and searches for counterexamples.

### Origins: Haskell's QuickCheck

Introduced by Koen Claessen and John Hughes (2000). Core ideas:
- **Arbitrary typeclass:** Defines how to generate random values for a type
- **Property:** A function returning `Bool` (or `Testable`) that should hold for all inputs
- **Shrinking:** When a failure is found, systematically reduce the input to the smallest failing case

### How shrinking works

Shrinking is what separates property-based testing from pure random testing. Without it, failures produce huge, unreadable inputs.

**QuickCheck-style (stateless, output-based):**
The `shrink` function on a type produces a list of "simpler" values. The framework tries each, keeping the first that still fails, and recurses. For integers, shrinking tries values closer to zero. For lists, it tries removing elements and shrinking remaining elements.

Drawback: shrinking operates on the generated value without knowledge of how it was generated. This can produce values that violate constraints the generator enforced, leading to spurious passes during shrinking.

**Hypothesis-style (stateful, byte-stream-based):**
Hypothesis shrinks the underlying byte stream that drives generation, not the generated values. This guarantees that shrunk values satisfy all generator constraints.

Rules:
- Shorter byte arrays are simpler
- Among equal-length arrays, lexicographically earlier is simpler (treating bytes as unsigned)

Uses Delta Debugging to delete and lower bytes. Strategies are designed so that byte deletion corresponds to meaningful data deletion (e.g., removing a list element removes a boolean marker + element bytes).

**proptest (Rust, value-tree-based):**
Each generated value carries a "value tree" representing its shrinking space. The tree knows how to produce simpler values while respecting the strategy's constraints. Stateful like Hypothesis but represented as a tree rather than byte manipulation.

### Strategies and generators

| Framework | Term | Description |
|-----------|------|-------------|
| QuickCheck (Haskell) | `Arbitrary` typeclass | One generator per type, defined by typeclass instance |
| hypothesis (Python) | `Strategy` | Composable objects: `st.integers()`, `st.lists(st.text())`, `st.builds(MyClass, ...)` |
| proptest (Rust) | `Strategy` trait | `0..100i32`, `prop::collection::vec(any::<u8>(), 0..10)`, composition via `prop_map`, `prop_flat_map` |
| quickcheck (Rust) | `Arbitrary` trait | One generator per type, same limitation as Haskell QuickCheck |

**Composite strategies (hypothesis):**
```python
@given(st.lists(st.integers(min_value=0, max_value=100), min_size=1))
def test_sorted_list_properties(xs):
    result = sorted(xs)
    assert result[0] == min(xs)
    assert result[-1] == max(xs)
    assert len(result) == len(xs)
```

**Proptest strategies (Rust):**
```rust
proptest! {
    #[test]
    fn sort_preserves_length(ref v in prop::collection::vec(any::<i32>(), 0..100)) {
        let sorted = sort(v);
        prop_assert_eq!(sorted.len(), v.len());
    }
}
```

Sources: [Hypothesis internals](https://hypothesis.works/articles/how-hypothesis-works/), [proptest book](https://altsysrq.github.io/proptest-book/proptest/vs-quickcheck.html), [quickcheck](https://github.com/BurntSushi/quickcheck)

---

## 6. Snapshot Testing

Store the expected output of a computation in a file. On subsequent runs, compare actual output against the stored snapshot.

### When snapshots help

- Large, complex output that's tedious to write by hand (serialized data structures, rendered UI trees, formatted reports)
- Exploratory testing: capture behavior first, verify correctness by inspection, then lock it in
- Regression detection: any change in output is flagged

### When snapshots hurt

- Brittle tests: cosmetic changes (whitespace, key ordering) cause failures
- Rubber-stamping: developers `--update` snapshots without reviewing diffs
- Large snapshot files obscure what's actually being tested
- Non-deterministic output (timestamps, random IDs) requires sanitization

### Jest snapshots

```javascript
test('renders correctly', () => {
  const tree = renderer.create(<Button label="Click" />).toJSON();
  expect(tree).toMatchSnapshot();
});
```

- First run: serializes `tree` and writes to `__snapshots__/Component.test.js.snap`
- Subsequent runs: compares against stored snapshot, fails on diff
- `--updateSnapshot` regenerates all snapshots
- `toMatchInlineSnapshot()` stores expected value in the source file itself
- Custom serializers control object representation

### insta (Rust)

```rust
#[test]
fn test_serialization() {
    let value = compute_something();
    insta::assert_yaml_snapshot!(value);
}
```

- Writes `.snap.new` files on failure
- `cargo insta review` opens interactive terminal UI showing diffs
- Accept/reject per snapshot
- Supports inline snapshots (stored as string literals in source)
- Format support: Debug, YAML, JSON, TOML, RON, CSV via serde
- File naming: `snapshots/<module>__<name>.snap`

### Approval testing (general pattern)

The broader pattern beyond Jest/insta:
1. Capture actual output
2. Compare against approved output
3. If different, present diff for human review
4. Human approves or rejects

Tools: ApprovalTests (multi-language), TextTest, and the frameworks above.

Sources: [Jest snapshots](https://jestjs.io/docs/snapshot-testing), [insta.rs](https://insta.rs/)

---

## 7. Fuzz Testing

Automated generation of inputs to find crashes, hangs, and unexpected behavior. Distinguished from property-based testing by its focus on coverage guidance and security-relevant failures.

### Coverage-guided fuzzing

The dominant modern approach. The fuzzer instruments the program to track which code paths are executed, then mutates inputs to maximize coverage.

**AFL (American Fuzzy Lop):**
- Process-based: forks a new process for each input
- Compile-time instrumentation via a modified `gcc`/`clang` drop-in replacement
- Genetic algorithm: mutate inputs, retain those that reach new code edges
- Edge coverage: tracks transitions between basic blocks, not just basic block visits
- Mutation strategies: bit flips, byte substitutions, arithmetic operations, block insertion/deletion, splicing from multiple inputs
- Corpus management: maintains a queue of interesting inputs, periodically trims them

**libFuzzer (LLVM):**
- In-process: runs target function in a loop within one process (faster, but less isolation)
- Target function signature: `extern "C" int LLVMFuzzerTestOneInput(const uint8_t *Data, size_t Size)`
- Coverage via LLVM's SanitizerCoverage instrumentation
- Tracks "features": edge coverage, edge counters, value profiles, indirect caller/callee pairs
- Mutations: ChangeByte, ChangeBit, CrossOver, InsertByte, EraseBytes, CopyPart
- Retains inputs that trigger previously-uncovered code paths

**Key difference:** AFL forks processes (safer, slower). libFuzzer stays in-process (faster, fragile — a crash in the target crashes the fuzzer).

### cargo-fuzz (Rust)

Wraps libFuzzer for Rust projects. Fuzz targets are written as functions in `fuzz/fuzz_targets/`:

```rust
fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = parse(s);
    }
});
```

Uses `cargo-fuzz` CLI to manage targets, corpora, and crash artifacts. Also integrates with `arbitrary` crate for structured fuzzing (generating typed values from byte slices).

### Go's built-in fuzzing (since 1.18)

Go is the first mainstream language to include fuzzing in the standard toolchain.

```go
func FuzzReverse(f *testing.F) {
    f.Add("hello")
    f.Fuzz(func(t *testing.T, s string) {
        rev := Reverse(s)
        doubleRev := Reverse(rev)
        if s != doubleRev {
            t.Errorf("double reverse of %q is %q", s, doubleRev)
        }
    })
}
```

**Two execution modes:**
1. `go test`: runs seed corpus entries as unit tests
2. `go test -fuzz=FuzzReverse`: continuously generates and mutates inputs

**Coverage guidance:** Gathers baseline coverage from seed corpus, then retains inputs that expand coverage. An input is "interesting" if it reaches previously-uncovered code paths.

**Minimization:** When a failing input is found, the engine reduces it to the smallest form that still fails, then writes it to `testdata/fuzz/{FuzzTestName}/{hash}` as a permanent regression test.

**Seed corpus:** `f.Add()` calls and files in `testdata/fuzz/{FuzzTestName}/`. Generated corpus cached in `$GOCACHE/fuzz`.

**Allowed types:** `string`, `[]byte`, all integer types, `float32/64`, `bool`.

Sources: [Go fuzzing docs](https://go.dev/doc/security/fuzz/), [libFuzzer docs](https://llvm.org/docs/LibFuzzer.html), [AFL GitHub](https://github.com/google/AFL)

---

## 8. Table-Driven and Parameterized Tests

Running the same test logic with multiple input/output pairs.

### Go's table-driven pattern

The idiomatic Go approach. Not a framework feature — it's a coding convention using language features (slices of structs + subtests):

```go
tests := []struct {
    name  string
    input string
    want  int
}{
    {"empty", "", 0},
    {"single", "a", 1},
    {"multi", "abc", 3},
}
for _, tt := range tests {
    t.Run(tt.name, func(t *testing.T) {
        got := Len(tt.input)
        if got != tt.want {
            t.Errorf("Len(%q) = %d, want %d", tt.input, got, tt.want)
        }
    })
}
```

Each `t.Run` creates a named subtest that can be filtered, run in parallel, and reports independently. This is arguably the purest form: no framework machinery, just data structures and a loop.

### pytest.mark.parametrize

```python
@pytest.mark.parametrize("input,expected", [
    ("hello", 5),
    ("", 0),
    ("world", 5),
])
def test_length(input, expected):
    assert len(input) == expected
```

Multiple `parametrize` decorators create a cartesian product. Each parameter set generates a separate test item with a unique ID (shown in output and filterable).

### Rust rstest

```rust
#[rstest]
#[case("hello", 5)]
#[case("", 0)]
#[case("world", 5)]
fn test_length(#[case] input: &str, #[case] expected: usize) {
    assert_eq!(input.len(), expected);
}
```

Each `#[case]` generates an independent test function. The `rstest` proc-macro expands this at compile time into separate `#[test]` functions.

### Without framework support

In languages without parametrize features, the pattern is implemented manually:

```rust
// Rust: loop over test cases, but they share one test name
#[test]
fn test_lengths() {
    let cases = vec![("hello", 5), ("", 0), ("world", 5)];
    for (input, expected) in cases {
        assert_eq!(input.len(), expected, "input: {:?}", input);
    }
}
```

Drawback: if the third case fails, you don't get individual test naming or the ability to run just that case.

### Data-driven testing

The generalization: test data comes from external sources (CSV files, JSON fixtures, database queries). The framework loads data and generates test cases. pytest's `parametrize` can consume any iterable, so loading from files is straightforward.

---

## 9. Test Isolation

Ensuring tests don't affect each other through shared state.

### Process isolation

The strongest form. Each test runs in a separate OS process.

- **pytest-forked / pytest-xdist:** Run tests in worker processes
- **Jest:** Each test file runs in a separate worker (via `jest-worker`), with its own module registry
- **Go:** Subtests share a process but test binaries are separate per package
- **Rust:** Tests in a single crate run as threads in one binary; integration tests (each file in `tests/`) are separate binaries

### Database isolation

**Transaction rollback:** Wrap each test in a database transaction and roll back afterward. The test sees its changes, but they never persist.

- **Elixir Ecto.Sandbox:** The canonical implementation. `Ecto.Adapters.SQL.Sandbox` checks out a connection per test in a transaction. On test completion, rolls back. Provides ACID isolation between concurrent async tests by giving each test its own connection.
- **Django TestCase:** Wraps each test in a transaction, rolls back. `TransactionTestCase` uses truncation instead (slower but tests actual transaction behavior).
- **Rails (database_cleaner):** Configurable strategies: transaction (fastest), truncation, deletion.

**In-memory databases:** SQLite `:memory:` mode for tests that need a real database engine but not persistence.

**Testcontainers:** Spawns isolated Docker containers with real databases per test suite. Provides actual database behavior without shared state.

### Temp directories

Most test frameworks provide temporary directory helpers:

- **pytest:** `tmp_path` fixture provides a unique `pathlib.Path` per test
- **Go:** `t.TempDir()` creates a temp directory cleaned up when the test ends
- **Rust:** `tempfile::TempDir` creates a directory removed on `Drop`
- **Node.js:** `os.tmpdir()` + `fs.mkdtemp()`

### Module/global state isolation

- **Jest:** Resets module registry between test files (each file gets fresh `require` cache)
- **pytest:** Fixtures with `function` scope create fresh state per test; `monkeypatch` fixture temporarily modifies objects and restores them
- **Rust:** `#[test]` functions share a process; global state (static variables) requires synchronization. `serial_test` crate forces sequential execution for tests with shared state.

---

## 10. Built-in vs External Testing

### Built-in (Go, Rust, Elixir)

**Go:**
- `testing` package in stdlib, `go test` command in toolchain
- Fuzzing, benchmarking, example tests all built in
- No assertions by design (the Go team considers assertion libraries harmful to clear error messages)
- Consistency: every Go project uses the same test conventions

**Rust:**
- `#[test]`, `assert!` macros, `cargo test` in toolchain
- `#[cfg(test)]` for conditional compilation of test code
- Integration tests via `tests/` directory convention
- Custom frameworks still unstable (RFC 2318)
- Benchmarking via `#[bench]` (unstable) or Criterion (external)

**Elixir:**
- ExUnit in stdlib, `mix test` in build tool
- Doctests, async tests, tags, filtering all built in
- Rich enough that external test frameworks are rare

**Advantages of built-in:**
- Zero-dependency test setup: clone and `go test` / `cargo test` / `mix test`
- Ecosystem consistency: every project uses the same conventions
- Compiler integration: conditional compilation, test binary generation, IDE support
- Test code ships with language learning materials

**Disadvantages:**
- Evolves at language release cadence (slow innovation)
- Limited by language team's testing philosophy (Go's no-assertions stance)
- Hard to experiment with alternative approaches

### External (Python, JavaScript, Ruby)

**Python:** unittest in stdlib is usable but verbose. pytest dominates with ~90% market share among Python developers.

**JavaScript:** No built-in testing until Node.js v18 (2022). Jest, Mocha, Vitest developed by the community. Multiple competing approaches serve different needs (Jest's all-in-one vs Mocha's bring-your-own-assertions).

**Ruby:** minitest in stdlib, but RSpec's BDD style dominates the Rails ecosystem.

**Advantages of external:**
- Faster innovation: pytest releases independently of Python
- Multiple approaches: RSpec (BDD) vs minitest (xUnit) serve different philosophies
- Community-driven: features respond to user demand, not language committee priorities
- Can be opinionated without constraining the entire ecosystem

**Disadvantages:**
- Dependency management: version conflicts, supply chain concerns
- Configuration overhead: new projects need setup before first test
- Fragmentation: harder for newcomers to know which framework to use
- Less compiler integration: no conditional compilation, no special binary generation

---

## 11. BDD-Style Testing

### RSpec (Ruby)

The originator of modern BDD testing frameworks.

```ruby
RSpec.describe Calculator do
  describe "#add" do
    context "with positive numbers" do
      it "returns the sum" do
        expect(Calculator.new.add(2, 3)).to eq(5)
      end
    end

    context "with negative numbers" do
      it "handles negatives correctly" do
        expect(Calculator.new.add(-1, -2)).to eq(-3)
      end
    end
  end
end
```

The `describe` / `context` / `it` hierarchy maps to: class -> scenario -> expected behavior. Output reads as a specification:

```
Calculator
  #add
    with positive numbers
      returns the sum
    with negative numbers
      handles negatives correctly
```

### Cucumber / Gherkin

Specification by example using natural language.

```gherkin
Feature: User registration
  Scenario: Successful registration
    Given a user visits the registration page
    When they fill in valid details
    And they submit the form
    Then they should see a welcome message
    And their account should be created
```

Each `Given`/`When`/`Then` step maps to a step definition function in code. Cucumber parses the Gherkin, matches steps to definitions via regex, and executes them.

**When BDD helps:**
- Bridge between business stakeholders and developers (non-technical people can read and write Gherkin)
- Living documentation: scenarios are always in sync with implementation
- Acceptance testing at the feature level

**When BDD hurts:**
- Overhead of maintaining step definitions and feature files
- Regex matching between prose and code is fragile
- For developer-to-developer testing, the natural language layer adds no value
- Step reuse across scenarios creates hidden coupling

**Implementations:** Cucumber (Ruby, Java, JS), behave (Python), SpecFlow (.NET), Behat (PHP).

### behave (Python BDD)

Python's Cucumber equivalent. Uses Gherkin syntax with Python step definitions. Less common than pytest in the Python ecosystem.

Sources: [Cucumber BDD](https://cucumber.io/docs/bdd/), [RSpec style guide](https://rspec.rubystyle.guide/)

---

## 12. Coverage

### How coverage tools work

**Source instrumentation:** Insert tracking code at every branch point before compilation. Each branch point has a counter; when executed, the counter increments. After the test run, report which counters are zero (uncovered).

- **Rust (llvm-cov/cargo-llvm-cov):** Uses LLVM's source-based code coverage. Instruments at the LLVM IR level during compilation. Produces precise line and branch coverage.
- **Go (go test -cover):** Rewrites source code before compilation, inserting counter increments at each basic block boundary.
- **Python (coverage.py):** Uses `sys.settrace()` to hook into the interpreter's line-by-line execution. Records which lines execute.
- **JavaScript (istanbul/v8 coverage):** Istanbul instruments source code via AST transformation. V8's built-in coverage uses the engine's internal block counters (more accurate, less overhead).

**Performance overhead:** Instrumentation typically adds 10-30% runtime overhead and 60-90% code size growth (for compiled languages). Interpreted languages (Python) may see higher overhead from tracing.

### Coverage metrics

- **Line coverage:** Percentage of executable lines reached. The most common metric. Misleading because a line with `if a && b` only needs one combination to be "covered."
- **Branch coverage:** Percentage of control flow branches (true/false of each condition) taken. More meaningful than line coverage. Catches cases where `if x > 0` is only tested with positive values.
- **Function coverage:** Percentage of functions called at least once.
- **Condition coverage:** Each boolean sub-expression evaluated to both true and false. The most thorough but most expensive to track.

### Mutation testing

Tests your tests. Makes small changes (mutations) to production code and checks if tests catch them.

**How it works:**
1. Parse the source code
2. Apply mutation operators: replace `+` with `-`, `>` with `>=`, `true` with `false`, delete statements, change return values
3. Run the test suite against each mutant
4. A mutant is "killed" if at least one test fails
5. Mutation score = killed mutants / total mutants

**Why it matters:** 100% line coverage does not mean good tests. Code can be executed without being meaningfully asserted against. Mutation testing reveals tests that execute code but don't check results.

```
Mutation score is a better metric than code coverage because code coverage
covers only execution, whereas mutation testing covers both execution and
assertion.
```

**Tools:**
- **mutmut (Python):** Mutation testing for Python
- **cargo-mutants (Rust):** Mutation testing for Rust
- **Stryker (JavaScript/TypeScript):** Full-featured mutation testing framework
- **PIT (Java):** The standard for Java mutation testing

**Cost:** Running the full test suite N times (once per mutant) is expensive. Mitigation strategies: only mutate changed code, sample mutants, use coverage data to run only relevant tests per mutant.

Sources: [Code Coverage vs Mutation Testing](https://journal.optivem.com/p/code-coverage-vs-mutation-testing), [Mutation Testing - Codecov](https://about.codecov.io/blog/mutation-testing-how-to-ensure-code-coverage-isnt-a-vanity-metric/)
