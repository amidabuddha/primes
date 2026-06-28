use std::{
    env, process,
    process::{Command as ProcessCommand, Stdio},
    thread,
    time::{Duration, Instant},
};

const TRIAL_DIVISION_MAX_DIGITS: usize = 15;
const DEBUG_ALGORITHM_TIMEOUT: Duration = Duration::from_secs(5);
const MILLER_RABIN_BASES: [u128; 16] = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FactorizationAlgorithm {
    OriginalTrialDivision,
    SixKTrialDivision,
    PollardRho,
}

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

impl FactorizationAlgorithm {
    fn label(self) -> &'static str {
        match self {
            FactorizationAlgorithm::OriginalTrialDivision => "original trial division",
            FactorizationAlgorithm::SixKTrialDivision => "6k +/- 1 trial division",
            FactorizationAlgorithm::PollardRho => "Pollard's Rho",
        }
    }

    fn cli_name(self) -> &'static str {
        match self {
            FactorizationAlgorithm::OriginalTrialDivision => "original",
            FactorizationAlgorithm::SixKTrialDivision => "six-k",
            FactorizationAlgorithm::PollardRho => "pollard-rho",
        }
    }

    fn from_cli_name(name: &str) -> Option<Self> {
        match name {
            "original" => Some(FactorizationAlgorithm::OriginalTrialDivision),
            "six-k" => Some(FactorizationAlgorithm::SixKTrialDivision),
            "pollard-rho" => Some(FactorizationAlgorithm::PollardRho),
            _ => None,
        }
    }
}

fn prime_factors(number: u128) -> Vec<u128> {
    factor_with_algorithm(number, factorization_algorithm(number))
}

fn factor_with_algorithm(number: u128, algorithm: FactorizationAlgorithm) -> Vec<u128> {
    let mut factors = match algorithm {
        FactorizationAlgorithm::OriginalTrialDivision => prime_factors_trial_original(number),
        FactorizationAlgorithm::SixKTrialDivision => prime_factors_trial_6k(number),
        FactorizationAlgorithm::PollardRho => {
            let mut factors = Vec::new();
            factor_pollard_rho(number, &mut factors);
            factors
        }
    };

    factors.sort_unstable();
    factors
}

fn factorization_algorithm(number: u128) -> FactorizationAlgorithm {
    if decimal_digit_count(number) <= TRIAL_DIVISION_MAX_DIGITS {
        FactorizationAlgorithm::SixKTrialDivision
    } else {
        FactorizationAlgorithm::PollardRho
    }
}

fn decimal_digit_count(number: u128) -> usize {
    number.to_string().len()
}

fn prime_factors_trial_original(mut number: u128) -> Vec<u128> {
    let mut factors = Vec::new();
    if number < 2 {
        return factors;
    }

    let mut divisor = 2;
    while divisor <= number / divisor {
        while number % divisor == 0 {
            factors.push(divisor);
            number /= divisor;
        }

        divisor += if divisor == 2 { 1 } else { 2 };
    }

    if number > 1 {
        factors.push(number);
    }

    factors
}

fn prime_factors_trial_6k(mut number: u128) -> Vec<u128> {
    let mut factors = Vec::new();
    if number < 2 {
        return factors;
    }

    while number % 2 == 0 {
        factors.push(2);
        number /= 2;
    }

    while number % 3 == 0 {
        factors.push(3);
        number /= 3;
    }

    let mut divisor = 5;
    let mut step = 2;
    while divisor <= number / divisor {
        while number % divisor == 0 {
            factors.push(divisor);
            number /= divisor;
        }

        divisor += step;
        step = 6 - step;
    }

    if number > 1 {
        factors.push(number);
    }

    factors
}

fn factor_pollard_rho(number: u128, factors: &mut Vec<u128>) {
    if number == 1 {
        return;
    }

    if number % 2 == 0 {
        factors.push(2);
        factor_pollard_rho(number / 2, factors);
        return;
    }

    if number % 3 == 0 {
        factors.push(3);
        factor_pollard_rho(number / 3, factors);
        return;
    }

    if is_probably_prime_miller_rabin(number) {
        factors.push(number);
        return;
    }

    let divisor = pollard_rho_divisor(number);
    factor_pollard_rho(divisor, factors);
    factor_pollard_rho(number / divisor, factors);
}

fn pollard_rho_divisor(number: u128) -> u128 {
    let mut constant = 1;

    loop {
        let constant_mod = constant % number;
        let mut x = 2 + constant_mod % (number - 3);
        let mut y = x;
        let mut divisor = 1;
        let mut iterations = 0;

        while divisor == 1 && iterations < 100_000 {
            x = pollard_rho_step(x, constant_mod, number);
            y = pollard_rho_step(
                pollard_rho_step(y, constant_mod, number),
                constant_mod,
                number,
            );
            divisor = gcd(x.abs_diff(y), number);
            iterations += 1;
        }

        if divisor > 1 && divisor < number {
            return divisor;
        }

        constant += 1;
    }
}

fn pollard_rho_step(value: u128, constant: u128, modulus: u128) -> u128 {
    add_mod(mul_mod(value, value, modulus), constant, modulus)
}

