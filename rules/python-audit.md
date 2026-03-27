# Python Codebase Quality Audit

Every item below is a binary check — a violation either exists or it does not. There is no "partially violates" or "could be improved." The audit checks each item across all `.py` files in all packages.

Run the **High Frequency** list first — these violations are commonly introduced by both humans and AI agents. Run the **Low Frequency** list second — these are rarer structural issues.

---

## High Frequency Checks

- **Dynamic attribute access** - `setattr()`, `hasattr()`, or `getattr()` used anywhere. These functions bypass type checkers entirely. Fix: use direct dot notation (`obj.field = value`, not `setattr(obj, "field", value)`).
  `rg 'setattr\(|hasattr\(|getattr\(' --type py src/`

- **Inline import paths** - a call site uses `module.path.Type` instead of a short name. Fix: add a `from module.path import Type` statement at the top of the file, use the short name at the call site.
  `rg 'import ' --type py src/`
  Manual review: for each file, check if qualified names like `module.submodule.func()` appear at call sites where a `from` import would be cleaner.

- **Missing or underused `__init__.py` exports** - a package that exports 3+ types/functions commonly imported by other packages does not have a clean `__init__.py` with explicit imports, or a consuming module imports individual items from submodules when the package's `__init__.py` already re-exports them. Both sides of the violation: (1) **missing exports** — a package has 3+ public items imported across 2+ consuming modules, but `__init__.py` does not re-export them. Fix: add explicit imports in `__init__.py`. (2) **exports exist but not used** — a consuming module imports from submodules when `__init__.py` already provides the items (e.g., `from mypackage.submodule import Foo` instead of `from mypackage import Foo`). Fix: import from the package root. (3) **stale exports** — `__init__.py` re-exports items no other module uses, or omits items commonly imported. Fix: add missing items, remove unused ones.
  `rg '__all__' --type py src/`
  `rg 'from \w+\.\w+ import' --type py src/`

- **Verbose patterns with idiomatic alternatives** - unnecessary list comprehension where a generator expression suffices (e.g., `sum([x for x in items])` instead of `sum(x for x in items)`), manual loop building a list where a comprehension does the same work, `if x == True` / `if x == False` / `if x == None` instead of `if x` / `if not x` / `if x is None`, `len(x) == 0` instead of `not x`, manual dictionary building where a dict comprehension suffices, `isinstance(x, (A,)) or isinstance(x, (B,))` instead of `isinstance(x, (A, B))`. Fix: use the idiomatic alternative.
  `rg '== True|== False|== None|!= None' --type py src/`
  `rg 'len\(\w+\) [=!]= 0' --type py src/`
  `rg 'sum\(\[' --type py src/`
  `rg 'for .* in .*:\s*$' --type py src/` (review for loops that could be comprehensions)

- **Self-assignments** - `x = x` exists with no purpose. Fix: remove the self-assignment.
  `rg '(\w+) = \1$' --type py src/`

- **Repeated literals** - a literal value (string, number, list) appears at 2+ call sites without being extracted. Fix: extract to a module-level constant.
  `rg -o '"[^"]{4,}"' --type py --no-filename src/ | sort | uniq -c | sort -rn | head -30`
  `rg -o "'[^']{4,}'" --type py --no-filename src/ | sort | uniq -c | sort -rn | head -30`
  `rg -o '\b[0-9][0-9][0-9][0-9]*\b' --type py --no-filename src/ | sort | uniq -c | sort -rn | head -30`

- **Suppression comments** - any `# noqa`, `# type: ignore`, `# pylint: disable`, or `# pragma: no cover` exists. Fix: remove the suppression, fix the underlying warning, type error, or remove the unused code.
  `rg '# noqa|# type: ignore|# pylint: disable|# pragma: no cover' --type py src/`

- **Overly specific parameter types** - a function accepts `list` when `Sequence` or `Iterable` would suffice (parameter is only iterated or indexed, never mutated), or accepts `dict` when `Mapping` suffices. Fix: use the appropriate abstract type from `collections.abc` or `typing`.
  `rg 'def .*\(.*: list\b' --type py src/`
  `rg 'def .*\(.*: dict\b' --type py src/`
  For each hit: check if the function mutates the collection. If not, use `Sequence`/`Iterable`/`Mapping`.

- **Intermediate list materialization** - a list comprehension or `list()` call is created only to be immediately iterated, passed to `sum()`, `any()`, `all()`, `min()`, `max()`, `set()`, `dict()`, or `str.join()`. Fix: use a generator expression or chain iterators directly.
  `rg 'sum\(\[|any\(\[|all\(\[|min\(\[|max\(\[|set\(\[|".*"\.join\(\[' --type py src/`
  `rg 'list\(.*\)' --type py src/` (check if result is immediately iterated)

