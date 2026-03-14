#!/usr/bin/env python3
import json
import sys

def handle(request):
    method = request.get("method", "")
    req_id = request.get("id")
    params = request.get("params", {})

    if req_id is None:
        return None

    if method == "initialize":
        return {
            "jsonrpc": "2.0", "id": req_id,
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": {"tools": {}},
                "serverInfo": {"name": "test-server", "version": "0.1.0"}
            }
        }

    if method == "tools/list":
        return {
            "jsonrpc": "2.0", "id": req_id,
            "result": {
                "tools": [
                    {"name": "echo", "description": "Echo input back",
                     "inputSchema": {"type": "object"}},
                    {"name": "add", "description": "Add two numbers",
                     "inputSchema": {"type": "object",
                                     "properties": {"a": {"type": "number"},
                                                    "b": {"type": "number"}}}}
                ]
            }
        }

    if method == "tools/call":
        name = params.get("name", "")
        args = params.get("arguments", {})
        if name == "echo":
            return {
                "jsonrpc": "2.0", "id": req_id,
                "result": {"content": [{"type": "text", "text": json.dumps(args)}]}
            }
        if name == "add":
            total = args.get("a", 0) + args.get("b", 0)
            return {
                "jsonrpc": "2.0", "id": req_id,
                "result": {"content": [{"type": "text", "text": str(total)}]}
            }
        if name == "fail":
            return {
                "jsonrpc": "2.0", "id": req_id,
                "result": {
                    "content": [{"type": "text", "text": "tool error"}],
                    "isError": True
                }
            }
        return {
            "jsonrpc": "2.0", "id": req_id,
            "error": {"code": -32601, "message": f"Unknown tool: {name}"}
        }

    if method == "resources/list":
        return {
            "jsonrpc": "2.0", "id": req_id,
            "result": {"resources": [
                {"uri": "test://doc", "name": "doc", "mimeType": "text/plain"}
            ]}
        }

    if method == "resources/read":
        return {
            "jsonrpc": "2.0", "id": req_id,
            "result": {"contents": [
                {"uri": params.get("uri", ""), "text": "hello from resource",
                 "mimeType": "text/plain"}
            ]}
        }

    if method == "prompts/list":
        return {
            "jsonrpc": "2.0", "id": req_id,
            "result": {"prompts": [
                {"name": "greet", "description": "Greeting prompt",
                 "arguments": [{"name": "name", "required": True}]}
            ]}
        }

    if method == "prompts/get":
        name_arg = params.get("arguments", {}).get("name", "world")
        return {
            "jsonrpc": "2.0", "id": req_id,
            "result": {"messages": [
                {"role": "user", "content": {"type": "text",
                                             "text": f"Hello {name_arg}"}}
            ]}
        }

    return {
        "jsonrpc": "2.0", "id": req_id,
        "error": {"code": -32601, "message": f"Unknown method: {method}"}
    }

for line in sys.stdin:
    request = json.loads(line.strip())
    response = handle(request)
    if response is not None:
        sys.stdout.write(json.dumps(response) + "\n")
        sys.stdout.flush()
