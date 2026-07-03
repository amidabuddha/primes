const TRIAL_DIVISION_MAX_DIGITS: usize = 15;
const POLLARD_RHO_BATCH_SIZE: u128 = 128;
const MILLER_RABIN_BASES: [u128; 16] = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FactorizationAlgorithm {
    OriginalTrialDivision,
    SixKTrialDivision,
    PollardRho,
}

impl FactorizationAlgorithm {
    pub(crate) fn label(self) -> &'static str {
        match self {
            FactorizationAlgorithm::OriginalTrialDivision => "original trial division",
            FactorizationAlgorithm::SixKTrialDivision => "6k +/- 1 trial division",
            FactorizationAlgorithm::PollardRho => "Pollard's Rho",
        }
    }

    pub(crate) fn cli_name(self) -> &'static str {
        match self {
            FactorizationAlgorithm::OriginalTrialDivision => "original",
            FactorizationAlgorithm::SixKTrialDivision => "six-k",
            FactorizationAlgorithm::PollardRho => "pollard-rho",
        }
    }

    pub(crate) fn from_cli_name(name: &str) -> Option<Self> {
        match name {
            "original" => Some(FactorizationAlgorithm::OriginalTrialDivision),
            "six-k" => Some(FactorizationAlgorithm::SixKTrialDivision),
            "pollard-rho" => Some(FactorizationAlgorithm::PollardRho),
            _ => None,
        }
    }
}

pub(crate) fn prime_factors(number: u128) -> Vec<u128> {
    factor_with_algorithm(number, factorization_algorithm(number))
}

pub(crate) fn factor_with_algorithm(number: u128, algorithm: FactorizationAlgorithm) -> Vec<u128> {
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

pub(crate) fn factorization_algorithm(number: u128) -> FactorizationAlgorithm {
    if decimal_digit_count(number) <= TRIAL_DIVISION_MAX_DIGITS {
        FactorizationAlgorithm::SixKTrialDivision
    } else {
        FactorizationAlgorithm::PollardRho
    }
}

pub(crate) fn decimal_digit_count(number: u128) -> usize {
    number.checked_ilog10().unwrap_or(0) as usize + 1
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
        let start = 2 + constant_mod % (number - 3);

        if let Some(divisor) = brent_cycle_divisor(number, constant_mod, start) {
            return divisor;
        }

        constant += 1;
    }
}

fn brent_cycle_divisor(number: u128, constant: u128, start: u128) -> Option<u128> {
    let mut current = start;
    let mut batch_start = start;
    let mut segment_length: u128 = 1;
    let mut product: u128 = 1;
    let mut divisor = 1;

    let anchor = loop {
        let anchor = current;
        for _ in 0..segment_length {
            current = pollard_rho_step(current, constant, number);
        }

        let mut compared = 0;
        while compared < segment_length && divisor == 1 {
            batch_start = current;
            let batch_size = POLLARD_RHO_BATCH_SIZE.min(segment_length - compared);

            for _ in 0..batch_size {
                current = pollard_rho_step(current, constant, number);
                product = mul_mod(product, anchor.abs_diff(current), number);
            }

            divisor = gcd(product, number);
            compared += batch_size;
        }

        if divisor != 1 {
            break anchor;
        }

        segment_length *= 2;
    };

    if divisor == number {
        divisor = 1;
        while divisor == 1 {
            batch_start = pollard_rho_step(batch_start, constant, number);
            divisor = gcd(anchor.abs_diff(batch_start), number);
        }
    }

    (divisor < number).then_some(divisor)
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

#[cfg(test)]
mod tests {
    use super::{
        FactorizationAlgorithm, decimal_digit_count, factor_with_algorithm,
        factorization_algorithm, is_probably_prime_miller_rabin, prime_factors,
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
    fn counts_decimal_digits() {
        assert_eq!(decimal_digit_count(0), 1);
        assert_eq!(decimal_digit_count(1), 1);
        assert_eq!(decimal_digit_count(9), 1);
        assert_eq!(decimal_digit_count(10), 2);
        assert_eq!(decimal_digit_count(u128::MAX), 39);
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
    fn factors_directly_with_pollard_rho() {
        assert_eq!(
            factor_with_algorithm(84, FactorizationAlgorithm::PollardRho),
            vec![2, 2, 3, 7]
        );
        assert_eq!(
            factor_with_algorithm(999_999_943_999_999_559, FactorizationAlgorithm::PollardRho),
            vec![999_999_937, 1_000_000_007]
        );
    }

    #[test]
    fn checks_probable_primes_with_miller_rabin() {
        assert!(is_probably_prime_miller_rabin(1_000_000_007));
        assert!(!is_probably_prime_miller_rabin(
            1_000_000_007 * 1_000_000_009
        ));
    }
}
