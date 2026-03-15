import json, sys
from collections import defaultdict

SKIP_PREFIXES = ("crates/fortigate-client/src/generated/",)
ALLOWED_PREFIXES = ("crates/", "mcps/", "apps/", "services/")


def parse_cargo_json(lines):
    items = []
    for line in lines:
        try:
            msg = json.loads(line)
        except json.JSONDecodeError:
            continue
        if msg.get("reason") != "compiler-message":
            continue
        m = msg["message"]
        spans = m.get("spans", [])
        if not spans:
            continue
        f = spans[0]["file_name"]
        if any(f.startswith(s) for s in SKIP_PREFIXES):
            continue
        if not f.startswith(ALLOWED_PREFIXES):
            continue
        code = (m.get("code") or {}).get("code", "unknown")
        key = f"{f}:{spans[0]['line_start']}:{code}"
        rendered = m["rendered"].split("\n")[0]
        prefix = f"{f}:{spans[0]['line_start']}:"
        col_prefix = f"{f}:{spans[0]['line_start']}:{spans[0].get('column_start', '')}:"
        for p in (col_prefix, prefix):
            if rendered.startswith(p):
                rendered = rendered[len(p):].strip()
                break
        if rendered.startswith(("warning: ", "error: ")):
            rendered = rendered.split(": ", 1)[1]
        items.append((key, {
            "level": m["level"],
            "file": f,
            "line": spans[0]["line_start"],
            "code": code,
            "msg": rendered,
        }))
    seen = set()
    deduped = []
    for key, item in items:
        if key not in seen:
            seen.add(key)
            deduped.append(item)
    return deduped


def group_and_format(deduped):
    groups = defaultdict(list)
    for item in deduped:
        krate = "/".join(item["file"].split("/")[:2])
        groups[krate].append(item)
    total_e = sum(1 for i in deduped if i["level"] == "error")
    total_w = sum(1 for i in deduped if i["level"] == "warning")
    output = [f"DIAGNOSE: {total_e} errors, {total_w} warnings", ""]
    for krate in sorted(groups, key=lambda k: (-sum(1 for i in groups[k] if i["level"] == "error"),
                                                -sum(1 for i in groups[k] if i["level"] == "warning"))):
        g = groups[krate]
        e = sum(1 for i in g if i["level"] == "error")
        w = sum(1 for i in g if i["level"] == "warning")
        if e == 0 and w == 0:
            continue
        output.append(f"## {krate} ({e}E / {w}W)")
        for i in g:
            output.append(f"  {i['file']}:{i['line']} {i['code']}: {i['msg']}")
        output.append("")
    return output, total_e


def run(lines):
    deduped = parse_cargo_json(lines)
    output, total_e = group_and_format(deduped)
    for line in output:
        print(line)
    if total_e > 0:
        sys.exit(1)
    print("diagnose: ok")
