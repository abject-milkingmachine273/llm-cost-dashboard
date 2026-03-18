# llm-cost-dashboard

A real-time terminal dashboard for tracking LLM token spend. Shows cost per
request, per-model breakdowns, projected monthly bills, and a configurable
budget gauge. Zero external services required.

Built with Rust, ratatui, and crossterm.

---

## Architecture

```
JSON log files / --log-file flag
        |
        |  newline-delimited JSON records
        v
+-------------------+
|  Parser           |
|  (RequestLog /    |
|   IncomingRecord) |
|  ingest_line()    |
+-------------------+
        |
        |  LogEntry -> CostRecord
        v
+-------------------+       +-------------------+
|  CostEngine       |       |  BudgetEnvelope   |
|  (CostLedger)     | ----> |  spend()          |
|  - total_usd()    |       |  remaining()      |
|  - by_model()     |       |  alert_triggered()|
|  - projected_     |       +-------------------+
|    monthly_usd()  |
|  - sparkline_data |
+-------------------+
        |
        v
+-------------------+
|  Ratatui TUI      |
|  - summary pane   |
|  - budget gauge   |
|  - model bar chart|
|  - request table  |
|  - cost sparkline |
+-------------------+
```

---

## Quickstart

**From git:**

```bash
cargo install --git https://github.com/Mattbusel/llm-cost-dashboard
```

**From source:**

```bash
git clone https://github.com/Mattbusel/llm-cost-dashboard
cd llm-cost-dashboard
cargo build --release
./target/release/llm-dash --demo
```

**Common invocations:**

```bash
# Launch with demo data
llm-dash --demo

# Set a $50/month budget
llm-dash --budget 50.0 --demo

# Tail a JSON log file
llm-dash --log-file requests.log --budget 25.0
```

---

## Input format specification

The dashboard ingests newline-delimited JSON (NDJSON). Each line is one
completed LLM request. Required fields:

| Field           | Type   | Description                          |
|-----------------|--------|--------------------------------------|
| `model`         | string | Model identifier (see pricing table) |
| `input_tokens`  | u64    | Number of prompt tokens consumed     |
| `output_tokens` | u64    | Number of completion tokens produced |
| `latency_ms`    | u64    | End-to-end request latency in ms     |

Optional fields:

| Field      | Type           | Default     | Description                         |
|------------|----------------|-------------|-------------------------------------|
| `provider` | string or null | `"unknown"` | Provider name                       |
| `error`    | string or null | absent      | If present, marks the request failed |

Example records:

```json
{"model":"claude-sonnet-4-6","input_tokens":512,"output_tokens":256,"latency_ms":340}
{"model":"gpt-4o-mini","input_tokens":128,"output_tokens":64,"latency_ms":12}
{"model":"gpt-4o","input_tokens":1024,"output_tokens":512,"latency_ms":180,"provider":"openai"}
```

Malformed lines are skipped with an error surfaced in the UI; they do not crash
the process or corrupt the ledger.

---

## Configuration options

All configuration is supplied via CLI flags. There is no config file.

| Flag         | Type  | Default | Description                        |
|--------------|-------|---------|------------------------------------|
| `--budget`   | f64   | 10.0    | Monthly hard budget limit in USD   |
| `--log-file` | path  | none    | NDJSON file to read on startup     |
| `--demo`     | flag  | off     | Pre-load 20 synthetic demo records |

---

## Supported models and pricing

Prices are in USD per 1,000,000 tokens. Unknown models fall back to the
fallback rate of $5.00 input / $15.00 output per 1M tokens.

| Model              | Input ($/1M) | Output ($/1M) |
|--------------------|-------------|----------------|
| claude-opus-4-6    | 15.00       | 75.00          |
| claude-sonnet-4-6  | 3.00        | 15.00          |
| claude-haiku-4-5   | 0.25        | 1.25           |
| gpt-4o             | 5.00        | 15.00          |
| gpt-4o-mini        | 0.15        | 0.60           |
| gpt-4-turbo        | 10.00       | 30.00          |
| o1-preview         | 15.00       | 60.00          |
| o3-mini            | 1.10        | 4.40           |
| gemini-1.5-pro     | 3.50        | 10.50          |
| gemini-1.5-flash   | 0.075       | 0.30           |

---

## Keyboard controls

| Key              | Action                   |
|------------------|--------------------------|
| `q` / `Esc`      | Quit                     |
| `r`              | Reset all data           |
| `d`              | Load demo data           |
| `j` / Down arrow | Scroll request list down |
| `k` / Up arrow   | Scroll request list up   |

---

## Dashboard layout

```
 LLM Cost Dashboard  [q: quit | r: reset | d: demo data | j/k: scroll]
+------------------+----------------------------------------------------+
| Summary          |  Cost by Model (bar chart)                         |
| Total: $0.0142   |  claude-sonnet-4-6  |||||||||||                    |
| Proj:  $0.42/mo  |  gpt-4o-mini        |||||                          |
+------------------+  claude-haiku-4-5   |||                            |
| Budget           +----------------------------------------------------+
| |||||||| 14.2%   |  Recent Requests (j/k to scroll)                   |
| $8.58 remaining  |  12:34:01  claude-sonnet  847in/312out  $0.0031    |
+------------------+----------------------------------------------------+
 Cost sparkline: last 60 requests
```

---

## Error handling

All public API functions return `Result<_, DashboardError>`. The dashboard
never panics on runtime input. Malformed JSON log lines produce
`DashboardError::LogParseError` and are skipped. Budget overruns produce
`DashboardError::BudgetExceeded`, which is silently absorbed in the main loop
so ingestion continues. Unknown model names fall back to a mid-range pricing
estimate rather than returning an error.

---

## License

MIT
