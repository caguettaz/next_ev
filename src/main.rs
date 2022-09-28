use std::fmt::Display;

use chrono::{naive::NaiveDate, Datelike, Duration};
use clap::Parser;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

fn digit_count(n: u64, base: u8) -> u32 {
    (n as f64 + 1.).log(base as f64).ceil() as u32
}

fn first_digit(n: u64, base: u8) -> u8 {
    (n / (base as u64).pow(digit_count(n, base) - 1)) as u8
}

fn add_months_to_date(date: &NaiveDate, mut months: u64) -> NaiveDate {
    let mut year = date.year();
    let mut month = date.month();

    months += month as u64 - 1;

    year += months as i32 / 12;
    month = (months as u32 % 12) + 1;

    NaiveDate::from_ymd(year, month, date.day())
}

#[derive(Default, Debug)]
struct Pattern {
    value: u64,
    base: u8,
}

impl Display for Pattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.base {
            10 => write!(f, "{}", self.value),
            16 => write!(f, "{:#0x}", self.value),
            _ => panic!("unhandled base {}", self.base),
        }
    }
}

trait PatternFinder {
    fn find_next(&self, n: u64, base: u8) -> Pattern;
}

#[derive(Default)]
struct RoundNumberFinder {}

impl PatternFinder for RoundNumberFinder {
    fn find_next(&self, n: u64, base: u8) -> Pattern {
        const MAX_EXPONENT: u32 = 19;

        // n - 1, so that powers of base are handled OK
        let digits = digit_count(n - 1, base);
        let first_digit = first_digit(n - 1, base);

        if digits > MAX_EXPONENT || digits < 2 {
            Pattern { value: 0, base }
        } else {
            Pattern {
                value: ((first_digit + 1) as u64) * (base as u64).pow(digits - 1),
                base,
            }
        }
    }
}

#[derive(Default)]
struct RepeatedNumberFinder {}

impl RepeatedNumberFinder {
    fn get_repeat_number(&self, first_digit: u8, digits: u32, base: u8) -> u64 {
        let mut res = first_digit as u64;

        for _ in 1..digits {
            res = res * (base as u64) + first_digit as u64;
        }

        res
    }
}

impl PatternFinder for RepeatedNumberFinder {
    fn find_next(&self, n: u64, base: u8) -> Pattern {
        let digits = digit_count(n, base);
        let first_digit = first_digit(n, base);

        let res = self.get_repeat_number(first_digit, digits, base);

        if res >= n {
            Pattern { value: res, base }
        } else {
            Pattern {
                value: self.get_repeat_number(first_digit + 1, digits, base),
                base,
            }
        }
    }
}

#[derive(Default)]
struct SequenceFinder {
    reverse: bool,
}

impl PatternFinder for SequenceFinder {
    fn find_next(&self, n: u64, base: u8) -> Pattern {
        let mut res = 1_u64;

        for i in 2..=(base as u64 - 1) {
            if res >= n {
                return Pattern { value: res, base };
            }
            if self.reverse {
                let d = digit_count(res, base);
                res += (base as u64).pow(d) * i;
            } else {
                res = res * (base as u64) + i;
            }
        }

        Pattern { value: 0, base }
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
            Box::new(SequenceFinder::default()),
            Box::new(SequenceFinder { reverse: true }),
        ];

        Self { pattern_finders }
    }

    fn find_patterns(&self, n: u64, base: u8) -> Vec<Pattern> {
        let mut res: Vec<Pattern> = self
            .pattern_finders
            .iter()
            .map(|f| f.find_next(n, base))
            .filter(|p| p.value != 0)
            .collect();

        res.sort_by(|l, r| l.value.cmp(&r.value));
        res
    }
}

