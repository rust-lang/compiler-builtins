//! Simple check of integer and float ops, for giving us an idea of how many test cases we can
//! expect to execute in a reasonable amount of time.

#![allow(clippy::type_complexity)]

use std::hint::black_box as bb;
use std::io::Write;
use std::process::exit;
use std::time::{Duration, Instant};
use std::{io, thread};

fn main() {
    // We need a real-ish test harness to not break nextest.
    let mut opts = getopts::Options::new();
    opts.optflag("", "list", "list available benchmarks");
    opts.optflag("", "ignored", "include ignored benchmarks");
    opts.optopt("", "format", "pretty or terse", "FORMAT");
    opts.optflag("", "bench", "invoke benchmarks (passed by Cargo)");
    let m = opts.parse(std::env::args().skip(1)).unwrap_or_else(|e| {
        eprintln!("{e}");
        exit(1);
    });
    let mut run = vec![];

    if m.opt_present("list") {
        for (name, _func) in BENCHES {
            println!("{name}: benchmark");
        }
        exit(0);
    }

    for name in &m.free {
        let Some(func) = BENCHES
            .iter()
            .find_map(|(n, func)| (name == *n).then_some(func))
        else {
            panic!("unrecognized benchmark {name}");
        };
        run.push(func);
    }

    if m.free.is_empty() {
        run.extend(BENCHES.iter().map(|(_name, func)| func));
    }

    let mut res = Results::default();

    for func in run {
        func(&mut res);
    }

    report_score(&res);
}

#[derive(Default)]
struct Results {
    int_mul: Option<Duration>,
    float_mul: Option<Duration>,
}

fn report_score(res: &Results) {
    let Some(int_mul) = res.int_mul else {
        println!("couldn't calculate performance score; missing int_mul");
        return;
    };
    let Some(float_mul) = res.float_mul else {
        println!("couldn't calculate performance score; missing float_mul");
        return;
    };
    let threads = match thread::available_parallelism() {
        Ok(v) => v,
        Err(e) => {
            println!("couldn't calculate performance score; missing parallelism ({e})");
            return;
        }
    };
    println!("Available parallelism: {threads}");

    // Super simple metric to give us a rough idea of how machines compare.
    let score = ((1.0 / int_mul.as_secs_f32()) + (1.0 / float_mul.as_secs_f32())) * 200.0;
    println!("Single-core performance score: {score:.0}");
    println!("Performance score: {:.0}", score * threads.get() as f32);
}

const BENCHES: &[(&str, fn(&mut Results))] =
    &[("int_mul", bench_int_mul), ("float_mul", bench_float_mul)];

fn bench_int_mul(res: &mut Results) {
    print!("Starting integer multiplication bench... ");
    io::stdout().flush().unwrap();
    let start = Instant::now();
    for i in 0..=i128::from(u32::MAX) {
        if i % 4 != 0 {
            continue;
        }
        let i = i | (i << 32) | (i << 64) | (i << 96);
        bb(bb(i).overflowing_mul(bb(i)));
    }
    let elapsed = start.elapsed();
    println!("completed in {elapsed:?}");
    res.int_mul = Some(elapsed);
}

fn bench_float_mul(res: &mut Results) {
    print!("Starting float multiplication bench... ");
    io::stdout().flush().unwrap();
    let start = Instant::now();
    for i in 0..=u32::MAX {
        if i % 4 != 0 {
            continue;
        }
        let f = f32::from_bits(i);
        bb(bb(f) * bb(f));
    }
    let elapsed = start.elapsed();
    println!("completed in {elapsed:?}");
    res.float_mul = Some(elapsed);
}
