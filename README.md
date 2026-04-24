# dnsperf

Benchmark DNS resolvers from your terminal. Tests your ISP's DNS and 9 public resolvers in parallel, then ranks them by failure rate and tail latency.

## Install

```bash
cargo install --path .
```

Or build manually:

```bash
cargo build --release
# binary is at ./target/release/dnsperf
```

## Sample Output

```
Resolver                    Avg (ms)  Med (ms)  P95 (ms)  Min (ms) Max (ms)   Successful
---------------------------- --------- --------- --------- -------- -------- ------------
Cloudflare (1.1.1.1)            4.20      3.90     11.20    1.000   12.000        36/36
Google (8.8.8.8)                5.81      5.10     16.40    2.000   18.000        36/36
ISP (192.168.1.1)               8.44      7.80     20.30    3.000   22.000        36/36
Quad9 (9.9.9.9)                12.33     11.90     29.60    4.000   31.000        36/36
...
```

## Built-in Resolvers

| Provider | IP |
|---|---|
| Cloudflare | 1.1.1.1 |
| Google | 8.8.8.8 |
| Quad9 | 9.9.9.9 |
| OpenDNS | 208.67.222.222 |
| AdGuard | 94.140.14.14 |
| Comodo | 8.26.56.26 |
| Level3 | 4.2.2.1 |
| NextDNS | 45.90.28.0 |
| CleanBrowsing | 185.228.168.9 |

Your ISP's resolver IP is auto-detected and included by default.

## Usage

```
dnsperf [OPTIONS] [NAME:IP]...
```

### Options

```
-r, --runs <NUM>          Queries per domain per resolver (default: 3)
    --warmup <NUM>        Warmup queries per domain per resolver, excluded from results (default: 1)
-d, --domains <FILE>      File with one domain per line (overrides built-in list)
-t, --timeout <SECS>      Query timeout in seconds (default: 2)
-q, --quiet               Suppress progress output; only show results
    --no-color            Disable colored output
    --no-isp              Skip testing the detected ISP DNS server
    --csv                 Output results in CSV format
-h, --help                Show help
```

### Examples

```bash
# More thorough test with 5 runs per domain
dnsperf -r 5

# Skip warmup queries
dnsperf --warmup 0

# Add custom resolvers
dnsperf "PiHole:10.0.0.53" "Work:172.16.0.1"

# Use a custom domain list and export to CSV
dnsperf -d my_domains.txt --csv > results.csv

# Quiet mode — no progress, just results
dnsperf -q
```

## License

[GPL-3.0-or-later](LICENSE)
