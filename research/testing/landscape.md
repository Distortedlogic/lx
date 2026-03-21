# Testing Frameworks Across Languages

A survey of how programming languages approach testing: built-in frameworks, third-party ecosystems, and the design decisions that shape each.

## Python

### unittest (xUnit style)

Python's standard library includes `unittest`, modeled after Java's JUnit and the broader xUnit family. Tests are methods on classes that inherit from `unittest.TestCase`.

**Structure:**
- Test classes extend `TestCase`
- Test methods are prefixed with `test_`
- `setUp()` runs before each test method; `tearDown()` runs after
- `setUpClass()` / `tearDownClass()` run once per class (class methods)
- `setUpModule()` / `tearDownModule()` run once per module

**Test discovery:** `python -m unittest discover` scans for files matching `test*.py`, finds classes derived from `TestCase`, and groups them into suites. The test loader inspects each module in a given directory looking for `TestCase` subclasses.

**Assertions:** Built-in methods like `assertEqual()`, `assertTrue()`, `assertRaises()`, `assertIn()`. Each produces a descriptive error message on failure. The assertion methods are instance methods on `TestCase`.

**Test runner:** A component that orchestrates execution and reports outcomes. The default `TextTestRunner` provides console output; custom runners can provide graphical interfaces or structured output formats.

