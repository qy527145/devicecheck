# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a specialized HTTP MITM proxy called `devicecheck` written in Rust that intercepts ChatGPT app traffic on iOS/iPad devices to extract `preauth_cookie` values for device authentication bypass. It's designed to work with jailbroken iOS devices or devices with TrollStore installed to bypass SSL pinning.

## Build and Development Commands

### Building the Project
```bash
cargo build                    # Debug build
cargo build --release          # Optimized release build
```

### Running the Server
```bash
cargo run -- run                                      # Basic server start
cargo run -- run --debug                             # Debug mode
cargo run -- run --bind 0.0.0.0:8080                # Custom bind address
cargo run -- run --proxy http://192.168.1.1:1080    # With upstream proxy
```

### Daemon Management

#### Unix/Linux/macOS
```bash
cargo run -- start                # Start as daemon
cargo run -- stop                 # Stop daemon
cargo run -- restart              # Restart daemon
cargo run -- ps                   # Show daemon status
cargo run -- log                  # Show daemon logs
```

#### Windows
```bash
cargo run -- start                # Start as background service
cargo run -- stop                 # Stop service (uses taskkill)
cargo run -- restart              # Restart service
cargo run -- status               # Show service status
cargo run -- log                  # Show service logs
```

**Windows-specific notes:**
- Service runs in background without true Windows service installation
- PID and log files stored in temporary directory (`%TEMP%`)
- No administrator privileges required (unlike Unix versions)
- Uses `taskkill` command to stop services

### Testing and Linting
```bash
cargo test              # Run tests
cargo clippy            # Lint code
cargo fmt               # Format code
```

## Architecture Overview

### Core Components

1. **Main Entry (`src/main.rs`)**: CLI argument parsing using clap, routes commands to appropriate modules
2. **Server (`src/serve.rs`)**: Main server initialization, certificate management, and proxy startup
3. **Proxy System (`src/proxy/`)**: MITM proxy implementation with multiple modules:
   - `mod.rs`: Main proxy structure and server setup
   - `mitm.rs`: Core MITM functionality for intercepting HTTP/HTTPS traffic  
   - `handler.rs`: Device check request handler that extracts preauth cookies
   - `ca.rs`: Certificate Authority management for SSL interception
   - `client.rs`: HTTP client for upstream requests
   - `rewind.rs`: Request/response buffering utilities

4. **Certificate Generation (`src/cagen.rs`)**: CA certificate and key generation for MITM
5. **Daemon Management (`src/daemon.rs`)**: Unix daemon functionality for background operation
6. **Error Handling (`src/error.rs`)**: Centralized error types and handling

### Key Data Flow

1. Client connects through proxy (default port 1080)
2. MITM proxy intercepts HTTPS traffic using generated CA certificates
3. `DeviceCheckHandler` specifically hooks `/backend-api/preauth_devicecheck` requests from ChatGPT app
4. Extracts `device_token` and forwards request to OpenAI servers
5. Captures `_preauth_devicecheck` cookie from response and caches it by device_id
6. Provides `/auth/preauth` endpoint to retrieve cached cookies

### Certificate Management

- Certificates auto-generated in `ca/` directory if not present
- Default paths: `ca/cert.crt` (certificate) and `ca/key.pem` (private key)
- Client devices must trust the generated CA certificate
- Certificate download available at `http://proxy-ip:port/mitm/cert`

### Configuration

- Default bind address: `0.0.0.0:1080`
- Supports upstream proxy chaining via `--proxy` flag
- Debug mode available with `--debug` flag for detailed logging
- Custom certificate paths configurable via `--cert` and `--key`

### Dependencies

Key external dependencies:
- `hyper`: HTTP server and client framework
- `tokio`: Async runtime
- `rustls`/`tokio-rustls`: TLS implementation for MITM
- `rcgen`: Certificate generation
- `reqwest`: HTTP client for upstream requests
- `moka`: In-memory caching for preauth cookies
- `clap`: CLI argument parsing
- `daemonize`: Unix daemon functionality (Unix only)
- `nix`: Unix system calls (Unix only)

## Windows Compatibility

This project has been made cross-platform compatible with Windows. Key changes include:

### Platform-Specific Features
- **Unix systems**: True daemon processes with proper privilege dropping and signal handling
- **Windows systems**: Background processes using temporary directory for PID/log files

### File System Differences
- **Unix**: Uses `/var/run/` for daemon files (requires root)
- **Windows**: Uses `%TEMP%` directory for service files (no admin required)

### Process Management
- **Unix**: Uses POSIX signals (`SIGINT`) for graceful shutdown
- **Windows**: Uses `taskkill` command for process termination

### Build Requirements
- All platforms require Rust 1.75+ 
- Windows builds automatically exclude Unix-only dependencies
- No additional Windows-specific dependencies required