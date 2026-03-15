#!/usr/bin/env python3
import json
import os
from http.server import HTTPServer, BaseHTTPRequestHandler

SESSION_ID = "lx-test-session"

class Handler(BaseHTTPRequestHandler):
    def log_message(self, *_args):
        pass

    def do_POST(self):
        length = int(self.headers.get("Content-Length", 0))
        body = self.rfile.read(length)
        request = json.loads(body)
        method = request.get("method", "")
        req_id = request.get("id")
        params = request.get("params", {})

        if req_id is None:
            self.send_response(202)
            self.send_header("Mcp-Session-Id", SESSION_ID)
            self.end_headers()
            return

        result = self.dispatch(method, params)
        if "error" in result:
            response = {"jsonrpc": "2.0", "id": req_id, "error": result["error"]}
        else:
            response = {"jsonrpc": "2.0", "id": req_id, "result": result}

        payload = json.dumps(response).encode()
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Mcp-Session-Id", SESSION_ID)
        self.end_headers()
        self.wfile.write(payload)

    def do_DELETE(self):
        self.send_response(200)
        self.end_headers()

    def dispatch(self, method, params):
        if method == "initialize":
            return {
                "protocolVersion": "2024-11-05",
                "capabilities": {"tools": {}},
                "serverInfo": {"name": "test-http-server", "version": "0.1.0"},
            }
        if method == "tools/list":
            return {
                "tools": [
                    {"name": "echo", "description": "Echo input",
                     "inputSchema": {"type": "object"}},
                    {"name": "add", "description": "Add two numbers",
                     "inputSchema": {"type": "object",
                                     "properties": {"a": {"type": "number"},
                                                    "b": {"type": "number"}}}},
                ]
            }
        if method == "tools/call":
            name = params.get("name", "")
            args = params.get("arguments", {})
            if name == "echo":
                return {"content": [{"type": "text", "text": json.dumps(args)}]}
            if name == "add":
                total = args.get("a", 0) + args.get("b", 0)
                return {"content": [{"type": "text", "text": str(total)}]}
            if name == "fail":
                return {"content": [{"type": "text", "text": "tool error"}],
                        "isError": True}
            return {"error": {"code": -32601, "message": f"Unknown tool: {name}"}}
        if method == "resources/list":
            return {"resources": [
                {"uri": "test://doc", "name": "doc", "mimeType": "text/plain"}
            ]}
        if method == "resources/read":
            return {"contents": [
                {"uri": params.get("uri", ""), "text": "hello from http resource",
                 "mimeType": "text/plain"}
            ]}
        if method == "prompts/list":
            return {"prompts": [
                {"name": "greet", "description": "Greeting prompt",
                 "arguments": [{"name": "name", "required": True}]}
            ]}
        if method == "prompts/get":
            name_arg = params.get("arguments", {}).get("name", "world")
            return {"messages": [
                {"role": "user",
                 "content": {"type": "text", "text": f"Hello {name_arg}"}}
            ]}
        return {"error": {"code": -32601, "message": f"Unknown method: {method}"}}


if __name__ == "__main__":
    server = HTTPServer(("127.0.0.1", 0), Handler)
    port = server.server_address[1]
    pid = os.getpid()
    with open("/tmp/lx_mcp_http_port", "w") as f:
        f.write(str(port))
    with open("/tmp/lx_mcp_http_pid", "w") as f:
        f.write(str(pid))
    print(port, flush=True)
    server.serve_forever()
