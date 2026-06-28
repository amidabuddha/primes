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
