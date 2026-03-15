#!/usr/bin/env python3
import json
import os
from http.server import HTTPServer, BaseHTTPRequestHandler

class Handler(BaseHTTPRequestHandler):
    def log_message(self, *_args):
        pass

    def do_GET(self):
        if self.path == "/json":
            body = json.dumps({"message": "hello", "count": 42}).encode()
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("X-Custom", "test-value")
            self.end_headers()
            self.wfile.write(body)
        elif self.path == "/text":
            body = b"plain text response"
            self.send_response(200)
            self.send_header("Content-Type", "text/plain")
            self.end_headers()
            self.wfile.write(body)
        elif self.path == "/status/404":
            self.send_response(404)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps({"error": "not found"}).encode())
        elif self.path == "/echo-headers":
            headers_out = {}
            for key in ["xtest", "xauth"]:
                val = self.headers.get(key)
                if val:
                    headers_out[key] = val
            body = json.dumps(headers_out).encode()
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(body)
        else:
            self.send_response(200)
            self.send_header("Content-Type", "text/plain")
            self.end_headers()
            self.wfile.write(b"ok")

    def do_POST(self):
        length = int(self.headers.get("Content-Length", 0))
        body = self.rfile.read(length)
        request = json.loads(body) if body else {}
        response = {"received": request, "method": "POST"}
        payload = json.dumps(response).encode()
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.end_headers()
        self.wfile.write(payload)

    def do_PUT(self):
        length = int(self.headers.get("Content-Length", 0))
        body = self.rfile.read(length)
        request = json.loads(body) if body else {}
        response = {"received": request, "method": "PUT"}
        payload = json.dumps(response).encode()
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.end_headers()
        self.wfile.write(payload)

    def do_DELETE(self):
        response = {"method": "DELETE", "path": self.path}
        payload = json.dumps(response).encode()
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.end_headers()
        self.wfile.write(payload)


if __name__ == "__main__":
    server = HTTPServer(("127.0.0.1", 0), Handler)
    port = server.server_address[1]
    pid = os.getpid()
    with open("/tmp/lx_http_test_port", "w") as f:
        f.write(str(port))
    with open("/tmp/lx_http_test_pid", "w") as f:
        f.write(str(pid))
    print(port, flush=True)
    server.serve_forever()
