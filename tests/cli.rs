use std::process::Command;

#[test]
fn prints_version() {
    let output = Command::new(env!("CARGO_BIN_EXE_primes"))
        .arg("--version")
        .output()
        .expect("failed to run primes --version");

    assert!(output.status.success());
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

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Usage: primes"));
}

#[test]
fn prints_help_when_positioned_after_number() {
    let output = Command::new(env!("CARGO_BIN_EXE_primes"))
        .args(["84", "--help"])
        .output()
        .expect("failed to run primes 84 --help");

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Usage: primes"));
}

#[test]
fn factors_number() {
    let output = Command::new(env!("CARGO_BIN_EXE_primes"))
        .arg("84")
        .output()
        .expect("failed to run primes 84");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "84 = 2 * 2 * 3 * 7"
    );
}

#[test]
fn factors_fast_number_with_timeout() {
    let output = Command::new(env!("CARGO_BIN_EXE_primes"))
        .args(["--timeout", "1", "84"])
        .output()
        .expect("failed to run primes --timeout 1 84");

    assert!(output.status.success());
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

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Algorithm timings (1s timeout per algorithm):"));
    assert!(stdout.contains("original trial division"));
    assert!(stdout.contains("6k +/- 1 trial division"));
    assert!(stdout.contains("Pollard's Rho"));
}
