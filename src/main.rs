use axum::{
    extract::Query,
    http::HeaderMap,
    response::{IntoResponse, Response, Sse},
    routing::{get, post},
    Json, Router,
};
use futures::stream;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{collections::HashMap, convert::Infallible, time::Duration};
use tokio_stream::StreamExt;
use tower_http::cors::CorsLayer;

// MCP Protocol Types
#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

// Tool Types
#[derive(Debug, Serialize)]
struct Tool {
    name: String,
    description: String,
    #[serde(rename = "inputSchema")]
    input_schema: Value,
}

// Calculator request types
#[derive(Debug, Deserialize)]
struct AdditionParams {
    a: f64,
    b: f64,
}

#[derive(Debug, Deserialize)]
struct MultiplicationParams {
    a: f64,
    b: f64,
}

#[derive(Debug, Deserialize)]
struct SquareParams {
    number: f64,
}

#[derive(Debug, Deserialize)]
struct SqrtParams {
    number: f64,
}

#[derive(Debug, Deserialize)]
struct ToolCallParams {
    name: String,
    arguments: Value,
}

// MCP Server Implementation
struct McpServer {
    server_info: Value,
    tools: Vec<Tool>,
}

impl McpServer {
    fn new() -> Self {
        let server_info = json!({
            "name": "Calculator MCP Server",
            "version": "1.0.0",
            "protocolVersion": "2024-11-05"
        });

        let tools = vec![
            Tool {
                name: "add".to_string(),
                description: "Add two numbers together".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "a": {
                            "type": "number",
                            "description": "The first number to add"
                        },
                        "b": {
                            "type": "number",
                            "description": "The second number to add"
                        }
                    },
                    "required": ["a", "b"]
                }),
            },
            Tool {
                name: "multiply".to_string(),
                description: "Multiply two numbers together".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "a": {
                            "type": "number",
                            "description": "The first number to multiply"
                        },
                        "b": {
                            "type": "number",
                            "description": "The second number to multiply"
                        }
                    },
                    "required": ["a", "b"]
                }),
            },
            Tool {
                name: "square".to_string(),
                description: "Calculate the square of a number".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "number": {
                            "type": "number",
                            "description": "The number to square"
                        }
                    },
                    "required": ["number"]
                }),
            },
            Tool {
                name: "sqrt".to_string(),
                description: "Calculate the square root of a number".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "number": {
                            "type": "number",
                            "description": "The number to find square root of (must be non-negative)"
                        }
                    },
                    "required": ["number"]
                }),
            },
        ];

        Self {
            server_info,
            tools,
        }
    }

    fn handle_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        match request.method.as_str() {
            "initialize" => self.handle_initialize(request.id),
            "tools/list" => self.handle_tools_list(request.id),
            "tools/call" => self.handle_tools_call(request.id, request.params),
            _ => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32601,
                    message: "Method not found".to_string(),
                    data: None,
                }),
            },
        }
    }

    fn handle_initialize(&self, id: Option<Value>) -> JsonRpcResponse {
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": self.server_info
            })),
            error: None,
        }
    }

    fn handle_tools_list(&self, id: Option<Value>) -> JsonRpcResponse {
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({
                "tools": self.tools
            })),
            error: None,
        }
    }

    fn handle_tools_call(&self, id: Option<Value>, params: Option<Value>) -> JsonRpcResponse {
        let params = match params {
            Some(p) => p,
            None => {
                return JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32602,
                        message: "Invalid params".to_string(),
                        data: None,
                    }),
                }
            }
        };

        let tool_call: ToolCallParams = match serde_json::from_value(params) {
            Ok(tc) => tc,
            Err(e) => {
                return JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32602,
                        message: format!("Invalid params: {}", e),
                        data: None,
                    }),
                }
            }
        };

        match tool_call.name.as_str() {
            "add" => self.handle_addition(id, tool_call.arguments),
            "multiply" => self.handle_multiplication(id, tool_call.arguments),
            "square" => self.handle_square(id, tool_call.arguments),
            "sqrt" => self.handle_sqrt(id, tool_call.arguments),
            _ => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32602,
                    message: "Unknown tool".to_string(),
                    data: None,
                }),
            },
        }
    }

    fn handle_addition(&self, id: Option<Value>, arguments: Value) -> JsonRpcResponse {
        let params: AdditionParams = match serde_json::from_value(arguments) {
            Ok(p) => p,
            Err(e) => {
                return JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32602,
                        message: format!("Invalid addition parameters: {}", e),
                        data: None,
                    }),
                }
            }
        };

        let result = params.a + params.b;
        println!("Performed addition: {} + {} = {}", params.a, params.b, result);

        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({
                "content": [{
                    "type": "text",
                    "text": format!("{} + {} = {}", params.a, params.b, result)
                }]
            })),
            error: None,
        }
    }

    fn handle_multiplication(&self, id: Option<Value>, arguments: Value) -> JsonRpcResponse {
        let params: MultiplicationParams = match serde_json::from_value(arguments) {
            Ok(p) => p,
            Err(e) => {
                return JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32602,
                        message: format!("Invalid multiplication parameters: {}", e),
                        data: None,
                    }),
                }
            }
        };

        let result = params.a * params.b;
        println!("Performed multiplication: {} × {} = {}", params.a, params.b, result);

        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({
                "content": [{
                    "type": "text",
                    "text": format!("{} × {} = {}", params.a, params.b, result)
                }]
            })),
            error: None,
        }
    }

    fn handle_square(&self, id: Option<Value>, arguments: Value) -> JsonRpcResponse {
        let params: SquareParams = match serde_json::from_value(arguments) {
            Ok(p) => p,
            Err(e) => {
                return JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32602,
                        message: format!("Invalid square parameters: {}", e),
                        data: None,
                    }),
                }
            }
        };

        let result = params.number * params.number;
        println!("Performed square: {}² = {}", params.number, result);

        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({
                "content": [{
                    "type": "text",
                    "text": format!("{}² = {}", params.number, result)
                }]
            })),
            error: None,
        }
    }

    fn handle_sqrt(&self, id: Option<Value>, arguments: Value) -> JsonRpcResponse {
        let params: SqrtParams = match serde_json::from_value(arguments) {
            Ok(p) => p,
            Err(e) => {
                return JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32602,
                        message: format!("Invalid sqrt parameters: {}", e),
                        data: None,
                    }),
                }
            }
        };

        if params.number < 0.0 {
            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32602,
                    message: format!("Cannot calculate square root of negative number: {}", params.number),
                    data: None,
                }),
            };
        }

        let result = params.number.sqrt();
        println!("Performed square root: √{} = {}", params.number, result);

        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({
                "content": [{
                    "type": "text",
                    "text": format!("√{} = {}", params.number, result)
                }]
            })),
            error: None,
        }
    }
}

