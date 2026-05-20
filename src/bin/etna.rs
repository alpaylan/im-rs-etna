// ETNA workload runner for im-rs.
//
// Usage: cargo run --release --bin etna -- <tool> <property>
//   tool:     etna | proptest | quickcheck | crabcheck | hegel
//   property: PathNextBacktrack | RangeOffByOne | RrbDebugPop
//             | RrbDensityCheck | PtrEqPrecedence | EqSingleChunk | All
//
// Every invocation prints exactly one JSON line to stdout with shape
//   {"status":"passed|failed|aborted","tests":N,"discards":0,
//    "time":"<us>us","counterexample":STRING|null,"error":STRING|null,
//    "tool":"...","property":"..."}
// and exits with status 0 (the one exception is argv parsing, which exits 2).
// Etna's log_process_output parses stdout line-by-line for JSON; a non-zero
// exit is recorded as status:aborted regardless of any PASS/FAIL text.

use crabcheck::quickcheck as crabcheck_qc;
use hegel::{generators as hgen, Hegel, Settings as HegelSettings, TestCase};
use im::etna::{
    property_eq_single_chunk, property_path_next_backtrack, property_ptr_eq_precedence,
    property_range_off_by_one, property_rrb_debug_pop, property_rrb_density_check,
    property_rrb_density_check_shape, PropertyResult,
};
use proptest::prelude::*;
use proptest::test_runner::{Config as ProptestConfig, TestCaseError, TestRunner};
use quickcheck::{QuickCheck, ResultStatus, TestResult};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

#[derive(Default, Clone, Copy)]
struct Metrics {
    inputs: u64,
    elapsed_us: u128,
}

impl Metrics {
    fn combine(self, other: Metrics) -> Metrics {
        Metrics {
            inputs: self.inputs + other.inputs,
            elapsed_us: self.elapsed_us + other.elapsed_us,
        }
    }
}

type Outcome = (Result<(), String>, Metrics);

fn to_err(r: PropertyResult) -> Result<(), String> {
    match r {
        PropertyResult::Pass | PropertyResult::Discard => Ok(()),
        PropertyResult::Fail(m) => Err(m),
    }
}

const ALL_PROPERTIES: &[&str] = &[
    "PathNextBacktrack",
    "RangeOffByOne",
    "RrbDebugPop",
    "RrbDensityCheck",
    "PtrEqPrecedence",
    "EqSingleChunk",
];

fn run_all<F: FnMut(&str) -> Outcome>(mut f: F) -> Outcome {
    let mut total = Metrics::default();
    for p in ALL_PROPERTIES {
        let (r, m) = f(p);
        total = total.combine(m);
        if let Err(e) = r {
            return (Err(e), total);
        }
    }
    (Ok(()), total)
}

// ---------- Canonical frozen witnesses ----------

fn check_path_next_backtrack() -> Result<(), String> {
    to_err(property_path_next_backtrack(1000, 100, 100))
}

fn check_range_off_by_one() -> Result<(), String> {
    to_err(property_range_off_by_one(1000, 501))
}

fn check_rrb_debug_pop() -> Result<(), String> {
    to_err(property_rrb_debug_pop(5000))
}

fn check_rrb_density_check() -> Result<(), String> {
    to_err(property_rrb_density_check(0))
}

fn check_ptr_eq_precedence() -> Result<(), String> {
    to_err(property_ptr_eq_precedence(200, 0))
}

fn check_eq_single_chunk() -> Result<(), String> {
    to_err(property_eq_single_chunk((0..40).collect()))
}

// ---------- etna (deterministic witness replay) ----------

fn run_etna_property(property: &str) -> Outcome {
    if property == "All" {
        return run_all(run_etna_property);
    }
    let t0 = Instant::now();
    let result = match property {
        "PathNextBacktrack" => check_path_next_backtrack(),
        "RangeOffByOne" => check_range_off_by_one(),
        "RrbDebugPop" => check_rrb_debug_pop(),
        "RrbDensityCheck" => check_rrb_density_check(),
        "PtrEqPrecedence" => check_ptr_eq_precedence(),
        "EqSingleChunk" => check_eq_single_chunk(),
        _ => {
            return (
                Err(format!("Unknown property for etna: {property}")),
                Metrics::default(),
            )
        }
    };
    (
        result,
        Metrics {
            inputs: 1,
            elapsed_us: t0.elapsed().as_micros(),
        },
    )
}

// ---------- proptest ----------

