# primes

A small Rust command-line app that prints the prime factorization of a positive
integer.

## Clean Installation

Prerequisites: Rust and Cargo with support for the Rust 2024 edition declared in
`Cargo.toml`, plus the linker/build tools required by your platform. The development
checks below also use the Clippy and rustfmt components.

From the parent directory of a fresh checkout, enter the repository and build:

```bash
cd primes
cargo build --locked --release
./target/release/primes 84
```

Keep the checked-in `Cargo.lock`. `--locked` requires Cargo to use it without
updating it. Cargo automatically restores any missing downloaded crates during
the build; this project currently declares no external dependencies. No separate
dependency installation or environment-variable/secret setup is required.

Output:

```text
84 = 2 * 2 * 3 * 7
```

## Normal Development

Run these commands from the repository root:

```bash
cargo build --locked
cargo run --locked -- 84
```

Cargo reuses existing build artifacts and rebuilds what changed. `cargo run`
builds before running; ordinary source changes do not require removing
dependencies or cleaning the build output. There is no configured watch command;
rerun these commands after editing.

Run the unit and CLI integration tests, lint checks, and formatting check:

```bash
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
```

For an optimized binary (including meaningful algorithm timings):

```bash
cargo build --locked --release
./target/release/primes 84
```

## Clean Rebuild

The repository's regenerable Cargo output is `target/`, which is ignored by Git.
From the repository root, use the following sequence when a full rebuild is
needed, including after local build output has been removed:

```bash
cargo clean --target-dir target
cargo build --locked --release --target-dir target
./target/release/primes 84
```

The explicit `--target-dir target` keeps cleanup and rebuilding in this checkout's
`target/` directory even if your Cargo settings select another output location.
Before cleaning, ensure `target/` is project-owned build output and is not a
symlink to a shared directory or a location containing user data. The other
commands in this README assume Cargo's default local `target/` location.

Keep source, configuration, any `.env` files, user data, and `Cargo.lock`. Do not
clear global Cargo caches, rustup toolchains, installed targets, or installed
executables. Cargo restores missing crates as part of the locked build. These
workflows build and run locally; they do not install a second global binary or
publish a release.

## Examples

```bash
./target/release/primes 97
# 97 = 97

./target/release/primes 1024
# 1024 = 2 * 2 * 2 * 2 * 2 * 2 * 2 * 2 * 2 * 2

./target/release/primes 1
# 1 has no prime factors
```

Use `--timeout` to cap a normal factorization attempt:

```bash
./target/release/primes --timeout 60 340282366920938461286658806734041124249
```

Print the installed version:

```bash
./target/release/primes --version
```

Show usage:

```bash
./target/release/primes --help
```

## Algorithm

The app chooses the factorization method from the input length:

- numbers with up to 15 decimal digits use `6k +/- 1` trial division
- longer numbers use Pollard's Rho (Brent's variant with batched gcds) with
  Miller-Rabin probable-prime checks

The output format is the same for both paths.

Pass `--algorithm` to skip the automatic selection and force one
implementation:

```bash
./target/release/primes --algorithm pollard-rho 84
```

Valid algorithm names are `original`, `six-k`, and `pollard-rho`. The
`--algorithm` form runs in-process without a timeout; debug mode uses it
internally to benchmark each algorithm in a separate process.

## Debug Timings

Use `--debug` to compare the three implemented algorithms:

```bash
./target/release/primes --debug 1000000016000000063
```

Set a longer per-algorithm timeout when needed:

```bash
./target/release/primes --debug --timeout 60 340282366920938461286658806734041124249
```

Debug mode reports:

- original trial division
- `6k +/- 1` trial division
- Pollard's Rho

Each algorithm runs in a separate process. Debug mode uses a 5-second timeout by
default, so very slow trial-division cases do not block the whole comparison
forever. For meaningful timings, run the release binary rather than `cargo run`.

If the automatically selected algorithm times out, debug mode reports that
instead of blocking while trying to print the final factorization.
