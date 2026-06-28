use std::{env, process};

fn prime_factors(mut number: u128) -> Vec<u128> {
    let mut factors = Vec::new();
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
    use super::{format_factorization, parse_number, prime_factors};

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