- **Files exceeding 300 lines** - any `.py` file exceeds 300 lines. Fix: split into multiple files/modules.
  `find src/ -name '*.py' -exec awk 'END { if (NR > 300) print FILENAME, NR }' {} \;`

- **Dead imports / unused dependencies** - an `import` statement imports a name never referenced in the file, or a dependency in `pyproject.toml` / `requirements.txt` is unused. Fix: remove the unused import or dependency.
  `ruff check --select F401 src/`
  `rg '^import |^from .* import' --type py src/`

- **Swallowed errors** - bare `except:`, `except Exception: pass`, `except: pass`, catching an exception and silently ignoring it, or using a broad `except Exception` where a specific exception type is appropriate. Fix: handle the error explicitly — propagate it, log it, or surface it to the user.
  `rg 'except.*:\s*$' --type py -A1 src/` (look for `pass` or empty bodies)
  `rg 'except:' --type py src/`
  `rg 'except Exception' --type py src/`

- **Free functions that should be methods** - a function takes an object as its first parameter and accesses that object's attributes or methods. Fix: move it to a method on that class.
  `rg '^def ' --type py src/`
  Manual review: for each function outside a class, check if the first parameter is an instance whose attributes are accessed.

- **Extracted function with single call site** - a non-public function (name starts with `_` or is module-private) is called from exactly one place. Fix: inline the body at the call site and delete the function. Exception: (1) inlining would push the file over 300 lines, (2) the function is recursive, (3) the function represents a genuinely self-contained algorithm whose name communicates a distinct contract.
  For each private function: `rg '\bfn_name\b' --type py src/` — if count is 2 (definition + call), inline.

- **Unnecessary exception chaining with no added info** - raising a new exception with `from e` or wrapping in a new exception type where the wrapping adds no context beyond what the original exception already says. Fix: let the original exception propagate or add meaningful context.
  `rg 'raise .* from ' --type py src/`
  `rg 'raise .*\(.*str\(e\)' --type py src/`
  Manual review: for each hit, check if the new exception adds meaningful context.

- **Field duplication across classes** - a dataclass, Pydantic model, or regular class duplicates 2+ fields from another class instead of composing it as a single field. Fix: hold the source class as a single field.
  `rg '@dataclass|class.*BaseModel' --type py src/`
  Manual review: compare field names across classes. Flag pairs sharing 2+ fields.

- **Duplicate types** - two classes share 3+ identical fields. Fix: merge into one class or use inheritance/composition.
  Manual review: extract field names from dataclasses/Pydantic models, compute pairwise intersection, flag pairs sharing 3+ fields.

- **Duplicate methods** - two or more methods across different classes or modules have identical or near-identical bodies. Fix: extract to a shared method, a mixin, a base class method, or a standalone function called by both.
  Manual review: compare method bodies across classes.

- **Re-exports from non-defining packages** - a type/function is re-exported from a package other than the one that defines it. Fix: import directly from the defining package at usage sites.
  `rg '^from .* import .*' --type py src/` (in `__init__.py` files)

- **Custom code vs established package** - custom utility code exists where an established package (e.g., `itertools`, `more-itertools`, `toolz`, `boltons`, `pathlib`, `dataclasses`, `attrs`) provides the same functionality. Fix: use the package.
  Manual review only.

- **Over-engineering** - unnecessary ABCs or Protocol classes used polymorphically by exactly one type, unnecessary metaclasses where a simple class suffices, unnecessary decorator factories for a single use, or multiple layers of indirection for a simple operation. Fix: remove the unnecessary abstraction, use concrete implementations directly.
  `rg 'class.*ABC|class.*Protocol|class.*Meta|metaclass=' --type py src/`
  For each abstract class/protocol, check if only one concrete implementation exists.

- **Inappropriate defaulting** - a value is defaulted where the default silences a bug, masks a missing value, or is semantically incorrect. Patterns: `or ""`, `or 0`, `or []`, `or {}`, `or False`, `.get(key, "")`, `.get(key, 0)`, `getattr(obj, attr, None)` where `None` masks a real absence, `Optional[T] = None` where the value is always expected to be present, mutable default arguments. Reasons a default can be inappropriate: (1) **bug silencing** — an exception or `None` signals a real problem but the default makes the code continue, (2) **semantic incorrectness** — `0` is not "no price," `""` is not "no name," (3) **silent data loss** — a failed parse or missing field is replaced with a default, (4) **incorrect aggregation** — a default zero skews sums/averages, (5) **deferred error** — the default creates invalid state that causes harder-to-debug failures later, (6) **optional-as-default** — `Optional[T]` with `None` used where the value is always expected present, making every access pay for an `if x is not None` check that can never legitimately be `None`. Fix: raise an exception, make the parameter required, or use a sentinel value.
  `rg 'or ""| or 0| or \[\]| or \{\}| or False' --type py src/`
  `rg '\.get\(' --type py src/`
  `rg 'Optional\[' --type py src/`
  Manual review: for each hit, determine whether the default is semantically valid.

