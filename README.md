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

## Test

```bash
cargo test
cargo clippy -- -D warnings
cargo fmt --check
```
