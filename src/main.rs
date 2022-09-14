use chrono::{naive::NaiveDate, Duration};
use clap::Parser;

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

fn _test_pattern_finders() {
    let f = MultiPatternFinder::new();

    let nums = vec![9, 99, 100, 4321, 123456];

    for n in nums {
        for p in f.find_patterns(n) {
            println!("{n} -> {p}");
        }
    }
}

#[derive(Parser, Debug)]
#[clap(version)]
struct Args {
    /// Name of the person to greet
    #[clap()]
    date: String,
}

#[derive(Debug)]
struct DeltaCandidate {
    pattern: u64,
    unit: String,
    delta_sec: u64,
}

fn add_best_candidate(
    n: u64,
    unit: &str,
    candidates: &mut Vec<DeltaCandidate>,
    f: &MultiPatternFinder,
    delta_mul: u64,
) {
    let mut best_pattern: u64 = 0;
    let mut best_delta = u64::MAX;

    // println!("***********");

    for p in f.find_patterns(n) {
        let delta = p - n;

        // println!("pattern: {p}, delta: {delta}");

        if delta < best_delta {
            best_delta = delta;
            best_pattern = p;
        }
    }

    candidates.push(DeltaCandidate {
        pattern: best_pattern,
        unit: unit.to_string(),
        delta_sec: best_delta * delta_mul,
    });
}

fn get_duration_str(d: &Duration) -> String {
    if d.num_weeks() > 20 {
        let months = d.num_days() * 2 / 70;
        return format!("{months} months");
    } else if d.num_days() > 99 {
        return format!("{} weeks", d.num_weeks());
    } else if d.num_hours() > 72 {
        return format!("{} days", d.num_days());
    } else if d.num_minutes() > 60 {
        return format!("{} hours", d.num_hours());
    } else if d.num_seconds() > 60 {
        return format!("{} minutes", d.num_minutes());
    } else {
        return format!("{} seconds", d.num_seconds());
    }
}

fn main() {
    let args = Args::parse();

    let naive_date = NaiveDate::parse_from_str(&args.date, "%Y-%m-%d");

    if let Err(_) = naive_date {
        println!("Failed to parse date {}", &args.date);
        return;
    }

    let naive_date = naive_date.unwrap();
    let cur_date = chrono::Local::now().date_naive();

    let delta = cur_date - naive_date;

    let mut res: Vec<DeltaCandidate> = Vec::new();
    let f = MultiPatternFinder::new();

    let seconds = delta.num_seconds().abs() as u64;
    add_best_candidate(seconds, "seconds", &mut res, &f, 1);

    let minutes = delta.num_minutes().abs() as u64;
    add_best_candidate(minutes, "minutes", &mut res, &f, 60);

    let hours = delta.num_hours().abs() as u64;
    add_best_candidate(hours, "hours", &mut res, &f, 60 * 60);

    let days = delta.num_days().abs() as u64;
    add_best_candidate(days, "days", &mut res, &f, 60 * 60 * 24);

    res.sort_by(|l, r| l.delta_sec.cmp(&r.delta_sec));

    let best = res.first().unwrap();
    let best_duration = Duration::seconds(best.delta_sec as i64);
    let best_duration_str = get_duration_str(&best_duration);

    println!(
        "It'll be {} {} in {}",
        best.pattern, best.unit, best_duration_str
    );
}
