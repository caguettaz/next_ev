fn digit_count(n: u64) -> u32 {
    (n as f64 + 1.).log(10.).ceil() as u32
}

fn first_digit(n: u64) -> u8 {
    (n / (10_u64).pow(digit_count(n) - 1)) as u8
}

trait PatternFinder {
    fn find_next(&self, n: u64) -> u64;
}

#[derive(Default)]
struct RoundNumberFinder {}

impl PatternFinder for RoundNumberFinder {
    fn find_next(&self, n: u64) -> u64 {
        const MAX_EXPONENT: u32 = 19;

        // n - 1, so that powers of 10 are handled OK
        let digits = digit_count(n - 1);

        if digits > MAX_EXPONENT {
            0
        } else {
            (10_u64).pow(digits)
        }
    }
}

#[derive(Default)]
struct RepeatedNumberFinder {}

impl RepeatedNumberFinder {
    fn get_repeat_number(&self, first_digit: u8, digits: u32) -> u64 {
        let mut res = first_digit as u64;

        for _ in 1..digits {
            res = res * 10 + first_digit as u64;
        }

        res
    }
}

impl PatternFinder for RepeatedNumberFinder {
    fn find_next(&self, n: u64) -> u64 {
        let digits = digit_count(n);
        let first_digit = first_digit(n);

        let res = self.get_repeat_number(first_digit, digits);

        if res >= n {
            res
        } else {
            self.get_repeat_number(first_digit + 1, digits)
        }
    }
}

#[derive(Default)]
struct SequenceFinder {
    reverse: bool,
}

impl PatternFinder for SequenceFinder {
    fn find_next(&self, n: u64) -> u64 {
        let mut res = 1_u64;

        for i in 2..=9 {
            if res >= n {
                return res;
            }
            if self.reverse {
                let d = digit_count(res);
                res += 10_u64.pow(d) * i;
            } else {
                res = res * 10 + i;
            }
        }

        0
    }
}

struct MultiPatternFinder {
    pattern_finders: Vec<Box<dyn PatternFinder>>,
}

impl MultiPatternFinder {
    fn new() -> Self {
        let pattern_finders: Vec<Box<dyn PatternFinder>> = vec![
            Box::new(RoundNumberFinder::default()),
            Box::new(RepeatedNumberFinder::default()),
            Box::new(SequenceFinder::default()),
            Box::new(SequenceFinder { reverse: true }),
        ];

        Self { pattern_finders }
    }

    fn find_patterns(&self, n: u64) -> Vec<u64> {
        let mut res: Vec<u64> = self
            .pattern_finders
            .iter()
            .map(|f| f.find_next(n))
            .filter(|n| *n != 0)
            .collect();

        res.sort();
        res
    }
}

fn main() {
    let f = MultiPatternFinder::new();

    let nums = vec![9, 99, 100, 4321, 123456];

    for n in nums {
        for p in f.find_patterns(n) {
            println!("{n} -> {p}");
        }
    }
}