- **Bare except that should re-raise** - catching an exception and returning a default value when the caller should decide how to handle the error. Fix: let the exception propagate.
  `rg 'except.*:' --type py -A3 src/` (look for `return` with a default value inside except blocks)

- **Mergeable code** - two or more functions, methods, if/elif branches, modules, or classes that share the majority of their logic. Fix: merge into a single unit with a parameter for the difference.
  Manual review.

- **Dicts instead of structured types** - a `dict` is used to represent a structured data object with known, fixed keys — in function parameters, return values, class fields, or local variables — where a Pydantic model or dataclass would provide type safety, attribute access, validation, and IDE support. Patterns to flag: (1) `dict[str, Any]` or untyped `dict` used for data with a known schema, (2) string-keyed dict access (`data["user_id"]`, `data["status"]`) repeated across multiple call sites with the same keys, (3) `TypedDict` used where a Pydantic model or dataclass would be more appropriate (TypedDict provides type hints but no validation, no attribute access, no default values), (4) functions returning `dict` where the keys are predictable and documented, (5) JSON/API response data accessed via raw dict keys throughout business logic instead of being parsed into a model at the boundary. Reasons this is inappropriate: (1) **no attribute access** — `data["key"]` has no autocomplete, no go-to-definition, (2) **no validation** — missing keys or wrong types are runtime errors, not caught by type checkers, (3) **no refactorability** — renaming a key requires finding every string occurrence, (4) **typo fragility** — `data["stauts"]` silently returns `KeyError` at runtime, (5) **unclear schema** — the shape of the data is implicit, scattered across access sites. Fix: define a Pydantic `BaseModel` (preferred when validation or serialization is needed) or `@dataclass` (preferred for plain data containers), parse dicts into the model at system boundaries, and use the model throughout business logic.
  `rg 'dict\[str' --type py src/`
  `rg '\[.{1,20}\]' --type py src/` (look for repeated string-key access patterns)
  `rg 'TypedDict' --type py src/`
  Manual review: for each `dict` usage, check if the keys are fixed and known. Flag if a model would be more appropriate.

- **String literals instead of enums** - a string literal (e.g., `"buy"`, `"sell"`, `"pending"`, `"error"`) is used to represent a value from a fixed, known set of variants — in class fields, function parameters, return values, if/elif chains, dict keys, or comparisons — where an `Enum` or `StrEnum` would provide exhaustiveness, typo prevention, and refactorability. Reasons this is inappropriate: (1) **no exhaustiveness** — adding a new variant silently does the wrong thing at every unhandled site, (2) **typo fragility** — `"recieve"` vs `"receive"` silently does the wrong thing, (3) **no tooling support** — rename/find-all-references do not work on string values, (4) **unclear domain** — the set of valid values is implicit rather than explicit in a type definition. Fix: define an `Enum` or `StrEnum` with the known variants, replace all string occurrences.
  `rg '"[a-z_]{2,}"' --type py src/`
  Manual review: for each string literal in comparisons (`==`, `!=`, `if/elif`, `match/case`), check whether the set of possible values is fixed and known.

- **String-based enum matching** - an enum value is converted to a string (via `.value`, `.name`, `str()`, or f-string) and then matched/compared as a string instead of comparing enum members directly. Fix: compare enum members directly. If the string comes from an external source, parse it into the enum first.
  `rg '\.value\s*==\s*"' --type py src/`
  `rg '\.name\s*==\s*"' --type py src/`
  `rg 'str\(.*\)\s*==\s*"' --type py src/`

- **Backwards compatibility code** - any code (shims, feature flags, migration logic, version checks, deprecated re-exports, `_old` / `_v2` type variants, conditional deserialization, fallback parsing, `DeprecationWarning`, or `warnings.warn`) that exists solely to handle old data formats, old API shapes, or old serialized state. This codebase is not in production and everything is in development — backwards compatibility is never a concern. Fix: remove the compatibility code entirely.
  `rg 'DeprecationWarning|warnings\.warn|_old|_v[0-9]|_legacy|_compat|backwards|backward|migrate|migration' --type py -i src/`

