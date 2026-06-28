use std::{
    env,
    process::{Command as ProcessCommand, Stdio},
    thread,
    time::{Duration, Instant},
};

use crate::algorithms::{
    FactorizationAlgorithm, decimal_digit_count, factor_with_algorithm, factorization_algorithm,
    prime_factors,
};

const DEFAULT_DEBUG_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CliCommand {
    Help,
    Version,
    Factor {
        number: u128,
        timeout: Option<Duration>,
    },
    Debug {
        number: u128,
        timeout: Duration,
    },
    Algorithm {
        algorithm: FactorizationAlgorithm,
        number: u128,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum BenchmarkOutcome {
    Completed { elapsed: Duration, output: String },
    TimedOut { elapsed: Duration },
    Failed { elapsed: Duration, message: String },
}

pub fn run_from_env() -> i32 {
    match run(env::args().skip(1)) {
        Ok(()) => 0,
        Err(error) => {
            eprintln!("{}", error.message);
            error.exit_code
        }
    }
}

fn run(args: impl Iterator<Item = String>) -> Result<(), CliError> {
    match parse_command(args).map_err(CliError::usage)? {
        CliCommand::Help => {
            println!("{}", usage());
            Ok(())
        }
        CliCommand::Version => {
            println!("primes {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        CliCommand::Factor { number, timeout } => {
            print_factorization(number, timeout).map_err(CliError::runtime)
        }
        CliCommand::Debug { number, timeout } => {
            print_debug_timings(number, timeout).map_err(CliError::runtime)
        }
        CliCommand::Algorithm { algorithm, number } => {
            let factors = factor_with_algorithm(number, algorithm);
            println!("{}", format_factorization_from_factors(number, &factors));
            Ok(())
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CliError {
    message: String,
    exit_code: i32,
}

impl CliError {
    fn usage(message: String) -> Self {
        Self {
            message,
            exit_code: 2,
        }
    }

    fn runtime(message: String) -> Self {
        Self {
            message,
            exit_code: 1,
        }
    }
}

fn format_factorization(number: u128) -> String {
    format_factorization_from_factors(number, &prime_factors(number))
}

fn print_factorization(number: u128, timeout: Option<Duration>) -> Result<(), String> {
    let Some(timeout) = timeout else {
        println!("{}", format_factorization(number));
        return Ok(());
    };

    let algorithm = factorization_algorithm(number);
    match benchmark_algorithm(number, algorithm, timeout)? {
        BenchmarkOutcome::Completed { output, .. } => {
            println!("{output}");
            Ok(())
        }
        BenchmarkOutcome::TimedOut { elapsed } => Err(format!(
            "Timed out after {} using {}",
            format_duration(elapsed),
            algorithm.label()
        )),
        BenchmarkOutcome::Failed { message, .. } => Err(message),
    }
}

fn format_factorization_from_factors(number: u128, factors: &[u128]) -> String {
    if factors.is_empty() {
        return format!("{number} has no prime factors");
    }

    let expression = factors
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(" * ");

    format!("{number} = {expression}")
}

fn parse_command(args: impl Iterator<Item = String>) -> Result<CliCommand, String> {
    let args = args.collect::<Vec<_>>();

    match args.as_slice() {
        [flag] if flag == "--help" || flag == "-h" => return Ok(CliCommand::Help),
        [flag] if flag == "--version" || flag == "-V" => return Ok(CliCommand::Version),
        _ => {}
    }

    if let [flag, algorithm, number] = args.as_slice()
        && flag == "--algorithm"
    {
        let algorithm = FactorizationAlgorithm::from_cli_name(algorithm)
            .ok_or_else(|| format!("Invalid algorithm: {algorithm}"))?;

        return Ok(CliCommand::Algorithm {
            algorithm,
            number: parse_positive_integer(number)?,
        });
    }

    let mut debug = false;
    let mut timeout = None;
    let mut number = None;
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--debug" => {
                if debug {
                    return Err(String::from("Duplicate --debug flag"));
                }

                debug = true;
                index += 1;
            }
            "--timeout" => {
                let seconds = args
                    .get(index + 1)
                    .ok_or_else(|| String::from("--timeout requires seconds"))?;

                timeout = Some(parse_timeout(seconds)?);
                index += 2;
            }
            value if value.starts_with("--") => return Err(format!("Invalid option: {value}")),
            value => {
                if number.is_some() {
                    return Err(usage());
                }

                number = Some(parse_positive_integer(value)?);
                index += 1;
            }
        }
    }

    let number = number.ok_or_else(usage)?;

    if debug {
        Ok(CliCommand::Debug {
            number,
            timeout: timeout.unwrap_or(DEFAULT_DEBUG_TIMEOUT),
        })
    } else {
        Ok(CliCommand::Factor { number, timeout })
    }
}

fn usage() -> String {
    String::from(
        "Usage: primes [--debug] [--timeout <seconds>] <positive-integer>\n       primes --version\n       primes --help",
    )
}

fn parse_positive_integer(input: &str) -> Result<u128, String> {
    let number = input
        .parse::<u128>()
        .map_err(|_| format!("Invalid number: {input}"))?;

    if number == 0 {
        return Err(String::from("Number must be greater than 0"));
    }

    Ok(number)
}

fn parse_timeout(input: &str) -> Result<Duration, String> {
    let seconds = input
        .parse::<u64>()
        .map_err(|_| format!("Invalid timeout: {input}"))?;

    if seconds == 0 {
        return Err(String::from("Timeout must be greater than 0"));
    }

    Ok(Duration::from_secs(seconds))
}

fn print_debug_timings(number: u128, timeout: Duration) -> Result<(), String> {
    let automatic_algorithm = factorization_algorithm(number);
    let mut outcomes = Vec::new();

    for algorithm in [
        FactorizationAlgorithm::OriginalTrialDivision,
        FactorizationAlgorithm::SixKTrialDivision,
        FactorizationAlgorithm::PollardRho,
    ] {
        outcomes.push((algorithm, benchmark_algorithm(number, algorithm, timeout)?));
    }

    let automatic_result = outcomes.iter().find_map(|(algorithm, outcome)| {
        match (algorithm == &automatic_algorithm, outcome) {
            (true, BenchmarkOutcome::Completed { output, .. }) => Some(output.as_str()),
            _ => None,
        }
    });

    println!("Input: {number} ({} digits)", decimal_digit_count(number));
    println!("Automatic algorithm: {}", automatic_algorithm.label());
    match automatic_result {
        Some(output) => println!("Result: {output}"),
        None => println!(
            "Result: automatic algorithm did not finish within {}s",
            timeout.as_secs()
        ),
    }
    println!(
        "Algorithm timings ({}s timeout per algorithm):",
        timeout.as_secs()
    );

    for (algorithm, outcome) in outcomes {
        match outcome {
            BenchmarkOutcome::Completed { elapsed, output } => {
                println!(
                    "- {}: {} ({})",
                    algorithm.label(),
                    format_duration(elapsed),
                    output
                );
            }
            BenchmarkOutcome::TimedOut { elapsed } => {
                println!(
                    "- {}: timed out after {}",
                    algorithm.label(),
                    format_duration(elapsed)
                );
            }
            BenchmarkOutcome::Failed { elapsed, message } => {
                println!(
                    "- {}: failed after {} ({message})",
                    algorithm.label(),
                    format_duration(elapsed)
                );
            }
        }
    }

    Ok(())
}

fn benchmark_algorithm(
    number: u128,
    algorithm: FactorizationAlgorithm,
    timeout: Duration,
) -> Result<BenchmarkOutcome, String> {
    let executable = env::current_exe()
        .map_err(|error| format!("Could not locate current executable: {error}"))?;
    let started_at = Instant::now();
    let mut child = ProcessCommand::new(executable)
        .arg("--algorithm")
        .arg(algorithm.cli_name())
        .arg(number.to_string())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("Could not start benchmark process: {error}"))?;

    loop {
        if child
            .try_wait()
            .map_err(|error| format!("Could not read benchmark status: {error}"))?
            .is_some()
        {
            let elapsed = started_at.elapsed();
            let output = child
                .wait_with_output()
                .map_err(|error| format!("Could not read benchmark output: {error}"))?;

            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout).trim().to_owned();
                return Ok(BenchmarkOutcome::Completed {
                    elapsed,
                    output: stdout,
                });
            }

            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
            let message = if stderr.is_empty() {
                output.status.to_string()
            } else {
                stderr
            };

            return Ok(BenchmarkOutcome::Failed { elapsed, message });
        }

        let elapsed = started_at.elapsed();
        if elapsed >= timeout {
            child
                .kill()
                .map_err(|error| format!("Could not stop timed-out benchmark: {error}"))?;
            let _ = child.wait();
            return Ok(BenchmarkOutcome::TimedOut { elapsed });
        }

        thread::sleep(Duration::from_millis(10));
    }
}

