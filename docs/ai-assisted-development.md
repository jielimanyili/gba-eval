# AI-assisted development

## How we work with AI here

This project is built with AI assistance (Claude Code) under explicit review gates — the same discipline as the other portfolio projects.

## Review gates (every change)

1. **Architecture fit** — matches `docs/ARCHITECTURE.md` and the ADRs.
2. **Correctness** — tests pass; emulation behavior is faithful to the GBA spec.
3. **Security** — no secrets, no reading Mesen2 source, no fetching ROMs at build time.
4. **Test coverage** — CPU instructions and grader math are unit-tested.
5. **Readability** — clear module boundaries, documented decisions.

## Honesty gate

- Never claim leaderboard-competitiveness or cycle-accuracy (see `docs/ARCHITECTURE.md` §7).
- Mesen2 = binary oracle only; never read its source (ADR-002).