- **Mutable default arguments** - a function uses a mutable default argument (`def f(x=[])`, `def f(x={})`, `def f(x=set())`). Fix: use `None` as default and create the mutable inside the function body.
  `rg 'def .*=\[\]|def .*=\{\}|def .*=set\(\)' --type py src/`

- **CLI built without Typer** - a Python CLI uses `argparse`, `click`, `optparse`, `sys.argv` parsing, or any other CLI framework instead of Typer. Typer is the standard CLI framework for this codebase. Fix: replace with `typer.Typer()` app, use `@app.command()` decorators, and use Typer's type-annotated parameter style.
  `rg 'import argparse|from argparse|import click|from click|import optparse|from optparse|sys\.argv' --type py src/`

- **Bare `*` imports** - `from module import *` used outside of `__init__.py`. Fix: import specific names.
  `rg 'from .* import \*' --type py src/`
  Exclude `__init__.py` files from results.

- **Manual tabular computation instead of Polars** - any non-trivial calculation on tabular data (filtering, grouping, aggregating, joining, pivoting, window functions, column arithmetic, sorting with ties, rolling calculations, or multi-step transforms) is implemented with manual Python loops, list comprehensions, dict accumulation, `csv` module row iteration, or pandas instead of Polars. Polars is the first-class citizen for all tabular data work in this codebase. Patterns to flag: (1) `for row in rows` / `for item in data` loops that filter, accumulate, or transform tabular records, (2) `collections.Counter`, `collections.defaultdict(list)`, or manual dict-of-lists used to group/aggregate rows, (3) nested comprehensions that pivot or reshape row-oriented data, (4) `csv.reader` / `csv.DictReader` followed by manual processing instead of `pl.read_csv`, (5) `import pandas` or `pd.DataFrame` used anywhere — pandas is not permitted; use the Polars equivalent, (6) manual `sum()` / `mean()` / `max()` / `min()` over extracted column values instead of Polars expressions, (7) `sorted(data, key=lambda ...)` on tabular records instead of `df.sort(...)`, (8) multi-step list/dict transforms that could be a single Polars expression chain. Exception: (1) trivial single-pass operations on fewer than ~5 items where Polars would add unnecessary weight, (2) operations that genuinely cannot be expressed as tabular transforms (graph traversal, recursive structures). Fix: load data into a `pl.DataFrame` or `pl.LazyFrame`, express the computation as Polars expressions, collect the result.
  `rg 'import pandas|from pandas|import csv|csv\.reader|csv\.DictReader' --type py src/`
  `rg 'defaultdict\(list\)|defaultdict\(int\)|Counter\(' --type py src/`
  Manual review: for each loop over tabular data, check if the operation could be a Polars expression chain.

---

## Low Frequency Checks

- **Wrapper class with no added behavior** - a class wraps a single attribute but adds no methods, no validation, and no logic beyond construction/access. Fix: use the inner type directly everywhere and delete the wrapper. Exception: (1) the wrapper enforces a semantic distinction (e.g., `Meters` vs `Feet`), (2) it provides validation in `__init__` or `__post_init__`.
  `rg 'class \w+' --type py src/`
  For each single-field class: check if it has any methods beyond `__init__`. Flag if not.

- **ABC/Protocol with single implementation** - an abstract base class or `Protocol` exists with exactly one concrete implementation and is never used for type-checking with `isinstance()` or in type annotations polymorphically. Fix: remove the ABC/Protocol and use the concrete class directly. Exception: (1) required by a framework, (2) enables mocking in tests that actually exist.
  `rg 'class.*ABC|class.*Protocol' --type py src/`
  For each: check how many concrete implementations exist and whether `isinstance()` checks or polymorphic type hints reference it.

- **Re-export-only `__init__.py`** - an `__init__.py` file contains only `from ... import ...` re-exports with no logic, constants, or type definitions. Fix: import directly from the defining modules at usage sites and simplify `__init__.py`.
  `find src/ -name '__init__.py' -exec wc -l {} +`
  For each: check if it contains anything beyond imports and `__all__`.

- **Unnecessary `__init__.py` intermediary** - a package directory's `__init__.py` contains only a single import from one child module and re-exports everything. The package exists solely to namespace a single child file. Fix: flatten the module structure.
  `find src/ -name '__init__.py' -exec wc -l {} +`
  Flag short `__init__.py` files (≤3 lines). For each: check if it only re-exports from one child.

- **Root-cause patterns** - existing code patterns that are themselves the root cause of the problem being audited. Fix: flag the pattern as the root cause, do not propose fixes that preserve it.
  Manual review only.
