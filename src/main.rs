use rmcp::{tool, tool_router, ServiceExt, ErrorData};
use rmcp::model::{CallToolResult, ErrorCode};
use rmcp::handler::server::wrapper::Parameters;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;
use std::env;
use tokio::io::{stdin, stdout};

/// Config state for bridging to the Firecrawl HTTP Engine
#[derive(Clone)]
struct FirecrawlServer {
    api_url: String,
    api_key: String,
    http_client: reqwest::Client,
}

// =========================================================================
// 📥 Input Parameter Schemas (For Automated JSON-RPC Schema Generation)
// =========================================================================

#[derive(Deserialize, JsonSchema)]
struct ScrapeParams {
    /// The target URL to scrape
    url: String,
    /// List of target formats to return, e.g., ["markdown"] or ["html"]
    formats: Option<Vec<String>>,
    /// Set to true to isolate and return only the main content blocks
    only_main_content: Option<bool>,
}

#[derive(Deserialize, JsonSchema)]
struct CrawlParams {
    /// The starting base URL to crawl
    url: String,
    /// Maximum number of pages to crawl (Default: 5)
    limit: Option<u32>,
}

#[derive(Deserialize, JsonSchema)]
struct CrawlStatusParams {
    /// The unique asynchronous crawl task ID
    job_id: String,
}

#[derive(Deserialize, JsonSchema)]
struct MapParams {
    /// The base domain to discover
    url: String,
    /// Optional query search filter for discovering sub-directories
    search: Option<String>,
}

// =========================================================================
// 🦀 Server Implementation & Tool Router
// =========================================================================

#[tool_router(server_handler)]
impl FirecrawlServer {
    pub fn new(api_url: String, api_key: String) -> Self {
        Self {
            api_url,
            api_key,
            http_client: reqwest::Client::new(),
        }
    }

    #[tool(
        name = "firecrawl_scrape",
        description = "Scrapes a single URL and converts it into clean, LLM-ready markdown or text."
    )]
    async fn scrape_url(&self, params: Parameters<ScrapeParams>) -> Result<CallToolResult, ErrorData> {
        let params = params.0;
        eprintln!("[Firecrawl MCP] Executing scrape for: {}", params.url);
        
        let scrape_endpoint = format!("{}/v1/scrape", self.api_url);
        let payload = serde_json::json!({
            "url": params.url,
            "formats": params.formats.unwrap_or_else(|| vec!["markdown".to_string()]),
            "onlyMainContent": params.only_main_content.unwrap_or(true),
        });

        let response = self.http_client
            .post(&scrape_endpoint)
            .bearer_auth(&self.api_key)
            .json(&payload)
            .send()
            .await
            .map_err(|e| ErrorData::new(ErrorCode(500), format!("HTTP Connection failed: {}", e), None))?;

        let body: Value = response
            .json()
            .await
            .map_err(|e| ErrorData::new(ErrorCode(500), format!("Failed parsing API JSON: {}", e), None))?;

        Ok(CallToolResult::structured(body))
    }

    #[tool(
        name = "firecrawl_crawl",
        description = "Asynchronously starts a crawl job on a target website domain."
    )]
    async fn crawl_website(&self, params: Parameters<CrawlParams>) -> Result<CallToolResult, ErrorData> {
        let params = params.0;
        eprintln!("[Firecrawl MCP] Starting asynchronous crawl for: {}", params.url);

        let crawl_endpoint = format!("{}/v1/crawl", self.api_url);
        let payload = serde_json::json!({
            "url": params.url,
            "limit": params.limit.unwrap_or(5),
        });

        let response = self.http_client
            .post(&crawl_endpoint)
            .bearer_auth(&self.api_key)
            .json(&payload)
            .send()
            .await
            .map_err(|e| ErrorData::new(ErrorCode(500), format!("HTTP Connection failed: {}", e), None))?;

        let body: Value = response
            .json()
            .await
            .map_err(|e| ErrorData::new(ErrorCode(500), format!("Failed parsing API JSON: {}", e), None))?;

        Ok(CallToolResult::structured(body))
    }

    #[tool(
        name = "firecrawl_get_crawl_status",
        description = "Retrieves the status and scraped results of an ongoing crawl job."
    )]
    async fn get_crawl_status(&self, params: Parameters<CrawlStatusParams>) -> Result<CallToolResult, ErrorData> {
        let params = params.0;
        eprintln!("[Firecrawl MCP] Checking status of crawl job: {}", params.job_id);

        let status_endpoint = format!("{}/v1/crawl/{}", self.api_url, params.job_id);

        let response = self.http_client
            .get(&status_endpoint)
            .bearer_auth(&self.api_key)
            .send()
            .await
            .map_err(|e| ErrorData::new(ErrorCode(500), format!("HTTP Connection failed: {}", e), None))?;

        let body: Value = response
            .json()
            .await
            .map_err(|e| ErrorData::new(ErrorCode(500), format!("Failed parsing API JSON: {}", e), None))?;

        Ok(CallToolResult::structured(body))
    }

    #[tool(
        name = "firecrawl_map",
        description = "Discovers and maps out all structural sub-URLs of a website."
    )]
    async fn map_website(&self, params: Parameters<MapParams>) -> Result<CallToolResult, ErrorData> {
        let params = params.0;
        eprintln!("[Firecrawl MCP] Mapping structural nodes for: {}", params.url);

        let map_endpoint = format!("{}/v1/map", self.api_url);
        let mut payload = serde_json::json!({
            "url": params.url,
        });

        if let Some(ref search) = params.search {
            if let Some(obj) = payload.as_object_mut() {
                obj.insert("search".to_string(), serde_json::json!(search));
            }
        }

        let response = self.http_client
            .post(&map_endpoint)
            .bearer_auth(&self.api_key)
            .json(&payload)
            .send()
            .await
            .map_err(|e| ErrorData::new(ErrorCode(500), format!("HTTP Connection failed: {}", e), None))?;

        let body: Value = response
            .json()
            .await
            .map_err(|e| ErrorData::new(ErrorCode(500), format!("Failed parsing API JSON: {}", e), None))?;

        Ok(CallToolResult::structured(body))
    }
}

// =========================================================================
// 🚀 Main Entry Point (Asynchronous Server Event Loop)
// =========================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Load configuration from environment variables
    let api_url = env::var("FIRECRAWL_API_URL")
        .unwrap_or_else(|_| "http://localhost:3002".to_string());
    let api_key = env::var("FIRECRAWL_API_KEY")
        .unwrap_or_else(|_| "local_development".to_string());

    // CRITICAL: Writing to stderr ensures stdout remains clean for JSON-RPC frames
    eprintln!("[Firecrawl MCP] Initializing Native Rust Server...");
    eprintln!("[Firecrawl MCP] Connecting to engine at: {}", api_url);

    // 2. Instantiate server state
    let service = FirecrawlServer::new(api_url, api_key);

    // 3. Bind standard I/O streams and start the service
    let transport = (stdin(), stdout());
    let server = service.serve(transport).await?;

    // 4. Await shutdown/interruption
    server.waiting().await?;
    
    eprintln!("[Firecrawl MCP] Server gracefully stopped.");
    Ok(())
}
