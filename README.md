# Load Handling Checker (Rust)

A fast async Rust CLI tool for website load testing with **global RPS control**.

## Features

* Multiple target URLs (round-robin)
* Configurable HTTP method, headers, and body
* Global RPS limiter
* Duration- or request-based stopping
* Latency stats (p50/p90/p99)
* JSON summary output

## Build

```bash
cargo build --release
```

## Usage

```bash
./load-checker -u https://example.com [options]
```

### Key options

| Flag                 | Description                         |
| -------------------- | ----------------------------------- |
| `-u, --urls`         | Target URL(s) (required)            |
| `-m, --method`       | HTTP method (default: GET)          |
| `-h, --header`       | Header `Key:Value`                  |
| `-b, --body`         | Request body                        |
| `-c, --concurrency`  | Number of workers (default: 50)     |
| `-r, --rps`          | Global requests/sec (0 = unlimited) |
| `-d, --duration-sec` | Duration in seconds                 |
| `-n, --requests`     | Total number of requests            |
| `-t, --timeout-ms`   | Timeout per request (ms)            |
| `-o, --out`          | Output JSON summary file            |

Either `--duration-sec` **or** `--requests` must be set.

## Example

```bash
./load-checker -u https://example.com -c 50 -r 200 -d 15 -t 5000
```

### Example output

```
==== Load Handling Summary ====
Runtime: 15.00 s
Total requests: 3000
Successes: 2990
Failures: 10
Achieved RPS: 199.8
Latency (ms): p50=35.2 p90=80.1 p99=210.4
```

## JSON summary (if `--out` used)

```json
{
  "runtime_s": 15.0,
  "total_requests": 3000,
  "successes": 2990,
  "failures": 10,
  "achieved_rps": 199.8,
  "latency_ms": { "p50_ms": 35.2, "p90_ms": 80.1, "p99_ms": 210.4 }
}
```

## License

MIT — use responsibly (only test systems you control).

---

Would you like me to make it in Markdown format (`README.md` file) for direct copy-paste?
