# primes

A small Rust command-line app that prints the prime factorization of a positive
integer.

## Run

From this directory:

```bash
cargo run -- 84
```

Output:

```text
84 = 2 * 2 * 3 * 7
```

## Build A Binary

```bash
cargo build --release
```

Then run the compiled binary directly:

```bash
./target/release/primes 84
```

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

## Algorithm

The app chooses the factorization method from the input length:

- numbers with up to 15 decimal digits use `6k +/- 1` trial division
- longer numbers use Pollard's Rho with Miller-Rabin probable-prime checks

The output format is the same for both paths.

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

## Test

```bash
cargo test
cargo clippy -- -D warnings
cargo fmt --check
```