fn is_probably_prime_miller_rabin(number: u128) -> bool {
    if number < 2 {
        return false;
    }

    for prime in MILLER_RABIN_BASES {
        if number == prime {
            return true;
        }

        if number % prime == 0 {
            return false;
        }
    }

    let mut odd_part = number - 1;
    let mut powers_of_two = 0;

    while odd_part % 2 == 0 {
        odd_part /= 2;
        powers_of_two += 1;
    }

    'bases: for base in MILLER_RABIN_BASES {
        let base = base % number;
        if base < 2 {
            continue;
        }

        let mut witness = pow_mod(base, odd_part, number);
        if witness == 1 || witness == number - 1 {
            continue;
        }

        for _ in 1..powers_of_two {
            witness = mul_mod(witness, witness, number);
            if witness == number - 1 {
                continue 'bases;
            }
        }

        return false;
    }

    true
}

fn pow_mod(mut base: u128, mut exponent: u128, modulus: u128) -> u128 {
    let mut result = 1;
    base %= modulus;

    while exponent > 0 {
        if exponent % 2 == 1 {
            result = mul_mod(result, base, modulus);
        }

        exponent /= 2;
        if exponent > 0 {
            base = mul_mod(base, base, modulus);
        }
    }

    result
}

fn mul_mod(mut left: u128, mut right: u128, modulus: u128) -> u128 {
    let mut product = 0;
    left %= modulus;

    while right > 0 {
        if right % 2 == 1 {
            product = add_mod(product, left, modulus);
        }

        right /= 2;
        if right > 0 {
            left = add_mod(left, left, modulus);
        }
    }

    product
}

fn add_mod(left: u128, right: u128, modulus: u128) -> u128 {
    if left >= modulus - right {
        left - (modulus - right)
    } else {
        left + right
    }
}

fn gcd(mut left: u128, mut right: u128) -> u128 {
    while right != 0 {
        let remainder = left % right;
        left = right;
        right = remainder;
    }

    left
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

fn format_factorization(number: u128) -> String {
    format_factorization_from_factors(number, &prime_factors(number))
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

    println!("Input: {number} ({} digits)", decimal_digit_count(number));
    println!("Automatic algorithm: {}", automatic_algorithm.label());
    println!("Result: {}", format_factorization(number));
    println!(
        "Algorithm timings ({}s timeout per algorithm):",
        DEBUG_ALGORITHM_TIMEOUT.as_secs()
    );

    for algorithm in [
        FactorizationAlgorithm::OriginalTrialDivision,
        FactorizationAlgorithm::SixKTrialDivision,
        FactorizationAlgorithm::PollardRho,
    ] {
        match benchmark_algorithm(number, algorithm)? {
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

fn main() {
    let command = match parse_command(env::args().skip(1)) {
        Ok(command) => command,
        Err(message) => {
            eprintln!("{message}");
            process::exit(2);
        }
    };

    match command {
        CliCommand::Factor { number } => println!("{}", format_factorization(number)),
        CliCommand::Debug { number } => {
            if let Err(message) = print_debug_timings(number) {
                eprintln!("{message}");
                process::exit(1);
            }
        }
        CliCommand::Algorithm { algorithm, number } => {
            let factors = factor_with_algorithm(number, algorithm);
            println!("{}", format_factorization_from_factors(number, &factors));
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{
        CliCommand, FactorizationAlgorithm, factorization_algorithm, format_duration,
        format_factorization, is_probably_prime_miller_rabin, parse_command, prime_factors,
        prime_factors_trial_6k, prime_factors_trial_original,
    };

    #[test]
    fn factors_composite_numbers() {
        assert_eq!(prime_factors(84), vec![2, 2, 3, 7]);
        assert_eq!(prime_factors(13_860), vec![2, 2, 3, 3, 5, 7, 11]);
        assert_eq!(
            prime_factors_trial_original(13_860),
            vec![2, 2, 3, 3, 5, 7, 11]
        );
    }

    #[test]
    fn factors_prime_numbers() {
        assert_eq!(prime_factors(97), vec![97]);
    }

    #[test]
    fn factors_powers() {
        assert_eq!(prime_factors(1_024), vec![2; 10]);
    }

    #[test]
    fn uses_six_k_trial_division_for_short_numbers() {
        assert_eq!(
            factorization_algorithm(999_999_937),
            FactorizationAlgorithm::SixKTrialDivision
        );
        assert_eq!(prime_factors_trial_6k(13_860), vec![2, 2, 3, 3, 5, 7, 11]);
    }

    #[test]
    fn uses_pollard_rho_for_long_numbers() {
        assert_eq!(
            factorization_algorithm(1_000_000_016_000_000_063),
            FactorizationAlgorithm::PollardRho
        );
        assert_eq!(
            prime_factors(1_000_000_016_000_000_063),
            vec![1_000_000_007, 1_000_000_009]
        );
    }

    #[test]
    fn checks_probable_primes_with_miller_rabin() {
        assert!(is_probably_prime_miller_rabin(1_000_000_007));
        assert!(!is_probably_prime_miller_rabin(
            1_000_000_007 * 1_000_000_009
        ));
    }

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
