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

const DEBUG_ALGORITHM_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CliCommand {
    Factor {
        number: u128,
    },
    Debug {
        number: u128,
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
        CliCommand::Factor { number } => {
            println!("{}", format_factorization(number));
            Ok(())
        }
        CliCommand::Debug { number } => print_debug_timings(number).map_err(CliError::runtime),
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
        [number] => Ok(CliCommand::Factor {
            number: parse_positive_integer(number)?,
        }),
        [flag, number] if flag == "--debug" => Ok(CliCommand::Debug {
            number: parse_positive_integer(number)?,
        }),
        [number, flag] if flag == "--debug" => Ok(CliCommand::Debug {
            number: parse_positive_integer(number)?,
        }),
        [flag, algorithm, number] if flag == "--algorithm" => {
            let algorithm = FactorizationAlgorithm::from_cli_name(algorithm)
                .ok_or_else(|| format!("Invalid algorithm: {algorithm}"))?;

            Ok(CliCommand::Algorithm {
                algorithm,
                number: parse_positive_integer(number)?,
            })
        }
        _ => Err(String::from("Usage: primes [--debug] <positive-integer>")),
    }
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

fn print_debug_timings(number: u128) -> Result<(), String> {
    let automatic_algorithm = factorization_algorithm(number);
    let mut outcomes = Vec::new();

    for algorithm in [
        FactorizationAlgorithm::OriginalTrialDivision,
        FactorizationAlgorithm::SixKTrialDivision,
        FactorizationAlgorithm::PollardRho,
    ] {
        outcomes.push((algorithm, benchmark_algorithm(number, algorithm)?));
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
            DEBUG_ALGORITHM_TIMEOUT.as_secs()
        ),
    }
    println!(
        "Algorithm timings ({}s timeout per algorithm):",
        DEBUG_ALGORITHM_TIMEOUT.as_secs()
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
        if elapsed >= DEBUG_ALGORITHM_TIMEOUT {
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
        CliCommand, FactorizationAlgorithm, format_duration, format_factorization, parse_command,
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
    fn parses_debug_arguments() {
        assert_eq!(
            parse_command(["--debug", "84"].map(String::from).into_iter()),
            Ok(CliCommand::Debug { number: 84 })
        );
        assert_eq!(
            parse_command(["84", "--debug"].map(String::from).into_iter()),
            Ok(CliCommand::Debug { number: 84 })
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
}