fn format_duration(duration: Duration) -> String {
    if duration.as_secs() > 0 {
        format!("{:.3}s", duration.as_secs_f64())
    } else if duration.as_millis() > 0 {
        format!("{}ms", duration.as_millis())
    } else {
        format!("{}us", duration.as_micros())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{
        CliCommand, DEFAULT_DEBUG_TIMEOUT, FactorizationAlgorithm, format_duration,
        format_factorization, parse_command, parse_timeout,
    };

    #[test]
    fn formats_output() {
        assert_eq!(format_factorization(84), "84 = 2 * 2 * 3 * 7");
        assert_eq!(format_factorization(1), "1 has no prime factors");
    }

    #[test]
    fn rejects_invalid_arguments() {
        assert!(parse_command(Vec::<String>::new().into_iter()).is_err());
        assert!(parse_command(["10", "20"].map(String::from).into_iter()).is_err());
        assert!(parse_command(["abc"].map(String::from).into_iter()).is_err());
        assert!(parse_command(["0"].map(String::from).into_iter()).is_err());
    }

    #[test]
    fn parses_help_and_version_arguments() {
        assert_eq!(
            parse_command(["--help"].map(String::from).into_iter()),
            Ok(CliCommand::Help)
        );
        assert_eq!(
            parse_command(["-h"].map(String::from).into_iter()),
            Ok(CliCommand::Help)
        );
        assert_eq!(
            parse_command(["--version"].map(String::from).into_iter()),
            Ok(CliCommand::Version)
        );
        assert_eq!(
            parse_command(["-V"].map(String::from).into_iter()),
            Ok(CliCommand::Version)
        );
    }

    #[test]
    fn parses_debug_arguments() {
        assert_eq!(
            parse_command(["--debug", "84"].map(String::from).into_iter()),
            Ok(CliCommand::Debug {
                number: 84,
                timeout: DEFAULT_DEBUG_TIMEOUT
            })
        );
        assert_eq!(
            parse_command(["84", "--debug"].map(String::from).into_iter()),
            Ok(CliCommand::Debug {
                number: 84,
                timeout: DEFAULT_DEBUG_TIMEOUT
            })
        );
    }

    #[test]
    fn parses_timeout_arguments() {
        assert_eq!(
            parse_command(["--timeout", "30", "84"].map(String::from).into_iter()),
            Ok(CliCommand::Factor {
                number: 84,
                timeout: Some(Duration::from_secs(30))
            })
        );
        assert_eq!(
            parse_command(
                ["--debug", "--timeout", "30", "84"]
                    .map(String::from)
                    .into_iter()
            ),
            Ok(CliCommand::Debug {
                number: 84,
                timeout: Duration::from_secs(30)
            })
        );
    }

    #[test]
    fn parses_algorithm_arguments() {
        assert_eq!(
            parse_command(
                ["--algorithm", "original", "84"]
                    .map(String::from)
                    .into_iter()
            ),
            Ok(CliCommand::Algorithm {
                algorithm: FactorizationAlgorithm::OriginalTrialDivision,
                number: 84
            })
        );
    }

    #[test]
    fn formats_durations() {
        assert_eq!(format_duration(Duration::from_micros(500)), "500us");
        assert_eq!(format_duration(Duration::from_millis(2)), "2ms");
        assert_eq!(format_duration(Duration::from_millis(1_500)), "1.500s");
    }

    #[test]
    fn rejects_invalid_timeouts() {
        assert!(parse_timeout("0").is_err());
        assert!(parse_timeout("abc").is_err());
    }
}
