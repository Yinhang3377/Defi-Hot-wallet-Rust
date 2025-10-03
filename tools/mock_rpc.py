#!/usr/bin/env python3
"""
A minimal JSON-RPC mock server to simulate basic blockchain RPCs used by tests.
This is intentionally tiny: it returns canned responses for a few common methods.
"""
import json
from http.server import BaseHTTPRequestHandler, HTTPServer

HOST = '0.0.0.0'
PORT = 8545

class Handler(BaseHTTPRequestHandler):
    def _set_headers(self):
        self.send_response(200)
        self.send_header('Content-type', 'application/json')
        self.end_headers()

    def do_POST(self):
        length = int(self.headers.get('content-length', 0))
        body = self.rfile.read(length).decode('utf-8')
        try:
            req = json.loads(body)
        except Exception:
            self._set_headers()
            self.wfile.write(json.dumps({'error': 'invalid json'}).encode())
            return

        method = req.get('method')
        resp = {'jsonrpc': '2.0', 'id': req.get('id', 1)}

        # Basic canned responses
        if method in ('eth_blockNumber',):
            resp['result'] = hex(100)
        elif method in ('eth_getBalance',):
            resp['result'] = hex(10**18)
        elif method in ('eth_sendRawTransaction',):
            resp['result'] = '0xmocktx'
        elif method in ('getStatus', 'get_account_info'):
            resp['result'] = {'status': 'ok'}
        else:
            resp['result'] = None

        self._set_headers()
        self.wfile.write(json.dumps(resp).encode())

    def log_message(self, format, *args):
        # silence default logging
        return

if __name__ == '__main__':
    server = HTTPServer((HOST, PORT), Handler)
    print(f"Mock RPC listening on {HOST}:{PORT}")
    server.serve_forever()
