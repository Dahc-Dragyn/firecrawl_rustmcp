import os
import subprocess
import json
import time
import sys

def main():
    # Dynamically find the binary path based on environment
    possible_paths = [
        os.path.join("target", "release", "firecrawl_mcp.exe"),
        os.path.join("target", "release", "firecrawl_mcp"),
        os.path.join("bin", "firecrawl_mcp.exe"),
        os.path.join("bin", "firecrawl_mcp"),
        r"c:\Antigravity projects\Rust\firecrawl\firecrawl_mcp\bin\firecrawl_mcp.exe"
    ]
    
    binary_path = None
    for path in possible_paths:
        if os.path.exists(path):
            binary_path = path
            break
            
    if not binary_path:
        print("[Harness] ERROR: Could not find compiled binary 'firecrawl_mcp'!")
        print("[Harness] Please run 'cargo build --release' first.")
        sys.exit(1)
        
    print(f"[Harness] Launching Native Rust MCP Server from: {binary_path}...")
    process = subprocess.Popen(
        [binary_path],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        bufsize=1
    )
    
    # Wait a fraction of a second for startup
    time.sleep(0.2)
    
    # 1. Send Initialize Request
    init_request = {
        "jsonrpc": "2.0",
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "HarnessClient",
                "version": "1.0.0"
            }
        },
        "id": 1
    }
    
    print("[Harness] Sending 'initialize' request...")
    process.stdin.write(json.dumps(init_request) + "\n")
    process.stdin.flush()
    
    # Read response
    init_response_raw = process.stdout.readline()
    init_response = json.loads(init_response_raw)
    print("[Harness] Received 'initialize' response.")
    
    # 2. Send Initialized Notification
    initialized_notification = {
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    }
    print("[Harness] Sending 'notifications/initialized' notification...")
    process.stdin.write(json.dumps(initialized_notification) + "\n")
    process.stdin.flush()
    
    # 3. Call firecrawl_scrape Tool
    tool_request = {
        "jsonrpc": "2.0",
        "method": "tools/call",
        "params": {
            "name": "firecrawl_scrape",
            "arguments": {
                "url": "https://example.com"
            }
        },
        "id": 2
    }
    
    print("[Harness] Sending 'tools/call' for 'firecrawl_scrape'...")
    start_time = time.time()
    
    process.stdin.write(json.dumps(tool_request) + "\n")
    process.stdin.flush()
    
    # Read tool call response
    response_raw = process.stdout.readline()
    end_time = time.time()
    
    latency_ms = (end_time - start_time) * 1000
    print(f"[Harness] Received tool call response in {latency_ms:.2f} ms.")
    
    try:
        response_json = json.loads(response_raw)
        print("\n=== [JSON-RPC Response Payload] ===")
        print(json.dumps(response_json, indent=2))
        print("===================================\n")
    except Exception as e:
        print(f"[Harness] Failed to parse JSON response: {e}")
        print(f"[Harness] Raw response: {response_raw}")
        
    # Read and print stderr diagnostics
    print("=== [Server Stderr Logs] ===")
    process.stdin.close()
    for line in process.stderr:
        print(line.strip())
    print("============================")
    
    process.terminate()
 
if __name__ == '__main__':
    main()
