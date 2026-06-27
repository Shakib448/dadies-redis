# Dadies

A Redis-compatible in-memory data store server written in Rust, implementing core Redis functionality with pub/sub messaging support.

## Overview

Dadies (mini-redis) is an educational implementation of a Redis-compatible server that demonstrates modern Rust async programming patterns, protocol implementation, and concurrent connection handling. It provides both a server binary and a command-line client for interacting with the data store.

## Features

### Core Data Operations
- **GET** - Retrieve values by key
- **SET** - Store key-value pairs with optional expiration
  - `EX seconds` - Set expiration time in seconds
  - `PX milliseconds` - Set expiration time in milliseconds
- **Automatic Key Expiration** - Background task purges expired entries

### Pub/Sub Messaging
- **PUBLISH** - Send messages to channels
- **SUBSCRIBE** - Listen to one or more channels
- **UNSUBSCRIBE** - Stop listening to channels

### Utility
- **PING** - Connection health check with optional echo message

### Server Features
- Multi-threaded concurrent connection handling (up to 250 connections)
- Graceful shutdown with connection cleanup
- RESP (Redis Serialization Protocol) implementation
- Structured logging with tracing
- Exponential backoff for accept errors

## Technologies

- **Rust 2024 Edition** - Modern systems programming language
- **Tokio** - Async runtime with full-featured networking
- **RESP Protocol** - Redis Serialization Protocol implementation
- **Tracing** - Structured logging and instrumentation

### Key Dependencies
- `tokio` v1.52.3 - Async runtime
- `bytes` v1.11.1 - Efficient byte string handling
- `serde` v1.0.228 - Serialization framework
- `clap` v4.6.1 - CLI argument parsing
- `tracing` / `tracing-subscriber` - Logging infrastructure

## Installation

### Prerequisites
- Rust toolchain (2024 edition)

### Build from Source

```bash
# Clone the repository
git clone <repository-url>
cd dadies

# Build the project
cargo build --release

# Binaries will be available at:
# - target/release/dadies-server
# - target/release/dadies-cli
```

## Usage

### Starting the Server

```bash
# Default port (5000)
cargo run --bin dadies-server

# Custom port
cargo run --bin dadies-server -- --port 6379

# With debug logging
RUST_LOG=debug cargo run --bin dadies-server
```

The server listens on `127.0.0.1:5000` by default.

### Using the Client

```bash
# Connect to local server (default: 127.0.0.1:5000)
cargo run --bin dadies-cli

# Connect to custom host/port
cargo run --bin dadies-cli -- --host 127.0.0.1 --port 6379
```

### Example Commands

Once connected with the client:

```redis
# Basic key-value operations
SET mykey "Hello World"
GET mykey

# Set with expiration (5 seconds)
SET tempkey "Temporary" EX 5
GET tempkey

# Set with millisecond expiration
SET fastkey "Quick" PX 1000

# Ping the server
PING
PING "Hello"

# Pub/sub messaging
SUBSCRIBE channel1 channel2
PUBLISH channel1 "Hello subscribers!"
UNSUBSCRIBE channel1
```

## Project Structure

```
src/
├── bin/
│   ├── cli.rs              # Command-line client entry point
│   └── server.rs           # Server entry point
├── clients/                # Client library implementations
│   ├── client.rs           # Async Redis client
│   ├── blocking_client.rs  # Synchronous client wrapper
│   └── buffered_client.rs  # Buffered client variant
├── cmd/                    # Command implementations
│   ├── mod.rs              # Command enum and dispatching
│   ├── get.rs              # GET command
│   ├── set.rs              # SET command
│   ├── ping.rs             # PING command
│   ├── publish.rs          # PUBLISH command
│   ├── subscribe.rs        # SUBSCRIBE/UNSUBSCRIBE
│   └── unknown.rs          # Unknown command handler
├── connection.rs           # TCP connection & framing
├── db.rs                   # In-memory database with pub/sub
├── frame.rs                # RESP frame types
├── parser.rs               # Command frame parsing
├── server.rs               # Server listener & handler
├── shutdown.rs             # Graceful shutdown coordination
└── lib.rs                  # Library exports
```

## Architecture

### Database Layer
- Thread-safe in-memory store using `Arc<Mutex<State>>`
- HashMap for key-value storage
- BTreeSet for efficient expiration scheduling
- Broadcast channels for pub/sub messaging
- Background task for automatic key expiration

### Protocol Implementation
- Full RESP (Redis Serialization Protocol) support
- Frame types: Simple Strings, Errors, Integers, Bulk Strings, Arrays, Null
- Async TCP connection with buffered I/O
- Cursor-based frame parsing

### Server Architecture
- Async connection handling with Tokio
- Connection semaphore limits concurrent connections (250 max)
- Graceful shutdown via broadcast channels
- Structured logging with tracing instrumentation

## Development

### Running Tests

```bash
cargo test
```

### Running Benchmarks

```bash
cargo bench
```

### Logging Levels

Control logging verbosity with the `RUST_LOG` environment variable:

```bash
RUST_LOG=trace cargo run --bin dadies-server  # Most verbose
RUST_LOG=debug cargo run --bin dadies-server
RUST_LOG=info cargo run --bin dadies-server
RUST_LOG=warn cargo run --bin dadies-server
RUST_LOG=error cargo run --bin dadies-server
```

## Design Patterns

- **Async-first** architecture with Tokio
- **Instrumentation** via tracing macros
- **Error handling** using Result types
- **Graceful shutdown** via broadcast channels
- **Connection pooling** via semaphore-based limiting
- **Background tasks** for maintenance operations

## Use Cases

This project serves as:
- Educational reference for Rust async programming
- Example of protocol implementation (RESP)
- Demonstration of concurrent connection handling
- Foundation for building Redis-compatible services
- Learning resource for production-grade Rust patterns

## License

See LICENSE file for details.
