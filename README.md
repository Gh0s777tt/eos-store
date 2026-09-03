# E-OS Store

Application store client for E-OS — browse, verify and install signed packages.

**Status: skeleton.** The window opens, the headless model has tests, the CI
gates are wired — and no product feature is implemented. Nothing in this
repository does the job its name promises yet. That sentence is the honest
state and stays here until it stops being true.

- **Platform:** E-OS (Redox microkernel) is the primary and only *supported*
  target. Linux, macOS and Windows are **secondary** — a development
  convenience so the UI can be iterated without booting a VM. A host build is
  not a supported product.
- **UI:** [Slint](https://slint.dev) 1.17, software renderer. Under E-OS the
  platform comes from [`eos-ui`](https://gitlab.com/e-os/eos-ui) (Slint over
  Orbital, plus the font bootstrap); on a host it comes from the optional
  `host-backend` feature (winit).
- **Licence:** AGPL-3.0-or-later. Slint is used under its GPL-3.0-only arm.

## Download

**[UNSIGNED.]** Nothing below is signed. PR-008 asks for *downloadable*, not
for *trusted*: the E-OS signing chain (hybrid ed25519 + ML-DSA-65, described in
the meta-repo) covers packages installed **on** E-OS, and these host archives
sit outside it. The `.sha256` shipped beside each archive proves the file did
not rot in transit — not who built it.

`packaging/release.sh <target-triple> [<output-dir>]` builds one target and then
looks at the FILE before it will call the result a package: `file -b` must match
the format the triple implies, and the binary must clear a 1 MiB floor. It exits
1 when either refuses and 2 when it could not run at all. `--selftest` feeds
those two checks fabricated inputs — a text file, 2 MiB of zeroes, a 512 KB
stub, a missing path, an unknown triple — and asserts every one is refused
(7 checks, exit 0, measured here).

Measured 2026-09-03 on macOS 26.6.1, Apple Silicon, rustc 1.98.0-nightly,
slint 1.17.1, zig 0.16.0, `cargo zigbuild`:

| target | binary | `file -b` says | archive |
|---|---|---|---|
| `x86_64-pc-windows-gnu` | 22 818 304 B | `PE32+ executable (console) x86-64, for MS Windows` | `eos-store-0.1.0-x86_64-pc-windows-gnu.zip`, 10 433 613 B |
| `x86_64-unknown-linux-gnu` | 17 001 800 B | `ELF 64-bit LSB pie executable, x86-64, version 1 (SYSV), dynamically linked, interpreter /lib64/ld-linux-x86-64.so.2, for GNU/Linux 2.0.0, stripped` | **not produced — see below** |

Both binaries were checked to be *this* product rather than a neighbour's:
`strings` finds `EOS-STORE-SELFTEST-OK` and `EOS-STORE-BACKEND-OK` in each. That
check earned its place — the sibling product `eos-sheets`, generated from the
same skeleton, builds to **byte-identical sizes** (22 818 304 and 17 001 800),
so a size alone could not have told the two apart. Their SHA-256 sums differ.

Two dependency stanzas in `Cargo.toml` exist solely to make these two targets
compile, and each was confirmed by removing it again and watching the build fail
(the errors are quoted next to them): the implicit `muda` feature for Windows,
and `fontconfig-dlopen` for Linux.

**What is NOT proven.** The Linux *archive* has never been produced on this
host. `packaging/release.sh` sends every non-Windows triple through plain
`cargo build --target`, so rustc hands GNU linker flags to Apple's `ld` and the
link dies after 20m52s of successful compilation:

```
ld: unknown options: --as-needed -Bstatic -Bdynamic --eh-frame-hdr -z --gc-sections -z -z --strip-debug
clang: error: linker command failed with exit code 1 (use -v to see invocation)
error: could not compile `eos-store` (bin "eos-store") due to 1 previous error
```

The same tree linked by zig succeeds in 16m52s, and that is where the ELF above
came from. The missing piece is one routing line in the shared packager (send
`*linux-gnu` through `cargo zigbuild`, as the Windows arm already does) or a
Linux runner — not anything in this repository. The `package-linux` CI job is
left in place and red rather than quietly deleted.

Neither target has ever been **run**: no Windows and no Linux machine was
available. `file` says what the artefacts are; nobody has yet watched the window
open on either.

## Building

### On a host (development)

```sh
cargo test                                   # model tests, no display needed
cargo run -- --selftest                      # prints EOS-STORE-SELFTEST-OK
cargo run --features host-backend            # opens the window
cargo run --features host-backend -- --check-backend   # prints EOS-STORE-BACKEND-OK
```

`host-backend` is **off by default** on purpose (fail-closed): the default
dependency graph carries no windowing backend, so a Redox cross build cannot
pick winit up by accident and a CI container needs no display libraries. Without
the feature, `--check-backend` exits **2** — "the check could not run" — and the
plain GUI run refuses with the same code. That refusal is the negative test of
the backend gate; it is expected to be red, and CI asserts that it is.

The host backend build was measured on macOS (Apple Silicon, rustc 1.98.0,
slint 1.17.1) and compiles.

**Correction, 2026-09-03.** This paragraph used to end "**[UNVERIFIED]** on
Linux and Windows — nobody has compiled it yet". Both now cross-compile from
this same macOS host, and the byte counts are in [Download](#download): the
Linux target does request `x11` and `wayland` and does build with them. What is
still unverified is *running* either one — compiling is not launching, and no
Windows or Linux machine has opened the window.

### For E-OS (the real target)

Built as a cookbook recipe in the meta-repo, not from this directory:

```sh
# in the E-OS meta-repo
bash scripts/eos-build.sh x86_64      # or aarch64
```

The recipe lives at `recipes/gui/eos-store/recipe.toml` (a copy ready to install is
in `packaging/` here) and pins this repository by revision. Bumping that pin is
step 2 of the loop in CLAUDE.md §20.5 — push to **both** remotes, bump
`repos.toml` **and** the recipe, `scripts/eos-repos.sh pins --strict`, resync the
build tree, rebuild.

## First commit checklist

The generator does not run cargo, so the tree it produces has **no `Cargo.lock`**
and CI runs `--locked`. Before the first push:

1. `cargo build` — writes `Cargo.lock`; commit it (this is a binary crate, the
   lock belongs in git).
2. Replace the placeholder icon `assets/eos-store.png` (48×48 Crimson placeholder
   drawn by the generator) with the real one.
3. Create the GitLab project `e-os/eos-store` and the GitHub mirror, push to both.
4. Add the repository to the meta-repo `repos.toml` as **type A**, and to the
   type-A list in `CLAUDE.md` §11 — `scripts/eos-check-repo-types.py`
   (ci-integrity check 7) fails on a mismatch between the two, which is exactly
   what should happen if only one is updated.
5. Copy `packaging/recipes/gui/eos-store/recipe.toml` into the meta-repo and put
   the real first-commit revision in it; `scripts/eos-repos.sh pins --strict`
   must be green.

## Exit codes

| code | meaning |
|---|---|
| 0 | success |
| 1 | a check found a defect (selftest failed, window could not be built) |
| 2 | the check could not run (no windowing backend linked in) |

The split is the same one `scripts/verify.sh` and `ci-integrity.sh` use in the
meta-repo: a broken tree and a broken toolbox need opposite reactions, so they
must not report the same thing (CLAUDE.md §13).

## Hosting

GitLab `gitlab.com/e-os/eos-store` is the source of truth; GitHub
`github.com/Gh0s777tt/eos-store` is a read-only mirror the build recipes may fetch
from (ADR-0001). This repository is **type A** — E-OS's own code — so every
rule for own code applies: tests with every change, a negative test for every
gate, docs in the same MR.

## Layout

```
src/main.rs      CLI (--selftest, --check-backend, --version, --help) + window wiring
src/model.rs     the headless half: catalogue, search, status line, selftest
ui/app.slint     the window (E-OS Crimson palette, identical to eos-notes)
build.rs         compiles the .slint file
assets/          Orbital launcher entry + 48×48 icon (placeholder — needs a designer)
packaging/       release.sh (per-OS archives) + the cookbook recipe for the meta-repo
deny.toml        cargo-deny: licences, sources, bans, advisories (feature graph)
osv-scanner.toml osv-scanner: advisory exceptions, each with an `ignoreUntil`
```

## Supply chain: two databases, and why both

`.gitlab-ci.yml` runs `cargo deny check` **and** `osv-scanner`. They disagree,
which is the point: cargo-deny walks the resolved **feature graph**, osv-scanner
reads **`Cargo.lock`**. Measured on a freshly generated tree —

```
cargo deny check advisories                    -> advisories ok            (exit 0)
osv-scanner scan source --lockfile Cargo.lock  -> 4 vulnerabilities        (exit 1)
```

The extra one is RUSTSEC-2025-0141 (bincode), which sits in the lockfile behind
an optional feature nothing enables, so cargo-deny cannot see it.

The second reason for `osv-scanner.toml` is **expiry**. cargo-deny 0.20.2 takes
only `id` and `reason` under `[[advisories.ignore]]`:

```
cargo deny --config <deny.toml with expires="…"> check advisories
  -> error[unexpected-keys]: found 1 unexpected keys, expected: ["id", "reason"]
```

so deny.toml's ignore list, which calls itself a debt register, has no mechanism
to expire. `osv-scanner.toml`'s `ignoreUntil` does: the finding comes back on the
date written next to it, and a stale exception fails the build.

`secret-scan` (gitleaks, full history) is the first job for the same reason it is
in the meta-repo (CLAUDE.md §13): a leaked credential is already too late by the
time the tests run.