Source: [unittest documentation](https://docs.python.org/3/library/unittest.html)

### pytest

The dominant Python testing framework. pytest strips away the boilerplate of unittest: tests are plain functions, assertions use the bare `assert` statement, and a dependency-injection fixture system replaces setUp/tearDown.

**Test discovery:**
- Files matching `test_*.py` or `*_test.py`
- Functions prefixed with `test_`
- Classes prefixed with `Test` (no `__init__` method)
- Configurable via `python_files`, `python_classes`, `python_functions` in config

**Assertion rewriting via AST transformation:**

pytest's most distinctive technical feature. When pytest starts, it installs a [PEP 302](https://peps.python.org/pep-0302/) import hook (`AssertionRewritingHook`) that intercepts test module imports. Before the module's AST is compiled to bytecode, pytest walks the tree and rewrites every `assert` statement:

1. The import hook parses the source into an AST
2. An `ast.NodeVisitor` subclass finds `Assert` nodes via `visit_Assert()`
3. Each assert's test expression is decomposed into subexpressions
4. New AST nodes are injected to capture intermediate values into temporary variables before evaluation
5. The `assert` is replaced with an `if not <expr>: raise AssertionError(...)` that includes a formatted message showing all intermediate values
6. The rewritten AST is compiled to bytecode and cached

This means `assert a == b` automatically shows the values of `a` and `b` on failure, without requiring special assertion methods. The rewriting only applies to test modules (as defined by `python_files` config) and plugin modules.

Sources: [pytest assertion rewriting source](https://github.com/pytest-dev/pytest/blob/main/src/_pytest/assertion/rewrite.py), [Python Insight: assertion rewriting part 3](https://www.pythoninsight.com/2018/02/assertion-rewriting-in-pytest-part-3-the-ast/)

**Fixture system (dependency injection):**

Fixtures are functions decorated with `@pytest.fixture` that provide test dependencies. Tests declare what they need by naming fixtures as parameters; pytest inspects the function signature and injects the return values.

- **Scoping:** `function` (default, per-test), `class`, `module`, `package`, `session`. Higher-scoped fixtures execute before lower-scoped ones.
- **Yield fixtures:** A fixture can `yield` a value instead of returning. Code after `yield` runs as teardown, in reverse order of fixture initialization.
- **Fixture composition:** Fixtures can request other fixtures, forming a dependency graph. pytest resolves the graph and executes in topological order.
- **conftest.py:** Fixtures defined in `conftest.py` files are automatically available to all tests in the same directory and subdirectories. No import needed. conftest.py files can exist at every level of the test directory hierarchy.
- **Autouse fixtures:** `@pytest.fixture(autouse=True)` makes a fixture apply to every test in scope without explicit parameter declaration.

Sources: [pytest fixtures documentation](https://docs.pytest.org/en/stable/how-to/fixtures.html), [Pytest Fixtures Complete Guide](https://devtoolbox.dedyn.io/blog/pytest-fixtures-complete-guide)

**Plugin architecture:**

pytest's internals use a hook-based plugin system built on `pluggy`. Plugins can:
- Add new command-line options
- Modify test collection and discovery
- Add new fixtures
- Transform test items
- Customize reporting

Plugins are loaded via `pytest_plugins` variable in conftest.py or via setuptools entry points. Plugins imported by `pytest_plugins` are automatically marked for assertion rewriting.

Source: [Writing plugins](https://docs.pytest.org/en/stable/how-to/writing_plugins.html)

**Parametrize:**

`@pytest.mark.parametrize("arg_name", [val1, val2, ...])` generates a separate test invocation for each value. Multiple parametrize decorators create a cartesian product of test cases.

### doctest

Tests embedded in docstrings, extracted and executed by the `doctest` module.

**How it works:**
1. `DocTestFinder` scans module, function, class, and method docstrings
2. `DocTestParser` identifies lines starting with `>>>` (primary prompt) or `...` (continuation prompt)
3. Lines after prompts without prompts are treated as expected output
4. Each example is executed; actual output is compared character-by-character against expected output
5. Blank lines or the next `>>>` prompt terminate an expected-output block

Objects imported into the module are not searched. The module docstring, all function docstrings, class docstrings, and method docstrings are searched.

pytest can also collect and run doctests via `--doctest-modules` or `--doctest-glob`.

Source: [doctest documentation](https://docs.python.org/3/library/doctest.html)

### hypothesis (property-based testing)

A property-based testing library inspired by Haskell's QuickCheck. Instead of writing specific input/output examples, you declare properties that should hold for all inputs matching a given description.

**Architecture — three layers:**

1. **Conjecture engine:** A low-level byte-stream fuzzer. At the core, a `TestData` object acts like an open file handle you can read bytes from. All generation ultimately draws from this byte stream.

2. **Strategy library:** Converts byte streams into structured data. Each strategy implements `do_draw(data)` which reads bytes from `TestData` and returns a structured value. Strategies compose: `st.lists(st.integers())` builds lists by drawing a boolean (continue/stop) then an integer element, repeating until the boolean is False.

3. **Testing interface:** `@given(strategy)` decorates a test function. Hypothesis calls it repeatedly with generated values, looking for failures.

**Shrinking — internal test-case reduction:**

Hypothesis shrinks by manipulating the underlying byte array, not the generated value. Two ordering rules:
- Shorter byte arrays are simpler
- Among equal-length arrays, lexicographically earlier (treating bytes as unsigned 8-bit integers) is simpler

The engine uses Delta Debugging variants to repeatedly delete and lower bytes. Because well-designed strategies arrange data so that byte deletion corresponds to meaningful data deletion (e.g., removing a list element), byte-level shrinking produces semantically useful minimal examples.

**Interval bookkeeping:** `TestData` tracks intervals via `start_interval()` / `stop_interval()` to mark deletion boundaries, reducing shrinking from O(n^2) to manageable complexity.

**Strategy composition:** `flatmap()` supports dependent chaining where later strategies depend on earlier values. The byte-stream approach handles this because "as long as the new strategy has roughly the same shape as the old strategy it will just pick up where the old shrinks left off."

**Database:** Hypothesis persists failing examples to a local database (`.hypothesis/` directory) for replay across runs.

Sources: [How Hypothesis Works](https://hypothesis.works/articles/how-hypothesis-works/), [Hypothesis GitHub](https://github.com/HypothesisWorks/hypothesis), [Strategies Reference](https://hypothesis.readthedocs.io/en/latest/data.html)

---

## Rust

### Built-in test framework

Rust builds testing into the language and toolchain. Tests are regular functions annotated with `#[test]`, compiled into a separate test binary by `cargo test`.

**Compilation model:**

`cargo test` compiles code in test mode, linking with `libtest` to create a special executable. The `--test` flag to `rustc` generates a `main()` function that discovers all `#[test]` functions and runs them, potentially in parallel across threads.

- `#[cfg(test)]` conditionally compiles code only when running `cargo test`, not `cargo build`. This is typically used on a `mod tests` block within each source file.
- Each file in the `tests/` directory is compiled as its own separate crate (integration test). These can only access the public API.
- Unit tests (inside `#[cfg(test)]` modules within `src/`) can access private functions.

**Attributes:**
- `#[test]` marks a function as a test (must take no arguments, return `()` or `Result<(), E>`)
- `#[should_panic]` expects the test to panic; optionally `#[should_panic(expected = "message")]` checks the panic message contains the substring
- `#[ignore]` skips the test by default; run with `cargo test -- --ignored`

**Assert macros:**
- `assert!(expr)` — panics if false
- `assert_eq!(left, right)` — panics if not equal, printing both values via `Debug`
- `assert_ne!(left, right)` — panics if equal
- All accept an optional format string: `assert_eq!(a, b, "values were {} and {}", a, b)`

**Output capture:** By default, `cargo test` captures stdout/stderr from passing tests. Use `cargo test -- --nocapture` to see output from all tests. Failing tests always show their captured output.

**Test filtering:** `cargo test substring` runs only tests whose name contains `substring`. `cargo test -- --test-threads=1` forces serial execution.

Sources: [The Rust Book: Testing](https://doc.rust-lang.org/book/ch11-01-writing-tests.html), [Test Organization](https://doc.rust-lang.org/book/ch11-03-test-organization.html)

**Custom test frameworks (unstable):**

[RFC 2318](https://rust-lang.github.io/rfcs/2318-custom-test-frameworks.html) proposes a mechanism for custom test frameworks. The design:

- Test frameworks are procedural macros evaluated after all other macros
- The framework receives all items annotated with declared attributes
- It generates a custom `main()` function for test orchestration
- Crates opt in via `Cargo.toml` configuration
- Setting `harness = false` in `Cargo.toml` already allows bypassing libtest for integration tests, enabling custom harnesses

This is used in practice by crates like Criterion (benchmarking) and for `no_std` testing (e.g., OS kernel development). The feature remains unstable as of 2025.

Sources: [RFC 2318](https://rust-lang.github.io/rfcs/2318-custom-test-frameworks.html), [Writing an OS in Rust: Testing](https://os.phil-opp.com/testing/)

### proptest and quickcheck

Two property-based testing crates, both inspired by Haskell's QuickCheck but with different architectures.

**quickcheck:**
- Type-based generation and shrinking: implements `Arbitrary` trait per type
- Only one generator per type (newtypes needed for alternatives)
- Stateless shrinking: operates on the output value directly
- Binary search over input space for efficient shrinking
- Default 100 iterations per property
- Created by BurntSushi (Andrew Gallant)

**proptest:**
- Strategy-based generation: explicit `Strategy` objects decouple generation from types
- Multiple strategies per type without newtypes (e.g., `0..100i32` vs `0..1000i32`)
- Stateful shrinking: maintains intermediate states and relationships for richer reduction
- Constraint-aware: strategies avoid generating/shrinking to values that violate declared constraints
- Composition via `prop_map`, `prop_flat_map` without manual bidirectional mapping
- Performance tradeoff: generating complex values can be up to an order of magnitude slower than quickcheck due to state maintenance overhead

Sources: [proptest vs quickcheck](https://altsysrq.github.io/proptest-book/proptest/vs-quickcheck.html), [quickcheck](https://github.com/BurntSushi/quickcheck), [proptest](https://github.com/proptest-rs/proptest)

### insta (snapshot testing)

A snapshot testing crate for Rust that stores reference values in separate `.snap` files alongside test code.

**Workflow:**
1. Write a test using assertion macros: `assert_snapshot!`, `assert_debug_snapshot!`, `assert_yaml_snapshot!`, `assert_json_snapshot!`
2. On first run (or when output changes), tests fail and write `.snap.new` files
3. Run `cargo insta review` for an interactive terminal UI showing diffs
4. Accept (writes `.snap`) or reject changes

**Storage:** Snapshots go in a `snapshots/` directory next to the test file, named `<module>__<name>.snap`. Inline snapshots store the reference value directly in the source file as a string literal.

**Format support:** CSV, JSON, TOML, YAML, RON via serde serialization.

Source: [insta.rs](https://insta.rs/), [insta docs.rs](https://docs.rs/insta)

### rstest (parameterized testing)

A procedural macro crate providing pytest-like fixtures and table-driven tests for Rust.

- `#[rstest]` attribute on test functions
- `#[case(...)]` attributes define test case rows
- Fixture functions annotated with `#[fixture]` provide reusable setup
- Generates an independent test for each case
- Supports async tests with any runtime

Source: [rstest](https://github.com/la10736/rstest)

### mockall

The dominant mocking crate for Rust. Generates mock structs from traits via procedural macros.

**Two generation modes:**
- `#[automock]` attribute on a trait: generates `MockTraitName` struct automatically
- `mock!` macro: manual definition for complex cases (multiple traits, external crates)

**Expectations:** Each trait method gets a corresponding `expect_method()` on the mock. Expectations support:
- Return values: `return_const()`, `returning(closure)`, `return_once(FnOnce)`
- Argument matching: `with(predicate)`, `withf(closure)`
- Call count verification: `times(n)`, `never()`
- Ordering: `Sequence` struct enforces cross-mock call ordering
- Checkpoints: `checkpoint()` validates pending expectations mid-test

**Generics:** Supports generic traits and methods. Type parameters in expectations require turbofish syntax. Associated types are specified in attribute metadata.

**Limitations:** `impl Trait` return types are transformed to `Box<dyn Trait>` internally. Lifetimes in return types have restrictions.

Sources: [mockall docs](https://docs.rs/mockall/latest/mockall/), [mockall GitHub](https://github.com/asomers/mockall)

---

## JavaScript

### Jest

Meta's testing framework, the default for React projects. Provides an all-in-one solution: test runner, assertion library, mocking, snapshots, and code coverage.

**Internal architecture (modular packages):**
- `jest-cli`: Entry point, orchestrates the test process
- `jest-config`: Loads CLI flags, config files, builds project configs
- `jest-haste-map`: Scans filesystem, builds dependency map from imports/requires, caches it
- `jest-worker`: Parallelizes heavy tasks across CPU cores
- `jest-runtime`: Custom module system for test isolation

**Snapshot testing:**
- `expect(value).toMatchSnapshot()` serializes the value and compares against a stored `.snap` file
- On first run, creates the snapshot; on subsequent runs, diffs against it
- `--updateSnapshot` (`-u`) flag regenerates all snapshots
- Inline snapshots: `toMatchInlineSnapshot()` stores the expected value in the source file
- Custom serializers control how objects appear in snapshots

**Mocking system:**
- `jest.fn()`: Creates a mock function that records calls, arguments, return values, and instances. Returns `undefined` by default.
- `jest.spyOn(object, method)`: Wraps an existing method, recording calls while still calling the original implementation by default. `mockRestore()` restores the original.
- `jest.mock('module')`: Replaces an entire module with auto-generated mocks or a manual factory function. Uses Jest's custom module resolver.
- `jest.unstable_mockModule()`: ESM-compatible module mocking (still experimental as of Jest 30)

**Jest 30 (June 2025):** Bundled into single files per package for performance. Module resolution became faster and more standards-compliant. Memory usage reduced. ESM support improved but still experimental.

Sources: [Jest documentation](https://jestjs.io/), [Jest 30 blog post](https://jestjs.io/blog/2025/06/04/jest-30), [Jest snapshot testing](https://jestjs.io/docs/snapshot-testing)

### Vitest

An ESM-native testing framework built on Vite. API-compatible with Jest but architecturally different.

**Key architectural differences from Jest:**
- Native ESM support without flags or configuration
- Uses Vite's esbuild for near-instant startup (cold runs up to 4x faster than Jest)
- Shares Vite's config, aliases, and plugins
- Hot Module Reloading for tests: changed tests re-run without restarting
- TypeScript and JSX support out of the box via Vite's transform pipeline
- ~30% lower memory usage vs Jest

**Configuration:** Extends `vite.config.ts` with a `test` section. Shares the same resolve aliases, plugins, and transform pipeline as the dev server.

**Compatibility:** Provides Jest-compatible APIs (`describe`, `it`, `expect`, `vi.fn()`, `vi.mock()`), making migration straightforward.

Sources: [Vitest comparisons](https://vitest.dev/guide/comparisons.html), [Jest vs Vitest 2025](https://medium.com/@ruverd/jest-vs-vitest-which-test-runner-should-you-use-in-2025-5c85e4f2bda9)

### Mocha

A flexible, minimal test runner for Node.js and browsers. Does not bundle assertions or mocking — you bring your own (typically Chai for assertions, Sinon for mocking).

**Internal architecture:**

Two-phase execution:

1. **Parsing phase:** CLI uses `yargs` for argument handling. Loads config, selects reporter and UI interface, constructs a `Mocha` instance with root `Suite`.

2. **Execution phase:** Files loaded asynchronously. `describe()` creates `Suite` objects; `it()` creates `Test` objects, building a hierarchical tree. The `Runner` (extends `EventEmitter`) traverses the tree recursively.

**Hook lifecycle per suite:**
1. `before` (beforeAll) hooks run once
2. For each test: `beforeEach` -> test execution -> `afterEach`
3. `after` (afterAll) hooks run after all tests

**Reporter architecture:** Reporters attach listeners to Runner events (`EVENT_TEST_PASS`, `EVENT_TEST_FAIL`, `EVENT_RUN_END`). A stats collector tracks counts and timing. Built-in reporters: spec, dot, nyan, landing, TAP, JSON, and more. Third-party reporters installable via npm.

**Key design:** `Suite`, `Test`, and `Hook` all inherit from `Runnable`, which provides the core `run()` method. The event-driven architecture fully decouples execution from output formatting.

Sources: [Mocha documentation](https://mochajs.org/), [Under the hood of test runners](https://craigtaub.dev/under-the-hood-of-test-runners/)

### Node.js built-in test runner (node:test)

Introduced in Node.js v18, stabilized in v20. Zero-dependency testing built into the runtime.

**Core API:**
- `test('name', async (t) => { ... })` defines a test
- `describe()` / `it()` for BDD-style organization (aliases for `test()`)
- Uses `node:assert` (specifically `node:assert/strict`) for assertions
- `t.mock.fn()` for mock functions, `t.mock.method()` for spying

**Execution:** Tests run in parallel by default. Supports `--test-reporter` flag for output format selection: spec (default), TAP, dot, JUnit, lcov.

**Advantages:** No external dependencies. Ships with the runtime. Good for projects that want minimal tooling overhead. Gradually approaching feature parity with external frameworks.

Sources: [Node.js test runner docs](https://nodejs.org/api/test.html), [node:test guide](https://nodejs.org/en/learn/test-runner/using-test-runner)

---

## Go

### testing package

Go's standard library `testing` package provides three testing modes, all built into `go test`.

**Regular tests (testing.T):**
- Functions named `func TestXxx(t *testing.T)` in `_test.go` files
- No assertion library by default; use `if` statements with `t.Error()`, `t.Fatal()`, `t.Errorf()`
- `t.Run("subtest", func(t *testing.T) { ... })` creates named subtests
- The Go team deliberately omits assertion helpers, preferring explicit if-checks and table-driven patterns

**Table-driven tests:**

The idiomatic Go testing pattern. Define a slice of test cases as structs, iterate with `t.Run`:

```go
tests := []struct {
    name     string
    input    int
    expected int
}{
    {"zero", 0, 0},
    {"positive", 5, 25},
    {"negative", -3, 9},
}
for _, tt := range tests {
    t.Run(tt.name, func(t *testing.T) {
        if got := Square(tt.input); got != tt.expected {
            t.Errorf("Square(%d) = %d, want %d", tt.input, got, tt.expected)
        }
    })
}
```

Each subtest runs independently, can be filtered by name (`go test -run TestSquare/negative`), and reports failures at the subtest level.

**Benchmarking (testing.B):**
- Functions named `func BenchmarkXxx(b *testing.B)`
- The framework controls `b.N` (iteration count) to get stable timing measurements
- `b.Run()` creates sub-benchmarks for table-driven benchmarks
- `b.ResetTimer()`, `b.StopTimer()`, `b.StartTimer()` for excluding setup from measurements
- Run with `go test -bench=.`

**Fuzzing (testing.F) — built-in since Go 1.18:**

Go is one of few languages with fuzzing built into the standard toolchain.

- Functions named `func FuzzXxx(f *testing.F)`
- `f.Add(values...)` provides seed corpus entries
- `f.Fuzz(func(t *testing.T, args...) { ... })` defines the fuzz target
- Allowed argument types: `string`, `[]byte`, all integer types, `float32/64`, `bool`

Two execution modes:
1. **Unit test mode** (`go test`): runs each seed corpus entry, reports failures
2. **Fuzzing mode** (`go test -fuzz=FuzzXxx`): continuously generates and mutates inputs using coverage guidance

Coverage-guided mutation: the engine gathers baseline coverage from seed corpus, then generates mutations, retaining inputs that expand code coverage. An input is "interesting" if it reaches previously-uncovered code paths.

Minimization: when a failing input is found, the engine reduces it to the smallest, most human-readable form, then writes it to `testdata/fuzz/{FuzzTestName}/{hash}` as a regression test that runs by default on future `go test` invocations.

Corpus file format:
```
go test fuzz v1
string("hello")
int64(572293)
```

**Example tests:**
- Functions named `func ExampleXxx()` with `// Output:` comments
- Serve as both tests (output is verified) and documentation (appear in `godoc`)

Sources: [testing package](https://pkg.go.dev/testing), [Go Fuzzing docs](https://go.dev/doc/security/fuzz/)

### testify

The most popular third-party assertion library for Go.

- `assert` package: rich assertion functions (`assert.Equal`, `assert.Contains`, `assert.Error`), continues on failure
- `require` package: same assertions but stops test execution on failure (calls `t.FailNow()`)
- `suite` package: xUnit-style test suites with `SetupTest()`, `TearDownTest()`, `SetupSuite()`, `TearDownSuite()`
- `mock` package: mock objects with expectation setting

Sources: [testify GitHub](https://github.com/stretchr/testify), [Testing in Go with Testify](https://betterstack.com/community/guides/scaling-go/golang-testify/)

---

## Elixir

### ExUnit

Elixir's built-in testing framework, included in the standard library.

**Structure:**
```elixir
defmodule MyTest do
  use ExUnit.Case, async: true

  describe "feature" do
    test "behavior", %{} do
      assert 1 + 1 == 2
    end
  end
end
```

**Key features:**

- **`use ExUnit.Case`** imports `ExUnit.Assertions`, `ExUnit.Callbacks`, and `ExUnit.DocTest`
- **`test` macro:** Defines individual test cases. Each test receives a context map as argument.
- **`describe` blocks:** Group related tests. Purely organizational (no nesting).
- **`setup` callback:** Runs before each test in scope. Returns a map merged into the test context.
- **`setup_all` callback:** Runs once before all tests in the module. For expensive initialization.

**Async execution:**

`async: true` allows the test module to run concurrently with other async test modules. Individual tests within a module still run serially. The `:max_cases` config controls maximum parallel modules.

`async_run/0` starts tests asynchronously during loading; `await_run/1` waits for completion.

**Doctest integration:**

```elixir
defmodule MyModule do
  @doc """
  Adds two numbers.

      iex> MyModule.add(1, 2)
      3
  """
  def add(a, b), do: a + b
end

defmodule MyModuleTest do
  use ExUnit.Case
  doctest MyModule
end
```

The `doctest` macro scans module documentation for `iex>` examples and generates test cases from them. Supports `:only` and `:except` options.

**Tags and filtering:**
- `@tag :slow` on tests, `--exclude slow` on command line
- `@moduletag :integration` applies to all tests in module
- `capture_log: true` captures Logger output during tests

**Configuration:**
- Default timeout: 60,000ms per test
- `:seed` for reproducible random ordering
- `:max_failures` halts after N failures

**Discovery:** Mix discovers `*_test.exs` files in the `test/` directory. `test/test_helper.exs` loads common setup and calls `ExUnit.start()`.

Sources: [ExUnit docs](https://hexdocs.pm/ex_unit/ExUnit.html), [ExUnit.Case](https://hexdocs.pm/ex_unit/ExUnit.Case.html), [ExUnit.DocTest](https://hexdocs.pm/ex_unit/ExUnit.DocTest.html)

---

## Ruby

### RSpec (BDD style)

The dominant Ruby testing framework, built around Behavior-Driven Development.

**Core DSL:**
- `describe`: Defines a group (typically a class or method under test)
- `context`: Nested group for a specific scenario (alias of `describe`, semantically distinct)
- `it`: Defines a single example (test case)
- `expect(value).to matcher`: Assertion syntax

**let and let!:**
- `let(:name) { expression }`: Lazily-evaluated, memoized helper. Not assigned until first called in a test. Re-evaluated per example.
- `let!(:name) { expression }`: Eagerly evaluated before each example.
- `subject { expression }`: Special `let` for the object under test. Can be named: `subject(:widget) { Widget.new }`

**Hooks:**
- `before(:each)` / `after(:each)`: Run around every example
- `before(:all)` / `after(:all)`: Run once per group
- `around(:each)`: Wraps each example (receives the example as a block)

**Mocking and test doubles:**
- `double("name")`: Creates a test double (fake object)
- `allow(obj).to receive(:method).and_return(value)`: Stub a method
- `expect(obj).to receive(:method).with(args)`: Set expectation that method will be called
- `instance_double(ClassName)`: Verified double that checks the real class's interface
- Integrates deeply with Ruby's object model via method interception

**Matchers:** Rich built-in matchers (`eq`, `be`, `include`, `match`, `raise_error`, `change`, `have_attributes`). Custom matchers via `RSpec::Matchers.define`.

Sources: [RSpec style guide](https://rspec.rubystyle.guide/), [BDD with RSpec](https://blog.appsignal.com/2024/01/24/behaviour-driven-development-in-ruby-with-rspec.html)

### minitest

Bundled with Ruby's standard library. Minimal, fast, and follows Ruby idioms.

**Two APIs in one:**

1. **Unit style (minitest/unit):** xUnit pattern with `Minitest::Test` subclasses
   - `def test_something` methods
   - `assert_equal expected, actual` assertions
   - `setup` / `teardown` methods

2. **Spec style (minitest/spec):** BDD-like DSL that hooks onto minitest/unit
   - `describe` / `it` blocks
   - `_(value).must_equal expected` expectations
   - Bridges assertions to spec expectations internally

**Design philosophy:** "Doesn't reinvent anything that Ruby already provides." Uses classes, modules, inheritance, and methods. Standard OO practices (extract-method refactoring) apply directly. Extremely fast startup.

Also includes `minitest/mock` for simple mocking and `minitest/benchmark` for performance assertions.

Sources: [minitest GitHub](https://github.com/minitest/minitest), [Minitest vs RSpec](https://betterstack.com/community/guides/scaling-ruby/minitest-vs-rspec/)

---

## Lua

### busted

A BDD-style testing framework for Lua, the most full-featured option in the ecosystem.

**Syntax:**
```lua
describe("Calculator", function()
  it("adds numbers", function()
    assert.are.equal(4, 2 + 2)
  end)

  it("handles negatives", function()
    assert.are.equal(-1, 1 + (-2))
  end)
end)
```

- Works with Lua >= 5.1, MoonScript, Terra, and LuaJIT >= 2.0.0
- BDD-style `describe`/`it` blocks
- Rich assertion library with chainable syntax
- Multiple output formats including TAP
- Supports mocking, stubs, and spies
- Installed via LuaRocks

Source: [busted documentation](https://lunarmodules.github.io/busted/)

### luaunit

An xUnit-style framework contained in a single file with no external dependencies.

**Design:**
- Interface matches Python's unittest, Java's JUnit, C#'s NUnit
- Test classes with `test_` prefixed methods
- `assertEquals`, `assertError`, `assertNil`, etc.
- Multiple output formats: Text, TAP, JUnit XML
- CI-friendly (Jenkins, Maven compatible)
- Single-file distribution: copy `luaunit.lua` into your project

Source: [LuaUnit GitHub](https://github.com/bluebird75/luaunit)

### Minimal alternatives

Lua's small standard library means testing frameworks must be self-contained:

- **Gambiarra:** ~120 lines, full testing capability for Lua 5.1/5.2
- **Testy:** Collects test functions from local variables
- **Minctest:** Tiny unit testing framework

The ecosystem reflects Lua's embedding-first design: many Lua deployments are inside game engines, embedded systems, or configuration layers where heavyweight test infrastructure is impractical.

Source: [Lua-users wiki: Unit Testing](http://lua-users.org/wiki/UnitTesting)

---

## Cross-cutting observations

### Built-in vs external testing

| Language | Built-in | External ecosystem | Trade-off |
|----------|----------|-------------------|-----------|
| Go | `testing` package, `go test` command, fuzzing | testify, gomock | Consistency and zero-dep setup; limited assertion expressiveness by design |
| Rust | `#[test]`, `cargo test`, libtest | proptest, mockall, insta, rstest | Deep compiler integration; custom frameworks still unstable |
| Elixir | ExUnit (full-featured) | Mox, StreamData | Built-in is sufficient for most projects |
| Python | unittest, doctest | pytest dominates | stdlib is usable but pytest won the ecosystem |
| JavaScript | node:test (v20+) | Jest, Vitest, Mocha | External frameworks matured first; built-in catching up |
| Ruby | minitest (stdlib) | RSpec dominates | Similar to Python: stdlib exists, community prefers third-party |
| Lua | None | busted, luaunit | Must bring everything; single-file frameworks fill the gap |

**Languages that build testing into the toolchain** (Go, Rust, Elixir) achieve: zero-dependency test setup, consistent conventions across the ecosystem, compiler-level integration (conditional compilation, binary generation), and unified documentation (Go examples, Elixir doctests).

**Languages that rely on external frameworks** (Python, JavaScript, Ruby) achieve: faster innovation in testing features, multiple competing approaches that serve different needs, community-driven evolution independent of language release cycles. The cost is fragmentation, configuration overhead, and dependency management.