// SSE Handler
async fn sse_handler(
    Query(_params): Query<HashMap<String, String>>,
    _headers: HeaderMap,
) -> Response {
    println!("SSE connection established");
    
    let server = McpServer::new();
    
    // Create a stream that handles incoming messages
    let stream = stream::unfold(server, |server| async move {
        // Simulate initialize request
        let init_request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(1)),
            method: "initialize".to_string(),
            params: Some(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "cursor-client",
                    "version": "1.0.0"
                }
            })),
        };
        
        let response = server.handle_request(init_request);
        let event_data = serde_json::to_string(&response).unwrap_or_else(|_| "{}".to_string());
        
        Some((
            Ok::<_, Infallible>(axum::response::sse::Event::default()
                .data(event_data)
                .event("message")),
            server,
        ))
    })
    .take(1); // Just send one response for demo

    Sse::new(stream)
        .keep_alive(axum::response::sse::KeepAlive::new().interval(Duration::from_secs(30)))
        .into_response()
}

// JSON-RPC endpoint for MCP
async fn jsonrpc_handler(Json(request): Json<JsonRpcRequest>) -> Json<JsonRpcResponse> {
    println!("Received request: {:?}", request);
    let server = McpServer::new();
    let response = server.handle_request(request);
    println!("Sending response: {:?}", response);
    Json(response)
}

// Health check endpoint
async fn health() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "server": "mcp-calculator-server",
        "version": "1.0.0"
    }))
}

// Main application
#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/sse", get(sse_handler))
        .route("/mcp", post(jsonrpc_handler))
        .route("/health", get(health))
        .layer(
            CorsLayer::new()
                .allow_origin(tower_http::cors::Any)
                .allow_methods(tower_http::cors::Any)
                .allow_headers(tower_http::cors::Any),
        );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    
    println!("Calculator MCP Server running on http://127.0.0.1:3000");
    println!("MCP JSON-RPC endpoint: http://127.0.0.1:3000/mcp");
    println!("SSE endpoint: http://127.0.0.1:3000/sse");
    println!("Health check: http://127.0.0.1:3000/health");
    println!("Available tools: add, multiply, square, sqrt");

    axum::serve(listener, app).await.unwrap();
}

// Example usage and testing
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_addition_tool() {
        let server = McpServer::new();
        
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(1)),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "add",
                "arguments": {
                    "a": 5.0,
                    "b": 3.0
                }
            })),
        };

        let response = server.handle_request(request);
        assert!(response.error.is_none());
        assert!(response.result.is_some());
    }

    #[test]
    fn test_multiplication_tool() {
        let server = McpServer::new();
        
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(1)),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "multiply",
                "arguments": {
                    "a": 4.0,
                    "b": 3.0
                }
            })),
        };

        let response = server.handle_request(request);
        assert!(response.error.is_none());
        assert!(response.result.is_some());
    }

    #[test]
    fn test_tools_list() {
        let server = McpServer::new();
        
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(1)),
            method: "tools/list".to_string(),
            params: None,
        };

        let response = server.handle_request(request);
        assert!(response.error.is_none());
        assert!(response.result.is_some());
    }
}

/* 
Cargo.toml dependencies needed:

[dependencies]
axum = "0.7"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
futures = "0.3"
tokio-stream = "0.1"
tower-http = { version = "0.5", features = ["cors"] }

To run:
1. Update Cargo.toml with the dependencies above
2. Run: cargo run
3. Test with: curl http://127.0.0.1:3000/health

For MCP client integration, connect to: http://127.0.0.1:3000/sse
*/
