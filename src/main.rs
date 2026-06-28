use std::{env, process};

const TRIAL_DIVISION_MAX_DIGITS: usize = 15;
const MILLER_RABIN_BASES: [u128; 16] = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FactorizationAlgorithm {
    SixKTrialDivision,
    PollardRho,
}

fn prime_factors(number: u128) -> Vec<u128> {
    let mut factors = match factorization_algorithm(number) {
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

fn format_factorization(number: u128) -> String {
    let factors = prime_factors(number);

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

fn parse_number(mut args: impl Iterator<Item = String>) -> Result<u128, String> {
    let input = args
        .next()
        .ok_or_else(|| String::from("Usage: primes <positive-integer>"))?;

    if args.next().is_some() {
        return Err(String::from("Usage: primes <positive-integer>"));
    }

    let number = input
        .parse::<u128>()
        .map_err(|_| format!("Invalid number: {input}"))?;

    if number == 0 {
        return Err(String::from("Number must be greater than 0"));
    }

    Ok(number)
}

fn main() {
    let number = match parse_number(env::args().skip(1)) {
        Ok(number) => number,
        Err(message) => {
            eprintln!("{message}");
            process::exit(2);
        }
    };

    println!("{}", format_factorization(number));
}

#[cfg(test)]
mod tests {
    use super::{
        FactorizationAlgorithm, factorization_algorithm, format_factorization,
        is_probably_prime_miller_rabin, parse_number, prime_factors, prime_factors_trial_6k,
    };

    #[test]
    fn factors_composite_numbers() {
        assert_eq!(prime_factors(84), vec![2, 2, 3, 7]);
        assert_eq!(prime_factors(13_860), vec![2, 2, 3, 3, 5, 7, 11]);
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
        assert!(parse_number(Vec::<String>::new().into_iter()).is_err());
        assert!(parse_number(["10", "20"].map(String::from).into_iter()).is_err());
        assert!(parse_number(["abc"].map(String::from).into_iter()).is_err());
        assert!(parse_number(["0"].map(String::from).into_iter()).is_err());
    }
}
