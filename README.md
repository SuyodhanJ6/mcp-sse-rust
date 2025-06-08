# 🧮 MCP SSE Rust Calculator Server

[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)](https://github.com)

A high-performance **Model Context Protocol (MCP)** server implemented in Rust that provides calculator functionality through Server-Sent Events (SSE) and JSON-RPC APIs.

## ✨ Features

- 🚀 **High Performance**: Built with Rust and Axum for maximum performance
- 🔄 **Real-time Communication**: Server-Sent Events (SSE) support
- 🧮 **Calculator Tools**: Addition, multiplication, square, and square root operations
- 📡 **JSON-RPC Protocol**: Compliant with MCP 2024-11-05 specification
- 🌐 **CORS Enabled**: Cross-origin resource sharing support
- 🧪 **Well Tested**: Comprehensive unit tests included
- ⚡ **Async/Await**: Fully asynchronous implementation with Tokio

## 🛠️ Available Tools

| Tool | Description | Parameters |
|------|-------------|------------|
| `add` | Add two numbers together | `a: number`, `b: number` |
| `multiply` | Multiply two numbers together | `a: number`, `b: number` |
| `square` | Calculate the square of a number | `number: number` |
| `sqrt` | Calculate the square root of a number | `number: number` (non-negative) |

## 🚀 Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) (version 1.70 or later)
- [Cargo](https://doc.rust-lang.org/cargo/) (comes with Rust)

### Installation

1. **Clone the repository**
   ```bash
   git clone https://github.com/SuyodhanJ6/mcp-sse-rust.git
   cd mcp-sse-rust
   ```

2. **Install dependencies**
   ```bash
   cargo build
   ```

3. **Run the server**
   ```bash
   cargo run
   ```

The server will start on `http://localhost:3000` 🎉

### MCP Integration with Cursor

To use this server with Cursor IDE, add the following to your `~/.cursor/mcp.json`:

```json
{
  "mcpServers": {
    "mcp-calculator-sse": {
      "url": "http://127.0.0.1:3000/mcp",
      "transport": "sse"
    }
  }
}
```

After adding the configuration, restart Cursor to enable the calculator tools.

### Development

For development with auto-reload:
```bash
cargo install cargo-watch
cargo watch -x run
```

## 📖 API Documentation

### Endpoints

#### Health Check
```
GET /health
```
Returns server health status.

#### MCP Endpoint (for Cursor integration)
```
GET /mcp
```
Model Context Protocol endpoint with Server-Sent Events for Cursor IDE integration.

#### JSON-RPC Endpoint
```
POST /jsonrpc
Content-Type: application/json
```

#### Server-Sent Events
```
GET /sse
```
Establishes SSE connection for real-time communication.

### JSON-RPC Methods

#### Initialize
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": {}
}
```

#### List Tools
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/list"
}
```

#### Call Tool
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "add",
    "arguments": {
      "a": 5,
      "b": 3
    }
  }
}
```

## 🧪 Examples

### Using curl

**Add two numbers:**
```bash
curl -X POST http://localhost:3000/jsonrpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/call",
    "params": {
      "name": "add",
      "arguments": {"a": 10, "b": 5}
    }
  }'
```

**Calculate square root:**
```bash
curl -X POST http://localhost:3000/jsonrpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 2,
    "method": "tools/call",
    "params": {
      "name": "sqrt",
      "arguments": {"number": 16}
    }
  }'
```

### Server-Sent Events

Connect to the SSE endpoint:
```bash
curl -N http://localhost:3000/sse
```

## 🧪 Testing

Run the test suite:
```bash
cargo test
```

Run tests with output:
```bash
cargo test -- --nocapture
```

## 📁 Project Structure

```
mcp-sse-rust/
├── src/
│   └── main.rs          # Main server implementation
├── Cargo.toml           # Dependencies and project metadata
├── Cargo.lock           # Dependency lock file
├── LICENSE              # MIT License
├── README.md            # This file
└── .gitignore          # Git ignore patterns
```

## 🔧 Configuration

The server runs on port `3000` by default. You can modify this in the `main()` function in `src/main.rs`.

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [Axum](https://github.com/tokio-rs/axum) - Web application framework
- [Tokio](https://tokio.rs/) - Asynchronous runtime
- [Serde](https://serde.rs/) - Serialization framework
- [Model Context Protocol](https://github.com/modelcontextprotocol) - Protocol specification
- [MCP SSE Rust](https://github.com/SuyodhanJ6/mcp-sse-rust) - Related MCP server implementation


Made with ❤️ in Rust by [Prashant Malge](https://github.com/SuyodhanJ6)
