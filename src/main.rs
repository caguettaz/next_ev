fn digit_count(n: u64) -> u32 {
    (n as f64 + 1.).log(10.).ceil() as u32
}

fn first_digit(n: u64) -> u8 {
    (n / (10_u64).pow(digit_count(n) - 1)) as u8
}

trait PatternFinder {
    fn find_next(&self, n: u64) -> u64;
}

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

fn main() {
    let mut finders = Vec::new();

    let round_num_finder = RoundNumberFinder {};
    finders.push(&round_num_finder as &dyn PatternFinder);

    let repeat_num_finder = RepeatedNumberFinder {};
    finders.push(&repeat_num_finder);

    let fw_seq_finder = SequenceFinder::default();
    finders.push(&fw_seq_finder);

    let bw_seq_finder = SequenceFinder { reverse: true };
    finders.push(&bw_seq_finder);

    let nums = vec![9, 99, 100, 4321, 123456];

    for n in nums {
        for f in &finders {
            let next = f.find_next(n);

            println!("{n} -> {next}");
        }
    }
}
