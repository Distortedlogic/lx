# Goal

Add `pkg/kit/template` — a lightweight template engine for code generation and scaffolding. Supports variable substitution, conditionals, and iteration. Pure lx, no Rust changes.

# Why

- Scaffolding agents generate boilerplate code from templates. String interpolation handles simple cases but not conditionals or loops over lists.
- Kiro's spec-driven development generates code from structured specifications. A template engine enables the same pattern in lx.
- Must be lightweight — not a full Jinja/Handlebars port, just enough for code generation: variable substitution (`{{var}}`), conditionals (`{{#if cond}}...{{/if}}`), iteration (`{{#each list}}...{{/each}}`), and inclusion (`{{> partial}}`).

# What Changes

**New file `pkg/kit/template.lx`:** Template engine with render, from_string, load, and partial support.

Syntax:
- `{{var}}` — variable substitution from context record
- `{{var.field}}` — dot-access into nested records
- `{{#if var}}...{{/if}}` — conditional (truthy: non-empty string, non-zero, non-None, non-empty list)
- `{{#if var}}...{{#else}}...{{/if}}` — conditional with else
- `{{#each list}}{{.}}{{/each}}` — iteration, `.` is current item, `@index` is 0-based index
- `{{#each list}}{{.name}}{{/each}}` — iteration with field access on each item

# Files Affected

- `pkg/kit/template.lx` — New file
- `tests/105_template.lx` — New test file

# Task List

### Task 1: Create pkg/kit/template.lx

**Subject:** Create template.lx with render, from_string, and load functions

**Description:** Create `pkg/kit/template.lx`:

```
-- Template engine -- lightweight code generation with {{var}}, {{#if}}, {{#each}}.
-- Not a full Handlebars port. Just enough for scaffolding and boilerplate.

use std/fs
use std/re

+from_string = (template_text) {
  {__template: true  text: template_text  partials: {}}
}

+load = (path) {
  text = fs.read path ^
  {__template: true  text: text  partials: {}}
}

+with_partial = (tmpl name partial_tmpl) {
  partials = tmpl.partials
  {..tmpl  partials: {..partials  (name): partial_tmpl.text}}
}

+render = (tmpl ctx) {
  text = tmpl.text
  partials = tmpl.partials ?? {}
  render_text text ctx partials
}

render_text = (text ctx partials) {
  text = resolve_partials text partials ctx
  text = resolve_each_blocks text ctx partials
  text = resolve_if_blocks text ctx partials
  text = resolve_vars text ctx
  text
}

resolve_vars = (text ctx) {
  re.replace text r/\{\{([a-zA-Z0-9_.]+)\}\}/ (m) {
    key = m.[1]
    lookup_path ctx key | to_str
  }
}

lookup_path = (ctx path) {
  parts = path | split "."
  parts | fold ctx (acc part) {
    acc == None ? None : {
      part == "." ? acc : {
        r = acc.(part) ?? None
        r
      }
    }
  }
}

resolve_if_blocks = (text ctx partials) {
  -- Process {{#if var}}...{{/if}} and {{#if var}}...{{#else}}...{{/if}}
  re.replace text r/\{\{#if ([a-zA-Z0-9_.]+)\}\}([\s\S]*?)\{\{\/if\}\}/ (m) {
    key = m.[1]
    body = m.[2]
    val = lookup_path ctx key
    truthy = is_truthy val

    -- Check for {{#else}} split
    else_parts = body | split "{{#else}}"
    else_parts | len > 1 ? {
      truthy ?
        (render_text (else_parts.[0]) ctx partials) :
        (render_text (else_parts.[1]) ctx partials)
    } : {
      truthy ? (render_text body ctx partials) : ""
    }
  }
}

resolve_each_blocks = (text ctx partials) {
  re.replace text r/\{\{#each ([a-zA-Z0-9_.]+)\}\}([\s\S]*?)\{\{\/each\}\}/ (m) {
    key = m.[1]
    body = m.[2]
    list = lookup_path ctx key
    list == None ? "" : {
      list | enumerate | map (pair) {
        idx = pair.[0]
        item = pair.[1]
        item_ctx = type_of item == "Record" ?
          {..ctx  ".": item  "@index": idx  ..item} :
          {..ctx  ".": item  "@index": idx}
        render_text body item_ctx partials
      } | join ""
    }
  }
}

resolve_partials = (text partials ctx) {
  re.replace text r/\{\{> ([a-zA-Z0-9_]+)\}\}/ (m) {
    name = m.[1]
    partial_text = partials.(name) ?? ""
    partial_text != "" ? (render_text partial_text ctx partials) : ""
  }
}

is_truthy = (val) {
  val == None ? false :
  val == false ? false :
  val == 0 ? false :
  val == "" ? false :
  (type_of val == "List" ? (val | len > 0) : true)
}
```

The regex patterns need to match the lx regex syntax (`r/.../`). The `re.replace` function takes a regex, input, and replacement function. Adjust patterns based on `std/re` API: `re.replace text pattern fn` where fn receives match groups.

Note: `re.replace` in std/re takes `(text pattern replacement)` where replacement is a Str, not a callback function. If callback-based replacement isn't supported, implement `resolve_vars` and `resolve_if_blocks` etc. using `re.find_all` + manual string manipulation instead. Check `std/re` API and adapt.

**ActiveForm:** Creating template.lx with render engine

---

### Task 2: Write tests for pkg/kit/template

**Subject:** Write tests covering variable substitution, conditionals, and iteration

**Description:** Create `tests/105_template.lx`:

```
use pkg/kit/template

-- Variable substitution
tmpl = template.from_string "Hello {{name}}, you are {{age}} years old."
result = template.render tmpl {name: "Claude"  age: 3}
assert (result == "Hello Claude, you are 3 years old.") "variable substitution"

-- Nested dot access
tmpl2 = template.from_string "{{user.name}} ({{user.role}})"
result2 = template.render tmpl2 {user: {name: "Alice"  role: "admin"}}
assert (result2 == "Alice (admin)") "dot access"

-- Conditionals
tmpl3 = template.from_string "{{#if admin}}Admin{{/if}}"
assert (template.render tmpl3 {admin: true} == "Admin") "if true"
assert (template.render tmpl3 {admin: false} == "") "if false"

-- Conditional with else
tmpl4 = template.from_string "{{#if premium}}Pro{{#else}}Free{{/if}}"
assert (template.render tmpl4 {premium: true} == "Pro") "if-else true"
assert (template.render tmpl4 {premium: false} == "Free") "if-else false"

-- Each loop
tmpl5 = template.from_string "{{#each items}}[{{.}}]{{/each}}"
result5 = template.render tmpl5 {items: ["a" "b" "c"]}
assert (result5 == "[a][b][c]") "each with primitives"

-- Each with records
tmpl6 = template.from_string "{{#each users}}{{name}},{{/each}}"
result6 = template.render tmpl6 {users: [{name: "A"} {name: "B"}]}
assert (result6 == "A,B,") "each with records"

-- Partials
header = template.from_string "HEADER"
tmpl7 = template.from_string "{{> hdr}} body"
tmpl7 = template.with_partial tmpl7 "hdr" header
result7 = template.render tmpl7 {}
assert (result7 == "HEADER body") "partial inclusion"

log.info "105_template: all passed"
```

Run `just test` to verify.

**ActiveForm:** Writing tests for template package

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.

---

## Task Loading Instructions

To execute this work item, load it with the workflow MCP:

```
mcp__workflow__load_work_item({ path: "work_items/PKG_TEMPLATE.md" })
```

Then call `next_task` to begin.
