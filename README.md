# llm-cost-dashboard

> Real-time terminal UI for LLM token spend — cost/request, per-model breakdown, projected monthly bills.

Built with [ratatui](https://ratatui.rs). Zero external services required.

## Install

```bash
cargo install --git https://github.com/Mattbusel/llm-cost-dashboard
```

## Usage

```bash
# Launch with demo data
llm-dash --demo

# Set a $50/month budget
llm-dash --budget 50.0 --demo

# Tail a JSON log file
llm-dash --log-file requests.log --budget 25.0

# Pipe from your app
your-app | llm-dash
```

## Dashboard Layout

```
 LLM Cost Dashboard  [q: quit | r: reset | d: demo data | j/k: scroll]
┌─────────────────┬──────────────────────────────────────────────────────┐
│ Summary         │  Cost by Model (bar chart)                           │
│ Total: $0.0142  │  ████████ claude-sonnet-4-6                          │
│ Proj:  $0.42/mo │  ████ gpt-4o-mini                                    │
├─────────────────│  ██ claude-haiku-4-5                                 │
│ Budget          ├──────────────────────────────────────────────────────┤
│ ████░░░ 14.2%   │  Recent Requests (scrollable with j/k)              │
│ $8.58 remaining │  12:34:01  claude-sonnet  847in/312out  $0.0031 45ms │
└─────────────────┴──────────────────────────────────────────────────────┘
│ Sparkline: spend over last 60 requests                                 │
└────────────────────────────────────────────────────────────────────────┘
```

## Log File Format

Newline-delimited JSON:

```json
{"model":"claude-sonnet-4-6","input_tokens":512,"output_tokens":256,"latency_ms":340}
{"model":"gpt-4o-mini","input_tokens":128,"output_tokens":64,"latency_ms":12}
```

## Keyboard Controls

| Key | Action |
|-----|--------|
| `q` / `Esc` | Quit |
| `d` | Load demo data |
| `r` | Reset all data |
| `j` / `↓` | Scroll requests down |
| `k` / `↑` | Scroll requests up |

## Pricing Table

| Model | Input ($/1M) | Output ($/1M) |
|-------|-------------|--------------|
| claude-opus-4-6 | $15.00 | $75.00 |
| claude-sonnet-4-6 | $3.00 | $15.00 |
| claude-haiku-4-5 | $0.25 | $1.25 |
| gpt-4o | $5.00 | $15.00 |
| gpt-4o-mini | $0.15 | $0.60 |
| o3-mini | $1.10 | $4.40 |
| gemini-1.5-pro | $3.50 | $10.50 |

## Related Projects by @Mattbusel

- [tokio-prompt-orchestrator](https://github.com/Mattbusel/tokio-prompt-orchestrator) — Rust LLM pipeline orchestration
- [rot-signals-api](https://github.com/Mattbusel/rot-signals-api) — Options signal REST API
- [prompt-observatory](https://github.com/Mattbusel/prompt-observatory) — LLM interpretability dashboard
- [rust-crates](https://github.com/Mattbusel/rust-crates) — Production Rust libraries for AI agents

## License

MIT
