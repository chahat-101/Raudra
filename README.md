# 🔥 Raudra

<div align="center">

```
██████╗  █████╗ ██╗   ██╗██████╗ ██████╗  █████╗ 
██╔══██╗██╔══██╗██║   ██║██╔══██╗██╔══██╗██╔══██╗
██████╔╝███████║██║   ██║██║  ██║██████╔╝███████║
██╔══██╗██╔══██║██║   ██║██║  ██║██╔══██╗██╔══██║
██║  ██║██║  ██║╚██████╔╝██████╔╝██║  ██║██║  ██║
╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝
```

**A blazing-fast HTTP load testing tool built with Rust 🦀**

[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Tokio](https://img.shields.io/badge/tokio-async-green?style=for-the-badge)](https://tokio.rs/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg?style=for-the-badge)](LICENSE)

[Features](#-features) • [Installation](#-installation) • [Usage](#-usage) • [Technical Details](#-technical-details) • [Contributing](#-contributing)

</div>

---

## 🚀 Features

- ⚡ **Blazing Fast** - Leverages Rust's performance and Tokio's async runtime for maximum throughput
- 🎯 **Concurrent Requests** - Send thousands of concurrent HTTP requests with ease
- 📊 **Detailed Analytics** - Comprehensive latency analysis with percentile distributions (P50, P90, P99)
- 🎭 **User Agent Rotation** - Built-in support for 10,000+ user agents (included in `user_agents.txt`)
- 🌐 **IP Spoofing** - Automatic random IP generation with proper header injection
- 🎨 **Beautiful CLI** - Colorful, centered terminal UI with real-time progress indicators
- 📈 **HDR Histograms** - High dynamic range histogram for accurate latency measurements
- 🔄 **Smart Redirects** - Configurable redirect policy (limited to 3 redirects)
- 🛡️ **Cache Busting** - Automatic cache-control headers to ensure accurate testing

## 📦 Installation

### Prerequisites

- Rust 1.70 or higher
- Cargo package manager

### Build from Source

```bash
# Clone the repository
git clone https://github.com/chahat-101/raudra.git
cd raudra

# Build in release mode for optimal performance
cargo build --release

# Run the binary
./target/release/raudra
```

### Dependencies

Raudra uses the following crates:

```toml
[dependencies]
reqwest = { version = "0.11", features = ["json"] }
tokio = { version = "1", features = ["full"] }
colored = "2.0"
hdrhistogram = "7.5"
rand = "0.8"
terminal_size = "0.3"
```

## 🎯 Usage

### Basic Usage

1. **Start Raudra**
   ```bash
   ./raudra
   ```

2. **Enter target URL**
   ```
   Enter target URL: https://example.com
   ```

3. **Specify number of requests**
   ```
   Enter number of requests: 1000
   ```

4. **Review results**
   - Success/failure summary
   - Latency analysis with percentiles
   - Min/max/average response times

### Example Session

```
🎯 Target Configuration
─────────────────────────────────────────────────────────────
Enter target URL: https://api.example.com/health

─────────────────────────────────────────────────────────────
Enter number of requests: 5000

═════════════════════════════════════════════════════════════
📋 Test Configuration
═════════════════════════════════════════════════════════════
  Target: https://api.example.com/health
  Requests: 5000
═════════════════════════════════════════════════════════════

🔥 Initiating load test...
─────────────────────────────────────────────────────────────
✓ 200 OK
✓ 200 OK
✓ 200 OK
...
```

## 📊 Output Metrics

### Request Summary
- **Total Requests** - Total number of requests sent
- **Successes** - Number of successful responses (2xx status codes)
- **Failed** - Number of failed requests (errors or non-2xx status)
- **Success Rate** - Percentage of successful requests

### Latency Analysis
- **Minimum** - Fastest response time
- **Maximum** - Slowest response time
- **Average** - Mean response time
- **P50 (Median)** - 50th percentile
- **P90** - 90th percentile (90% of requests faster than this)
- **P99** - 99th percentile (99% of requests faster than this)

## 🔧 Technical Details

### Architecture

Raudra uses an async, concurrent architecture built on Tokio:

```
┌─────────────┐
│   CLI Input │
└──────┬──────┘
       │
       ▼
┌─────────────────┐
│ Request Spawner │
└────────┬────────┘
         │
         ▼
┌────────────────────────────┐
│   Concurrent Async Tasks   │
│  ┌──────┐ ┌──────┐ ┌──────┐
│  │Task 1│ │Task 2│ │Task N│
│  └──────┘ └──────┘ └──────┘
└────────────┬───────────────┘
             │
             ▼
┌────────────────────────────┐
│   Shared State (Mutex)     │
│  • Summary Statistics      │
│  • Latency Histogram       │
└────────────┬───────────────┘
             │
             ▼
┌────────────────────────────┐
│    Results Aggregation     │
└────────────────────────────┘
```

### Features Breakdown

#### User Agent Rotation
- **10,000+ User Agents** pre-loaded in `user_agents.txt`
- Random selection for each request
- Helps simulate realistic traffic patterns

#### IP Spoofing
- Generates random public IPv4 addresses
- Excludes private ranges (10.x.x.x, 172.16-31.x.x, 192.168.x.x)
- Excludes loopback (127.x.x.x) and multicast (224-239.x.x.x)
- Injects headers: `X-Forwarded-For` and `Forwarded`

#### Latency Measurement
- Uses HDR Histogram for accurate percentile calculations
- Microsecond precision timing
- Tracks full distribution of response times

## 🎨 Customization

### Modifying Request Headers

Edit the header configuration in `main()`:

```rust
let mut headers = HeaderMap::new();
headers.insert("X-Custom-Header", HeaderValue::from_str("value")?);
```

### Adjusting Redirect Policy

Change the redirect limit:

```rust
let client = Client::builder()
    .redirect(Policy::limited(5))  // Allow 5 redirects
    .build()?;
```

### Custom User Agents

Replace or append to `user_agents.txt` with your own user agent strings (one per line).

## ⚠️ Disclaimer

**Important**: This tool is designed for legitimate load testing purposes only. 

- Only test systems you own or have explicit permission to test
- Respect rate limits and terms of service
- Do not use this tool for malicious purposes
- Be aware of legal implications in your jurisdiction
- High request volumes can impact system availability

**The authors are not responsible for misuse of this tool.**

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### Development Setup

```bash
# Clone and enter directory
git clone https://github.com/yourusername/raudra.git
cd raudra

# Run in development mode
cargo run

# Run tests
cargo test

# Format code
cargo fmt

# Lint code
cargo clippy
```

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Built with [Rust](https://www.rust-lang.org/)
- Async runtime by [Tokio](https://tokio.rs/)
- HTTP client by [Reqwest](https://github.com/seanmonstar/reqwest)
- Terminal colors by [Colored](https://github.com/mackwic/colored)
- Latency histograms by [HDR Histogram](https://github.com/HdrHistogram/HdrHistogram_rust)

---

<div align="center">

**Made with ❤️ and Rust 🦀**

[⬆ Back to Top](#-raudra)

</div>  