impl PatternFinder for MultiPatternFinder {
    fn find_next(&self, n: u64, base: u8) -> Pattern {
        let mut best_pattern = Pattern::default();
        let mut best_delta = u64::MAX;

        // println!("***********");

        for p in self.find_patterns(n, base) {
            let delta = p.value - n;

            // println!("pattern: {p}, delta: {delta}");

            if delta < best_delta {
                best_delta = delta;
                best_pattern = p;
            }
        }

        best_pattern
    }
}

fn _test_pattern_finders() {
    let f = MultiPatternFinder::new();

    let nums = vec![9, 99, 100, 1000, 2000, 4321, 123456];
    let bases = vec![10_u8, 0x10_u8];

    for n in nums {
        for b in &bases {
            for p in f.find_patterns(n, *b) {
                println!("{n} -> {}", p);
            }
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

#[derive(Debug, Copy, Clone, EnumIter)]
enum TimeUnit {
    Second,
    Minute,
    Hour,
    Day,
    Week,
    Month,
}

impl TimeUnit {
    fn to_seconds(&self) -> Option<f64> {
        match self {
            TimeUnit::Second => Some(1.),
            TimeUnit::Minute => Some(60.),
            TimeUnit::Hour => Some(60. * 60.),
            TimeUnit::Day => Some(60. * 60. * 24.),
            TimeUnit::Week => Some(60. * 60. * 24. * 7.),
            TimeUnit::Month => None,
        }
    }
}

impl Display for TimeUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimeUnit::Second => write!(f, "second"),
            TimeUnit::Minute => write!(f, "minute"),
            TimeUnit::Hour => write!(f, "hour"),
            TimeUnit::Day => write!(f, "day"),
            TimeUnit::Week => write!(f, "week"),
            TimeUnit::Month => write!(f, "month"),
        }
    }
}

// TODO: fix below
#[derive(Debug)]
struct DeltaCandidate {
    pattern: Pattern,
    unit: TimeUnit,
}

impl DeltaCandidate {
    fn to_seconds(&self) -> Option<u64> {
        match self.unit.to_seconds() {
            Some(s) => Some(self.pattern.value * (s as u64)),
            None => None,
        }
    }

    fn add_to_date(&self, date: &NaiveDate) -> NaiveDate {
        if let Some(s) = self.to_seconds() {
            return *date + Duration::seconds(s as i64);
        }

        match self.unit {
            TimeUnit::Month => return add_months_to_date(date, self.pattern.value),
            _ => return NaiveDate::default(),
        }
    }
}

impl Display for DeltaCandidate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {}{}",
            self.pattern,
            self.unit,
            if self.pattern.value > 1 { "s" } else { "" }
        )
    }
}

fn get_duration_str(d: &Duration) -> String {
    if d.num_weeks() > 20 {
        let months = d.num_days() * 2 / 61;
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
    // _test_pattern_finders();

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

    let bases = vec![10, 0x10];
    let seconds = delta.num_seconds().abs() as u64;

    for time_unit in TimeUnit::iter() {
        for base in &bases {
            if let Some(s) = time_unit.to_seconds() {
                let n = (seconds as f64 / s).ceil() as u64;
                res.push(DeltaCandidate {
                    pattern: f.find_next(n, *base),
                    unit: time_unit,
                });
            }
        }
    }

    let mut months = (cur_date.year() as u64 - naive_date.year() as u64) * 12
        + cur_date.month() as u64
        - naive_date.month() as u64;

    if cur_date.day() > naive_date.day() {
        months += 1;
    }

    for base in &bases {
        res.push(DeltaCandidate {
            pattern: f.find_next(months, *base),
            unit: TimeUnit::Month,
        });
    }

    res.sort_by(|l, r| l.add_to_date(&naive_date).cmp(&r.add_to_date(&naive_date)));

    let best = res.first().unwrap();
    let best_date = best.add_to_date(&naive_date);

    let best_wait_str = get_duration_str(&(best_date - cur_date));

    println!(
        "It'll be {} on {} (in {})",
        best,
        best_date.format("%Y-%m-%d"),
        best_wait_str
    );
}
