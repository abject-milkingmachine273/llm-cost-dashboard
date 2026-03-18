# Changelog

All notable changes to this project will be documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-03-17

### Added

- Real-time terminal dashboard built with ratatui and crossterm.
- Cost ledger with append-only storage, per-model aggregation, and rolling
  window monthly projection.
- Pricing table covering ten major models from Anthropic, OpenAI, and Google,
  with case-insensitive lookup and a configurable fallback rate for unknown
  models.
- JSON log ingestion from newline-delimited files or stdin. Malformed lines
  return a typed error and are skipped without corrupting the ledger.
- Budget envelope with a hard spend limit, a configurable soft alert threshold,
  remaining-balance query, and gauge percentage for the TUI.
- Demo mode that pre-loads twenty representative requests across five model
  families for development and demonstration purposes.
- Trace span store with per-request correlation IDs, tag builder, and failure
  annotation.
- `RequestLog` supporting filter-by-model iteration, JSON serialization, and
  line ingestion.
- Keyboard controls: quit (`q` / `Esc`), reset (`r`), demo (`d`), scroll
  (`j` / `k` / arrows).
- Dashboard layout: summary pane, budget gauge, per-model bar chart, scrollable
  request table, and a 60-request cost sparkline.
- CLI via clap: `--budget`, `--log-file`, `--demo` flags.
- Unified `DashboardError` enum covering ledger, budget, parse, IO,
  serialization, and terminal failure domains.
- Comprehensive test suite: unit tests in every source module plus two external
  test files (`tests/unit_tests.rs`, `tests/integration_tests.rs`) with over
  100 test functions covering cost accuracy, parser correctness, pricing table
  completeness, rolling aggregation, demo data validity, and budget enforcement.
