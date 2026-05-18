import json
from http.server import HTTPServer, BaseHTTPRequestHandler

class MockFirecrawlHandler(BaseHTTPRequestHandler):
    def do_POST(self):
        if self.path == '/v1/scrape':
            # Read content length
            content_length = int(self.headers.get('Content-Length', 0))
            post_data = self.rfile.read(content_length).decode('utf-8')
            req_json = json.loads(post_data)
            
            target_url = req_json.get('url', 'https://example.com')
            print(f"[Mock Engine] Received scrape request for: {target_url}")
            
            # Send success headers
            self.send_response(200)
            self.send_header('Content-Type', 'application/json')
            self.end_headers()
            
            # Construct standard Firecrawl scrape response
            response_data = {
                "success": True,
                "data": {
                    "markdown": f"# Example Domain\n\nThis domain is for use in illustrative examples in documents. You may use this domain in literature without prior coordination or asking for permission.\n\n[More information...](https://www.iana.org/domains/reserved)",
                    "html": "<h1>Example Domain</h1>",
                    "metadata": {
                        "title": "Example Domain",
                        "statusCode": 200,
                        "sourceUrl": target_url
                    }
                }
            }
            
            self.wfile.write(json.dumps(response_data).encode('utf-8'))
        else:
            self.send_response(404)
            self.end_headers()

    def do_GET(self):
        self.send_response(200)
        self.send_header('Content-Type', 'application/json')
        self.end_headers()
        self.wfile.write(json.dumps({"status": "running"}).encode('utf-8'))

def run(server_class=HTTPServer, handler_class=MockFirecrawlHandler, port=3002):
    server_address = ('127.0.0.1', port)
    httpd = server_class(server_address, handler_class)
    print(f"[Mock Engine] Starting local mock Firecrawl Engine on port {port}...")
    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        pass
    print("[Mock Engine] Stopping mock engine.")

if __name__ == '__main__':
    run()
