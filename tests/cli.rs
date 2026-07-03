use std::process::{Command, Output};

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "expected success, got {}; stderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stderr.is_empty(),
        "expected empty stderr, got: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn prints_version() {
    let output = Command::new(env!("CARGO_BIN_EXE_primes"))
        .arg("--version")
        .output()
        .expect("failed to run primes --version");

    assert_success(&output);
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        format!("primes {}", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn prints_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_primes"))
        .arg("--help")
        .output()
        .expect("failed to run primes --help");

    assert_success(&output);
    assert!(String::from_utf8_lossy(&output.stdout).contains("Usage: primes"));
}

#[test]
fn prints_help_when_positioned_after_number() {
    let output = Command::new(env!("CARGO_BIN_EXE_primes"))
        .args(["84", "--help"])
        .output()
        .expect("failed to run primes 84 --help");

    assert_success(&output);
    assert!(String::from_utf8_lossy(&output.stdout).contains("Usage: primes"));
}

#[test]
fn factors_number() {
    let output = Command::new(env!("CARGO_BIN_EXE_primes"))
        .arg("84")
        .output()
        .expect("failed to run primes 84");

    assert_success(&output);
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "84 = 2 * 2 * 3 * 7"
    );
}

#[test]
fn factors_number_with_explicit_algorithm() {
    for algorithm in ["original", "six-k", "pollard-rho"] {
        let output = Command::new(env!("CARGO_BIN_EXE_primes"))
            .args(["--algorithm", algorithm, "84"])
            .output()
            .unwrap_or_else(|_| panic!("failed to run primes --algorithm {algorithm} 84"));

        assert_success(&output);
        assert_eq!(
            String::from_utf8_lossy(&output.stdout).trim(),
            "84 = 2 * 2 * 3 * 7"
        );
    }
}

#[test]
fn factors_fast_number_with_timeout() {
    let output = Command::new(env!("CARGO_BIN_EXE_primes"))
        .args(["--timeout", "1", "84"])
        .output()
        .expect("failed to run primes --timeout 1 84");

    assert_success(&output);
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "84 = 2 * 2 * 3 * 7"
    );
}

#[test]
fn debug_smoke_test() {
    let output = Command::new(env!("CARGO_BIN_EXE_primes"))
        .args(["--debug", "--timeout", "1", "84"])
        .output()
        .expect("failed to run primes --debug --timeout 1 84");

    assert_success(&output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Algorithm timings (1s timeout per algorithm):"));
    assert!(stdout.contains("original trial division"));
    assert!(stdout.contains("6k +/- 1 trial division"));
    assert!(stdout.contains("Pollard's Rho"));
}

#[test]
fn rejects_invalid_algorithm() {
    let output = Command::new(env!("CARGO_BIN_EXE_primes"))
        .args(["--algorithm", "unknown", "84"])
        .output()
        .expect("failed to run primes --algorithm unknown 84");

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("Invalid algorithm: unknown"));
}

#[test]
fn rejects_duplicate_timeout_flag() {
    let output = Command::new(env!("CARGO_BIN_EXE_primes"))
        .args(["--timeout", "1", "--timeout", "2", "84"])
        .output()
        .expect("failed to run primes with duplicate --timeout flags");

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("Duplicate --timeout flag"));
}
