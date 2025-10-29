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

**A blazing-fast HTTP load testing tool written in Rust**

[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Tokio](https://img.shields.io/badge/tokio-async-green?style=for-the-badge)](https://tokio.rs/)

</div>

---

## 📋 Overview

Raudra is a powerful, concurrent HTTP load testing tool that helps you stress-test your web applications and APIs. Built with Rust's async/await capabilities using Tokio, it provides detailed performance metrics and latency analysis.

### ✨ Key Features

- 🚀 **Concurrent Requests**: Spawn thousands of parallel HTTP requests
- 📊 **Detailed Metrics**: Track success rates, failures, and response times
- 📈 **Latency Analysis**: Get percentile-based latency statistics (P50, P90, P99)
- 🎭 **User Agent Rotation**: Randomize user agents for realistic testing
- 🌐 **IP Spoofing**: Generate random public IPs for X-Forwarded-For headers
- 🎨 **Beautiful CLI**: Interactive command-line interface with colored output
- ⚡ **High Performance**: Leverages Rust's zero-cost abstractions and async I/O

---

## 🛠️ Installation

### Prerequisites

- Rust 1.70 or higher
- Cargo (comes with Rust)

### Build from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/raudra.git
cd raudra

# Build the project
cargo build --release

# Run the binary
./target/release/raudra
```

---

## 📦 Dependencies

Add these to your `Cargo.toml`:

```toml
[dependencies]
tokio = { version = "1.0", features = ["full"] }
reqwest = { version = "0.11", features = ["json"] }
hdrhistogram = "7.5"
rand = "0.8"
colored = "2.0"
```

---

## 🚀 Usage

### Interactive Mode

Simply run the binary and follow the prompts:

```bash
./raudra
```

You'll be asked to:
1. Confirm if you want to begin testing
2. Enter the target URL
3. Specify the number of concurrent requests

### Example Session

```
Do you want to begin!?
For yes enter Y/y else N/n
> y

Enter the target
> https://example.com

Enter the number of requests:
> 1000
```

### Output

Raudra provides comprehensive statistics after each test:

```
Total: 1000 | Successes: 995 | Failed: 5

Latency Summary (ms):
  Min:  45.23
  Max:  1234.56
  Mean: 123.45
  P50:  98.76
  P90:  234.12
  P99:  567.89
```

---

## 📊 Metrics Explained

### Success/Failure Rates
- **Total**: Total number of requests sent
- **Successes**: Requests that received a valid HTTP response
- **Failed**: Requests that encountered errors or timeouts

### Latency Percentiles
- **Min/Max**: Fastest and slowest response times
- **Mean**: Average response time across all requests
- **P50 (Median)**: 50% of requests were faster than this
- **P90**: 90% of requests were faster than this
- **P99**: 99% of requests were faster than this

---

## ⚙️ Configuration

### User Agents

Create a `user_agents.txt` file in the same directory as the binary:

```
Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36
Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36
Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36
```

Each line should contain one user agent string.

### Request Headers

Raudra automatically sets the following headers:
- `X-Forwarded-For`: Random public IP
- `Forwarded`: IP forwarding information
- `User-Agent`: Randomly selected from user_agents.txt
- `Cache-Control`: Prevents caching
- `Pragma`: Legacy cache control
- `Expires`: Cache expiration

---

## 🔧 Technical Details

### Architecture

- **Async Runtime**: Tokio for efficient async I/O
- **HTTP Client**: reqwest with connection pooling
- **Concurrency**: Spawns individual tokio tasks for each request
- **Synchronization**: Arc<Mutex<T>> for thread-safe metric collection
- **Histogram**: HDR Histogram for accurate latency percentiles

### IP Generation

Raudra generates random public IPv4 addresses while excluding:
- Private ranges (10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16)
- Loopback (127.0.0.0/8)
- Multicast (224.0.0.0 - 239.255.255.255)

---

## ⚠️ Disclaimer

**IMPORTANT**: This tool is designed for testing your own applications and services. 

- ✅ Use on systems you own or have permission to test
- ❌ Do not use for malicious purposes or unauthorized testing
- ⚖️ Comply with all applicable laws and terms of service
- 🛡️ The authors are not responsible for misuse of this tool

---

## 🤝 Contributing

Contributions are welcome! Here's how you can help:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

---

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## 🙏 Acknowledgments

- Built with [Rust](https://www.rust-lang.org/)
- Async runtime by [Tokio](https://tokio.rs/)
- HTTP client by [reqwest](https://github.com/seanmonstar/reqwest)
- Colored terminal output by [colored](https://github.com/mackwic/colored)

---

<div align="center">

**Made with ❤️ and Rust**

If you find this tool useful, consider giving it a ⭐!

</div>