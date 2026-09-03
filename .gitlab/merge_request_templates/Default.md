<!--
eos-store merge request. The checklist is the Definition of Done from the E-OS
meta-repo CLAUDE.md §6 — https://gitlab.com/e-os/e-os/-/blob/main/CLAUDE.md#6-definicja-ukończenia
Tick what applies; strike through (~~…~~) what genuinely does not and say why.
-->

## What & why

<!-- One paragraph: what this changes and the reason. Link the ROADMAP item / issue. -->

## Verification (paste real command output, not a summary — CLAUDE.md §5.3)

- [ ] `cargo fmt -- --check`
- [ ] `cargo clippy --locked --all-targets -- -D warnings`
- [ ] `cargo test --locked`
- [ ] `cargo run --locked -- --selftest` prints `EOS-STORE-SELFTEST-OK`
- [ ] `cargo run --locked --features host-backend -- --check-backend` prints `EOS-STORE-BACKEND-OK`
- [ ] `cargo deny check`

```text
<!-- paste the output here -->
```

## Definition of Done (CLAUDE.md §6)

- [ ] **Every change has a test** — new or updated; if a test was impossible, the reason is written here and was agreed before the change (§5.1)
- [ ] **Negative test** — each added check has been seen red, and it was *that* check that went red (§5.4, §5.9 level 2)
- [ ] **Counter-measurement** — the same measurement run on the code before the change (§5.9 level 4)
- [ ] **Artefact inspected**, not just the exit code (§5.3)
- [ ] **Doc-comments** — `//!`/`///` on every new public item
- [ ] **Docs updated in this same MR** — README / recipe / meta-repo CHANGELOG (§5.8)
- [ ] **Pins & mirrors** — pushed to **GitLab and GitHub**, then `repos.toml` + `recipes/gui/eos-store/recipe.toml` bumped, `scripts/eos-repos.sh pins --strict` green
- [ ] **Boot / runtime evidence** if the image changed (`scripts/ci-boot-smoke.sh`)
- [ ] **Conventional Commit(s)**, one logical change

<!-- If a code change ships no docs on purpose, write `docs: n/a` in this description. -->
