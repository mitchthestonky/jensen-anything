# Contributing

Bring the same rigor you would bring to something that matters.

Before opening a pull request, run:

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

## House rules

- Arithmetic stays in integers and stays reproducible. No clocks, randomness, network calls or LLMs.
- New phrases in `src/pivot.rs` contain no digits and at least one AI term. A test enforces both.
- Fixtures are obviously fictional companies. Never add a real company, ticker or financial figure.
- Never write a quote or statement attributed to a real person.
- No real logos or trademarks.
- Every generated output keeps the line `SATIRE, NOT FINANCIAL ADVICE`.
- If a change moves a fixture's result, update the README table. A test compares them.
