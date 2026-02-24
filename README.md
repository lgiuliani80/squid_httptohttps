# squid_httptohttps

A Dockerized [Squid](http://www.squid-cache.org/) forward proxy that automatically rewrites all outgoing **HTTP** requests to **HTTPS**, using a lightweight Rust URL-rewriter.

## How It Works

```
Client ──► Squid (port 3128) ──► URL rewriter (Rust) ──► Target site (HTTPS)
```

1. A client sends an HTTP request through the proxy (e.g. `http://example.com`).
2. Squid invokes the built-in **url_rewrite_program** — a small Rust binary that replaces `http://` with `https://`.
3. Squid forwards the request to the rewritten HTTPS URL.

This is useful when you have legacy applications or devices that only speak HTTP and you want to transparently upgrade all their traffic to HTTPS.

## Container Image

Pre-built images are published to GitHub Container Registry on every versioned release:

```
ghcr.io/lgiuliani80/squid_httptohttps:<tag>
```

For example, to pull the `v1.0.0` image:

```bash
docker pull ghcr.io/lgiuliani80/squid_httptohttps:v1.0.0
```

## Quick Start

### Prerequisites

- [Docker](https://docs.docker.com/get-docker/) and [Docker Compose](https://docs.docker.com/compose/install/)

### Build and Run

```bash
docker compose up -d --build
```

This starts two containers:

| Container      | Description                                      |
|----------------|--------------------------------------------------|
| `squid-proxy`  | Squid proxy with the URL rewriter on port **3128** |
| `demo-client`  | A curl-based container for interactive testing   |

### Test It

Use the demo container to send a request through the proxy:

```bash
docker exec demo-client curl -x http://squid-proxy:3128 http://example.com
```

The request to `http://example.com` is transparently upgraded to `https://example.com` by the proxy.

You can also point any HTTP client at `localhost:3128`:

```bash
curl -x http://localhost:3128 http://example.com
```

### Stop

```bash
docker compose down
```

## Sample Log Output

After running `docker logs squid-proxy`, you should see output similar to:

```
2026/02/23 11:31:07| helperOpenServers: Starting 1/5 'https-to-http' processes
2026/02/23 11:31:07| Logfile: opening log daemon:/var/log/squid/access.log
2026/02/23 11:31:07| Local cache digest enabled; rebuild/rewrite every 3600/3600 sec
2026/02/23 11:31:07| Max Mem  size: 262144 KB
2026/02/23 11:31:07| Max Swap size: 0 KB
2026/02/23 11:31:07| Accepting HTTP Socket connections at conn5 local=[::]:3128 remote=[::] FD 14 flags=9
    listening port: 3128
2026/02/23 11:31:08| storeLateRelease: released 0 objects
[rewriter] http://example.com/ -> https://example.com/
1771846271.801     54 172.20.0.3 TCP_MISS/200 922 GET http://example.com/ - HIER_DIRECT/104.18.26.120 text/html
[rewriter] http://example.com/ -> https://example.com/
1771846298.698      0 172.20.0.3 TCP_MEM_HIT/200 923 GET http://example.com/ - HIER_NONE/- text/html
```

Lines prefixed with `[rewriter]` are emitted by the Rust URL rewriter, showing the `http → https` conversion. The Squid access log lines confirm the request was served successfully (`200`) and show cache behavior (`TCP_MISS` on first request, `TCP_MEM_HIT` on subsequent ones).

## Configuration

### Squid (`squid.conf`)

| Directive                | Purpose                                            |
|--------------------------|----------------------------------------------------|
| `http_port 3128`         | Proxy listens on port 3128                         |
| `http_access allow all`  | Allows all clients (restrict in production!)       |
| `url_rewrite_program`    | Points to the Rust rewriter binary                 |
| `url_rewrite_children`   | Number of rewriter worker processes                |

To restrict access, edit the `acl` and `http_access` rules in `squid.conf`.

### Rewriter (`src/main.rs`)

The rewriter reads URLs from stdin (Squid's rewrite protocol) and responds with the rewritten URL. It simply replaces `http://` with `https://`. Diagnostic messages are logged to stderr.

## Project Structure

```
.
├── Cargo.toml           # Rust project manifest
├── src/
│   └── main.rs          # URL rewriter (http → https)
├── Dockerfile           # Multi-stage build: Rust compile + Squid image
├── docker-compose.yml   # Orchestrates proxy + demo client
└── squid.conf           # Squid proxy configuration
```

## Building Without Docker

If you have a [Rust toolchain](https://rustup.rs/) installed:

```bash
cargo build --release
```

The binary is produced at `target/release/https-to-http` and can be used as a standalone Squid `url_rewrite_program`.

## Additional Guides

- [Deploy an HTTPS Echo Server on Azure Container Instance](ACI_HTTPS_ECHO.md) — deploy a demo echo container on ACI with HTTP (80) and HTTPS (443) ports.

## License

This project is provided as-is for educational and operational use.