fn small_u32() -> BoxedStrategy<u32> {
    // Keep the property workload fast but large enough to exercise the bug
    // paths: OrdMap sizes up to ~4k, Vector segment lengths up to ~2k.
    (0u32..4096u32).boxed()
}

fn run_proptest_property(property: &str) -> Outcome {
    if property == "All" {
        return run_all(run_proptest_property);
    }
    let counter = Arc::new(AtomicU64::new(0));
    let t0 = Instant::now();
    // Keep cases small — the im-rs property bodies build 1000-element OrdMaps
    // and multi-million-element RRB trees; 100 cases is plenty to hit them.
    let cfg = ProptestConfig {
        cases: 64,
        max_shrink_iters: 64,
        ..ProptestConfig::default()
    };
    let mut runner = TestRunner::new(cfg);
    let c = counter.clone();
    let result: Result<(), String> = match property {
        "PathNextBacktrack" => runner
            .run(&(small_u32(), small_u32(), small_u32()), move |(n, lo, span)| {
                c.fetch_add(1, Ordering::Relaxed);
                match property_path_next_backtrack(n, lo, span) {
                    PropertyResult::Pass | PropertyResult::Discard => Ok(()),
                    PropertyResult::Fail(m) => Err(TestCaseError::fail(m)),
                }
            })
            .map_err(|e| e.to_string()),
        "RangeOffByOne" => runner
            .run(&(small_u32(), small_u32()), move |(n, hi)| {
                c.fetch_add(1, Ordering::Relaxed);
                match property_range_off_by_one(n, hi) {
                    PropertyResult::Pass | PropertyResult::Discard => Ok(()),
                    PropertyResult::Fail(m) => Err(TestCaseError::fail(m)),
                }
            })
            .map_err(|e| e.to_string()),
        "RrbDebugPop" => runner
            .run(&(0u32..2048u32), move |n| {
                c.fetch_add(1, Ordering::Relaxed);
                match property_rrb_debug_pop(n) {
                    PropertyResult::Pass | PropertyResult::Discard => Ok(()),
                    PropertyResult::Fail(m) => Err(TestCaseError::fail(m)),
                }
            })
            .map_err(|e| e.to_string()),
        "RrbDensityCheck" => {
            // Widened generator: span keep_every (n_removed), build size,
            // and post-build op mode so the distribution covers small &
            // medium Vectors (which don't form a level-2 sparse RRB tree)
            // and a variety of post-build op sequences. Only a slice of the
            // input space hits the canonical bug-triggering shape, so the
            // mean tests-to-failure under the buggy build is no longer 1.
            // Each call still allocates a ~262k-element Vector when
            // `size_factor==0`, so cap cases conservatively.
            let cfg2 = ProptestConfig {
                cases: 12,
                max_shrink_iters: 8,
                ..ProptestConfig::default()
            };
            let mut rr = TestRunner::new(cfg2);
            let cc = counter.clone();
            rr.run(
                &(0u32..16u32, 0u32..8u32, 0u32..6u32),
                move |(n, sf, om)| {
                    cc.fetch_add(1, Ordering::Relaxed);
                    match property_rrb_density_check_shape(n, sf, om) {
                        PropertyResult::Pass | PropertyResult::Discard => Ok(()),
                        PropertyResult::Fail(m) => Err(TestCaseError::fail(m)),
                    }
                },
            )
            .map_err(|e| e.to_string())
        }
        "PtrEqPrecedence" => runner
            .run(&(small_u32(), small_u32()), move |(n, slot)| {
                c.fetch_add(1, Ordering::Relaxed);
                match property_ptr_eq_precedence(n, slot) {
                    PropertyResult::Pass | PropertyResult::Discard => Ok(()),
                    PropertyResult::Fail(m) => Err(TestCaseError::fail(m)),
                }
            })
            .map_err(|e| e.to_string()),
        "EqSingleChunk" => runner
            .run(&prop::collection::vec(any::<i32>(), 32..61), move |xs| {
                c.fetch_add(1, Ordering::Relaxed);
                match property_eq_single_chunk(xs) {
                    PropertyResult::Pass | PropertyResult::Discard => Ok(()),
                    PropertyResult::Fail(m) => Err(TestCaseError::fail(m)),
                }
            })
            .map_err(|e| e.to_string()),
        _ => {
            return (
                Err(format!("Unknown property for proptest: {property}")),
                Metrics::default(),
            )
        }
    };
    let elapsed_us = t0.elapsed().as_micros();
    let inputs = counter.load(Ordering::Relaxed);
    (result, Metrics { inputs, elapsed_us })
}

