#!/usr/bin/env python3
import json
import subprocess
import sys
import os

def main():
    if len(sys.argv) < 3:
        print("usage: yield_orchestrator.py <lx_script> <responses_json>", file=sys.stderr)
        sys.exit(1)

    script = sys.argv[1]
    responses = json.loads(sys.argv[2])
    yield_idx = 0
    yields_seen = []

    lx_bin = os.path.join(os.path.dirname(__file__), "..", "..", "target", "debug", "lx")
    proc = subprocess.Popen(
        [lx_bin, "run", script],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )

    output_lines = []
    while True:
        line = proc.stdout.readline()
        if not line:
            break
        line = line.rstrip("\n")
        try:
            msg = json.loads(line)
            if isinstance(msg, dict) and "__yield" in msg:
                yields_seen.append(msg["__yield"])
                if yield_idx < len(responses):
                    response = json.dumps(responses[yield_idx])
                    proc.stdin.write(response + "\n")
                    proc.stdin.flush()
                    yield_idx += 1
                else:
                    proc.stdin.write(json.dumps(None) + "\n")
                    proc.stdin.flush()
                continue
        except json.JSONDecodeError:
            pass
        output_lines.append(line)

    proc.wait()
    result = {
        "exit_code": proc.returncode,
        "yields": yields_seen,
        "output": "\n".join(output_lines),
        "stderr": proc.stderr.read(),
    }
    print(json.dumps(result))

if __name__ == "__main__":
    main()
