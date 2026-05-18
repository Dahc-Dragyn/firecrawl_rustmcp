# 🦀 Native Rust Firecrawl MCP Server

A blazing-fast, ultra-lightweight, and zero-runtime-dependency Model Context Protocol (MCP) server for Firecrawl, engineered in systems-level Rust.

This server replaces heavy, process-hogging Node.js/npm MCP instances with a standalone compiled native binary (~4.5 MB), consuming virtually zero idle memory (~3-5MB RAM) and utilizing the high-performance **Tokio** asynchronous event loop.

---

## 🛠️ Exposed Tools

The server automatically compiles and registers four robust tools mapping to the Firecrawl HTTP Engine:

1. **`firecrawl_scrape`**
   - **Description:** Scrapes a single URL and converts it into clean, LLM-ready markdown, stripping ads, trackers, headers, and navigation footers.
   - **Parameters:**
     - `url` (string, required): The target website URL.
     - `formats` (array of strings, optional): Formats to return (defaults to `["markdown"]`).
     - `only_main_content` (boolean, optional): Isolates main body blocks (defaults to `true`).

2. **`firecrawl_crawl`**
   - **Description:** Triggers an asynchronous recursive crawl job starting from a base URL up to a defined limit.
   - **Parameters:**
     - `url` (string, required): The starting website base URL.
     - `limit` (integer, optional): Maximum pages to crawl (defaults to `5`).

3. **`firecrawl_get_crawl_status`**
   - **Description:** Retrieves the status and extracted page results of an active crawling job.
   - **Parameters:**
     - `job_id` (string, required): The unique asynchronous crawl task ID.

4. **`firecrawl_map`**
   - **Description:** Maps out structural directories and sub-URLs of a target domain without downloading body text.
   - **Parameters:**
     - `url` (string, required): The domain to map.
     - `search` (string, optional): A filter query for structural sub-paths.

---

## 🚀 Quick Start: Installation & Setup

### Option 1: Using the Precompiled Binary (Windows Only)
A precompiled Windows executable is included in the repository at `bin/firecrawl_mcp.exe`. You can directly reference this executable in your MCP configurations without any dependencies.

### Option 2: Build From Source (Cross-Platform - Windows, macOS, Linux)
To compile the binary yourself, ensure you have the Rust compiler installed ([rustup.rs](https://rustup.rs/)).

```bash
# 1. Clone the repository
git clone https://github.com/Dahc-Dragyn/firecrawl_rustmcp.git
cd firecrawl_rustmcp

# 2. Build the binary in Release mode
cargo build --release
```

Once compiled, you can find the compiled standalone binary under:
* **Windows:** `target/release/firecrawl_mcp.exe`
* **macOS / Linux:** `target/release/firecrawl_mcp`

---

## 🔧 Environment & Integration Configuration

The server reads standard environment variables to connect to your Firecrawl engine:
* `FIRECRAWL_API_URL` (Optional): The endpoint of your Firecrawl API server (defaults to local self-hosted `http://localhost:3002`). Use `https://api.firecrawl.dev` to connect to the cloud engine.
* `FIRECRAWL_API_KEY` (Optional): Your Firecrawl API key (defaults to `local_development`).

### 1. Claude Desktop Integration
Update your `%APPDATA%\Claude\claude_desktop_config.json` (Windows) or `~/Library/Application Support/Claude/claude_desktop_config.json` (macOS/Linux). Ensure you use absolute paths and escape backslashes on Windows:

```json
{
  "mcpServers": {
    "firecrawl_rust": {
      "command": "/absolute/path/to/firecrawl_mcp",
      "args": [],
      "env": {
        "FIRECRAWL_API_URL": "http://localhost:3002",
        "FIRECRAWL_API_KEY": "local_development"
      }
    }
  }
}
```

### 2. Antigravity / Gemini Client Integration
Add the configuration block inside `%USERPROFILE%\.gemini\antigravity\mcp_config.json`:

```json
{
  "mcpServers": {
    "firecrawl_rust": {
      "command": "C:\\absolute\\path\\to\\firecrawl_mcp.exe",
      "args": [],
      "env": {
        "FIRECRAWL_API_URL": "http://localhost:3002",
        "FIRECRAWL_API_KEY": "local_development"
      }
    }
  }
}
```

### 3. Cursor or Cline Integration
In Cursor (Settings > Features > MCP) or Cline (MCP settings panel), register the server as a new **command-type MCP server**:
* **Name:** `firecrawl_rust`
* **Type:** `command`
* **Command:** `/absolute/path/to/firecrawl_mcp`
* **Environment Variables:**
  * `FIRECRAWL_API_URL` = `http://localhost:3002`
  * `FIRECRAWL_API_KEY` = `local_development`

---

## 🧪 Local Testing & Mock Validation

You can validate the MCP server locally without running a full Docker environment or paying for Cloud API credits using our Python test harness suite:

1. **Start the Mock Engine**:
   Runs a local HTTP server that mimics the exact success payloads of the Firecrawl API on port 3002:
   ```bash
   python mock_engine.py
   ```

2. **Run the Test Harness**:
   Runs a test client that spawns the compiled Rust binary, initializes the JSON-RPC handshake over standard I/O, calls the `firecrawl_scrape` tool, and prints the raw JSON-RPC responses and timing metadata:
   ```bash
   python harness.py
   ```

---

## 🏗️ Technical Spec & Architecture

- **Protocol Core:** Built using the Warp-maintained `rmcp` SDK (`v1.7.0`) ensuring strict conformance to the Model Context Protocol JSON-RPC specification.
- **Asynchronous Loop:** Powered by **Tokio Async I/O** for highly efficient concurrent execution on multiple threads.
- **HTTP Client:** Powered by **Reqwest** with native connection pooling, HTTP/2 support, and connection reuse.
- **Stdout Sandbox:** The server strictly routes all debugging, diagnostics, and telemetry messages to **Standard Error (`stderr`)** using `eprintln!`. This guarantees that the Standard Output (`stdout`) remains an unpolluted, 100% compliant JSON-RPC frame transport channel.