// ---------- quickcheck (fork with `etna` feature) ----------

static QC_COUNTER: AtomicU64 = AtomicU64::new(0);

fn qc_path_next_backtrack(n: u16, lo: u16, span: u16) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_path_next_backtrack(n as u32, lo as u32, span as u32) {
        PropertyResult::Pass => TestResult::passed(),
        PropertyResult::Discard => TestResult::discard(),
        PropertyResult::Fail(_) => TestResult::failed(),
    }
}

fn qc_range_off_by_one(n: u16, hi: u16) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_range_off_by_one(n as u32, hi as u32) {
        PropertyResult::Pass => TestResult::passed(),
        PropertyResult::Discard => TestResult::discard(),
        PropertyResult::Fail(_) => TestResult::failed(),
    }
}

fn qc_rrb_debug_pop(n: u16) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_rrb_debug_pop(n as u32) {
        PropertyResult::Pass => TestResult::passed(),
        PropertyResult::Discard => TestResult::discard(),
        PropertyResult::Fail(_) => TestResult::failed(),
    }
}

fn qc_rrb_density_check(n: u8, sf: u8, om: u8) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    // Widened: include size_factor and op_mode so most random inputs build a
    // smaller Vector or skip the remove/split/append combo. Only inputs that
    // happen to land on (sf == 0 || sf == 7) AND (om == 0 || om == 5) hit the
    // canonical bug-triggering shape; the rest pass even on the buggy build.
    match property_rrb_density_check_shape(n as u32, sf as u32, om as u32) {
        PropertyResult::Pass => TestResult::passed(),
        PropertyResult::Discard => TestResult::discard(),
        PropertyResult::Fail(_) => TestResult::failed(),
    }
}

fn qc_ptr_eq_precedence(n: u16, slot: u16) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_ptr_eq_precedence(n as u32, slot as u32) {
        PropertyResult::Pass => TestResult::passed(),
        PropertyResult::Discard => TestResult::discard(),
        PropertyResult::Fail(_) => TestResult::failed(),
    }
}

// QuickCheck (fork) requires `Display` on arguments for counterexample
// reporting. `Vec<i32>` doesn't implement Display, so we pass a seed and
// expand it into a small Vec<i32> inside.
fn qc_eq_single_chunk(len_byte: u8, seed: u64) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    let len = (len_byte as usize % 29) + 32;
    let mut xs = Vec::with_capacity(len);
    let mut x = seed;
    for _ in 0..len {
        xs.push(x as i32);
        x = x.wrapping_mul(6364136223846793005).wrapping_add(1);
    }
    match property_eq_single_chunk(xs) {
        PropertyResult::Pass => TestResult::passed(),
        PropertyResult::Discard => TestResult::discard(),
        PropertyResult::Fail(_) => TestResult::failed(),
    }
}

fn run_quickcheck_property(property: &str) -> Outcome {
    if property == "All" {
        return run_all(run_quickcheck_property);
    }
    QC_COUNTER.store(0, Ordering::Relaxed);
    let t0 = Instant::now();
    let mut qc = QuickCheck::new().tests(64).max_tests(256);
    let result = match property {
        "PathNextBacktrack" => {
            qc.quicktest(qc_path_next_backtrack as fn(u16, u16, u16) -> TestResult)
        }
        "RangeOffByOne" => qc.quicktest(qc_range_off_by_one as fn(u16, u16) -> TestResult),
        "RrbDebugPop" => qc.quicktest(qc_rrb_debug_pop as fn(u16) -> TestResult),
        "RrbDensityCheck" => {
            let mut qcd = QuickCheck::new().tests(12).max_tests(48);
            qcd.quicktest(qc_rrb_density_check as fn(u8, u8, u8) -> TestResult)
        }
        "PtrEqPrecedence" => qc.quicktest(qc_ptr_eq_precedence as fn(u16, u16) -> TestResult),
        "EqSingleChunk" => qc.quicktest(qc_eq_single_chunk as fn(u8, u64) -> TestResult),
        _ => {
            return (
                Err(format!("Unknown property for quickcheck: {property}")),
                Metrics::default(),
            )
        }
    };
    let elapsed_us = t0.elapsed().as_micros();
    let inputs = QC_COUNTER.load(Ordering::Relaxed);
    let metrics = Metrics { inputs, elapsed_us };
    let status = match result.status {
        ResultStatus::Finished => Ok(()),
        ResultStatus::Failed { arguments } => Err(format!(
            "quickcheck failed with counterexample: ({})",
            arguments.join(" ")
        )),
        ResultStatus::Aborted { err } => Err(format!("quickcheck aborted: {err:?}")),
        ResultStatus::TimedOut => Err("quickcheck timed out".to_string()),
        ResultStatus::GaveUp => Err(format!(
            "quickcheck gave up: passed={}, discarded={}",
            result.n_tests_passed, result.n_tests_discarded
        )),
    };
    (status, metrics)
}

// ---------- crabcheck ----------

static CC_COUNTER: AtomicU64 = AtomicU64::new(0);

fn cc_range_off_by_one((n, hi): (usize, usize)) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_range_off_by_one((n as u32).wrapping_mul(17), (hi as u32).wrapping_mul(13)) {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn cc_rrb_debug_pop(n: usize) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_rrb_debug_pop((n as u32).wrapping_mul(31)) {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn cc_ptr_eq_precedence((n, slot): (usize, usize)) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_ptr_eq_precedence((n as u32).wrapping_mul(17), (slot as u32).wrapping_mul(7)) {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn cc_eq_single_chunk_seeded(seed: u32) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    let len = (seed as usize % 29) + 32;
    let mut xs = Vec::with_capacity(len);
    let mut x: u64 = seed as u64;
    for _ in 0..len {
        xs.push(x as i32);
        x = x.wrapping_mul(6364136223846793005).wrapping_add(1);
    }
    match property_eq_single_chunk(xs) {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

// Seeded single-argument variants for bounded runs.
fn cc_path_next_backtrack_seeded(seed: u32) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_path_next_backtrack(seed, seed.wrapping_mul(13), seed.wrapping_mul(11) + 1) {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn cc_rrb_density_check_seeded(seed: u32) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    // Widened: derive (n_removed, size_factor, op_mode) from `seed` so the
    // distribution spans small/medium Vectors and a variety of post-build
    // op sequences. Only ~1/(8*6) of seeds land on the canonical
    // bug-triggering shape, so under the buggy build crabcheck no longer
    // hits the failure on the very first test.
    // Bias the mapping with non-zero offsets so seed=0 doesn't immediately
    // collide with (size_factor=0, op_mode=0).
    let n_removed = seed;
    let size_factor = seed
        .wrapping_add(0x9E3779B9)
        .wrapping_mul(2654435761)
        .rotate_left(7)
        % 8;
    let op_mode = seed
        .wrapping_add(0xC2B2AE35)
        .wrapping_mul(40503)
        .rotate_left(13)
        % 6;
    match property_rrb_density_check_shape(n_removed, size_factor, op_mode) {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

// Bounded-iteration driver for properties too expensive to run 20k times.
fn cc_run_bounded(n: u32, f: fn(u32) -> Option<bool>) -> crabcheck_qc::RunResult {
    let mut passed: u64 = 0;
    let mut discarded: u64 = 0;
    for seed in 0..n {
        match f(seed) {
            Some(true) => passed += 1,
            Some(false) => {
                return crabcheck_qc::RunResult {
                    passed,
                    discarded,
                    status: crabcheck_qc::ResultStatus::Failed {
                        arguments: vec![format!("{seed}")],
                    },
                };
            }
            None => discarded += 1,
        }
    }
    crabcheck_qc::RunResult {
        passed,
        discarded,
        status: crabcheck_qc::ResultStatus::Finished,
    }
}

fn run_crabcheck_property(property: &str) -> Outcome {
    if property == "All" {
        return run_all(run_crabcheck_property);
    }
    CC_COUNTER.store(0, Ordering::Relaxed);
    let t0 = Instant::now();
    let result = match property {
        // Build a 20480-entry OrdMap and sweep ~640 range queries per call;
        // 20k cases is ~5 min. Cap to 32 to keep full runs responsive.
        "PathNextBacktrack" => cc_run_bounded(32, cc_path_next_backtrack_seeded),
        "RangeOffByOne" => crabcheck_qc::quickcheck(cc_range_off_by_one),
        "RrbDebugPop" => crabcheck_qc::quickcheck(cc_rrb_debug_pop),
        // RrbDensityCheck's canonical shape allocates a 262k-element Vector
        // per call. With the widened generator most seeds build smaller
        // Vectors so this is cheaper on average; cap to 64 so a few seeds
        // still land on the bug-trigger shape.
        "RrbDensityCheck" => cc_run_bounded(64, cc_rrb_density_check_seeded),
        "PtrEqPrecedence" => crabcheck_qc::quickcheck(cc_ptr_eq_precedence),
        // Default Vec<i32> generator almost never hits the len>=32 band; use seeded.
        "EqSingleChunk" => cc_run_bounded(256, cc_eq_single_chunk_seeded),
        _ => {
            return (
                Err(format!("Unknown property for crabcheck: {property}")),
                Metrics::default(),
            )
        }
    };
    let elapsed_us = t0.elapsed().as_micros();
    let inputs = CC_COUNTER.load(Ordering::Relaxed);
    let metrics = Metrics { inputs, elapsed_us };
    let status = match result.status {
        crabcheck_qc::ResultStatus::Finished => Ok(()),
        crabcheck_qc::ResultStatus::Failed { arguments } => Err(format!(
            "crabcheck failed with counterexample: ({})",
            arguments.join(" ")
        )),
        crabcheck_qc::ResultStatus::TimedOut => Err("crabcheck timed out".to_string()),
        crabcheck_qc::ResultStatus::GaveUp => Err(format!(
            "crabcheck gave up: passed={}, discarded={}",
            result.passed, result.discarded
        )),
        crabcheck_qc::ResultStatus::Aborted { error } => {
            Err(format!("crabcheck aborted: {error}"))
        }
    };
    (status, metrics)
}

// ---------- hegel ----------

static HG_COUNTER: AtomicU64 = AtomicU64::new(0);

fn hegel_settings() -> HegelSettings {
    HegelSettings::new().test_cases(64).seed(Some(0xF100_A7))
}

fn run_hegel_property(property: &str) -> Outcome {
    if property == "All" {
        return run_all(run_hegel_property);
    }
    HG_COUNTER.store(0, Ordering::Relaxed);
    let t0 = Instant::now();
    let settings = hegel_settings();
    let run_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| match property {
        "PathNextBacktrack" => {
            Hegel::new(|tc: TestCase| {
                HG_COUNTER.fetch_add(1, Ordering::Relaxed);
                let n = tc.draw(hgen::integers::<u16>()) as u32;
                let lo = tc.draw(hgen::integers::<u16>()) as u32;
                let span = tc.draw(hgen::integers::<u16>()) as u32;
                if let PropertyResult::Fail(m) = property_path_next_backtrack(n, lo, span) {
                    panic!("{}", m);
                }
            })
            .settings(settings.clone())
            .run();
        }
        "RangeOffByOne" => {
            Hegel::new(|tc: TestCase| {
                HG_COUNTER.fetch_add(1, Ordering::Relaxed);
                let n = tc.draw(hgen::integers::<u16>()) as u32;
                let hi = tc.draw(hgen::integers::<u16>()) as u32;
                if let PropertyResult::Fail(m) = property_range_off_by_one(n, hi) {
                    panic!("{}", m);
                }
            })
            .settings(settings.clone())
            .run();
        }
        "RrbDebugPop" => {
            Hegel::new(|tc: TestCase| {
                HG_COUNTER.fetch_add(1, Ordering::Relaxed);
                let n = tc.draw(hgen::integers::<u16>()) as u32;
                if let PropertyResult::Fail(m) = property_rrb_debug_pop(n) {
                    panic!("{}", m);
                }
            })
            .settings(settings.clone())
            .run();
        }
        "RrbDensityCheck" => {
            // Widened: hegel draws three independent generator dimensions
            // (n_removed, size_factor, op_mode). Only a slice of the joint
            // space lands on the canonical bug-triggering build sequence;
            // the rest produce smaller Vectors or alternate op sequences.
            // Allow more test cases than before since most are cheap.
            Hegel::new(|tc: TestCase| {
                HG_COUNTER.fetch_add(1, Ordering::Relaxed);
                let n = tc.draw(hgen::integers::<u8>()) as u32;
                let sf = (tc.draw(hgen::integers::<u8>()) as u32) % 8;
                let om = (tc.draw(hgen::integers::<u8>()) as u32) % 6;
                if let PropertyResult::Fail(m) = property_rrb_density_check_shape(n, sf, om) {
                    panic!("{}", m);
                }
            })
            .settings(HegelSettings::new().test_cases(24).seed(Some(0xF100_A7)))
            .run();
        }
        "PtrEqPrecedence" => {
            Hegel::new(|tc: TestCase| {
                HG_COUNTER.fetch_add(1, Ordering::Relaxed);
                let n = tc.draw(hgen::integers::<u16>()) as u32;
                let slot = tc.draw(hgen::integers::<u16>()) as u32;
                if let PropertyResult::Fail(m) = property_ptr_eq_precedence(n, slot) {
                    panic!("{}", m);
                }
            })
            .settings(settings.clone())
            .run();
        }
        "EqSingleChunk" => {
            Hegel::new(|tc: TestCase| {
                HG_COUNTER.fetch_add(1, Ordering::Relaxed);
                let len = (tc.draw(hgen::integers::<u8>()) as usize % 29) + 32;
                let xs: Vec<i32> = (0..len).map(|_| tc.draw(hgen::integers::<i32>())).collect();
                if let PropertyResult::Fail(m) = property_eq_single_chunk(xs) {
                    panic!("{}", m);
                }
            })
            .settings(settings.clone())
            .run();
        }
        _ => panic!("{}", format!("__unknown_property:{}", property)),
    }));
    let elapsed_us = t0.elapsed().as_micros();
    let inputs = HG_COUNTER.load(Ordering::Relaxed);
    let metrics = Metrics { inputs, elapsed_us };
    let status = match run_result {
        Ok(()) => Ok(()),
        Err(e) => {
            let msg = if let Some(s) = e.downcast_ref::<String>() {
                s.clone()
            } else if let Some(s) = e.downcast_ref::<&str>() {
                s.to_string()
            } else {
                "hegel panicked with non-string payload".to_string()
            };
            if let Some(rest) = msg.strip_prefix("__unknown_property:") {
                return (
                    Err(format!("Unknown property for hegel: {rest}")),
                    Metrics::default(),
                );
            }
            Err(format!("hegel found counterexample: {msg}"))
        }
    };
    (status, metrics)
}

// ---------- dispatch ----------

fn run(tool: &str, property: &str) -> Outcome {
    match tool {
        "etna" => run_etna_property(property),
        "proptest" => run_proptest_property(property),
        "quickcheck" => run_quickcheck_property(property),
        "crabcheck" => run_crabcheck_property(property),
        "hegel" => run_hegel_property(property),
        _ => (Err(format!("Unknown tool: {tool}")), Metrics::default()),
    }
}

fn json_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn emit_json(
    tool: &str,
    property: &str,
    status: &str,
    metrics: Metrics,
    counterexample: Option<&str>,
    error: Option<&str>,
) {
    let cex = counterexample.map_or("null".to_string(), json_str);
    let err = error.map_or("null".to_string(), json_str);
    println!(
        "{{\"status\":{},\"tests\":{},\"discards\":0,\"time\":{},\"counterexample\":{},\"error\":{},\"tool\":{},\"property\":{}}}",
        json_str(status),
        metrics.inputs,
        json_str(&format!("{}us", metrics.elapsed_us)),
        cex,
        err,
        json_str(tool),
        json_str(property),
    );
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <tool> <property>", args[0]);
        eprintln!("Tools: etna | proptest | quickcheck | crabcheck | hegel");
        eprintln!(
            "Properties: PathNextBacktrack | RangeOffByOne | RrbDebugPop | RrbDensityCheck | PtrEqPrecedence | EqSingleChunk | All"
        );
        std::process::exit(2);
    }
    let (tool, property) = (args[1].as_str(), args[2].as_str());

    // Silence library-under-test panic noise (frameworks catch panics internally
    // but the default hook still prints "thread 'main' panicked at ..." to
    // stderr).
    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run(tool, property)));
    std::panic::set_hook(previous_hook);

    let (result, metrics) = match caught {
        Ok(outcome) => outcome,
        Err(payload) => {
            let msg = if let Some(s) = payload.downcast_ref::<String>() {
                s.clone()
            } else if let Some(s) = payload.downcast_ref::<&str>() {
                s.to_string()
            } else {
                "panic with non-string payload".to_string()
            };
            emit_json(
                tool,
                property,
                "aborted",
                Metrics::default(),
                None,
                Some(&format!("adapter panic: {msg}")),
            );
            return;
        }
    };

    match result {
        Ok(()) => emit_json(tool, property, "passed", metrics, None, None),
        Err(msg) => emit_json(tool, property, "failed", metrics, Some(&msg), None),
    }
}
