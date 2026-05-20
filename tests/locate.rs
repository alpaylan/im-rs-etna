//! Fault-localization integration tests for im-rs.
//!
//! Each test calls `crabcheck::quickcheck_with_locate!` on one property from
//! the workload's `etna-faultloc.rs` dispatch, then prints the report and
//! emits an `@@LOCATE@@ <json>` line on stdout. Tests never panic.

use im::etna::{
    property_eq_single_chunk, property_path_next_backtrack, property_ptr_eq_precedence,
    property_range_off_by_one, property_rrb_debug_pop, property_rrb_density_check_shape,
    PropertyResult,
};

fn to_opt(r: PropertyResult) -> Option<bool> {
    match r {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

// ---- property wrappers (mirroring src/bin/etna-faultloc.rs) ----

fn prop_path_next_backtrack(seed: usize) -> Option<bool> {
    let s = seed as u32;
    to_opt(property_path_next_backtrack(
        s,
        s.wrapping_mul(13),
        s.wrapping_mul(11).wrapping_add(1),
    ))
}

fn prop_range_off_by_one((n, hi): (usize, usize)) -> Option<bool> {
    to_opt(property_range_off_by_one(
        (n as u32).wrapping_mul(17),
        (hi as u32).wrapping_mul(13),
    ))
}

fn prop_rrb_debug_pop(n: usize) -> Option<bool> {
    to_opt(property_rrb_debug_pop((n as u32).wrapping_mul(31)))
}

fn prop_rrb_density_check(seed: usize) -> Option<bool> {
    // Mirror src/bin/etna-faultloc.rs's widened dispatcher.
    let seed = seed as u32;
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
    to_opt(property_rrb_density_check_shape(n_removed, size_factor, op_mode))
}

fn prop_ptr_eq_precedence((n, slot): (usize, usize)) -> Option<bool> {
    to_opt(property_ptr_eq_precedence(
        (n as u32).wrapping_mul(17),
        (slot as u32).wrapping_mul(7),
    ))
}

fn prop_eq_single_chunk(seed: usize) -> Option<bool> {
    let seed = seed as u32;
    let len = (seed as usize % 29) + 32;
    let mut xs = Vec::with_capacity(len);
    let mut x: u64 = seed as u64;
    for _ in 0..len {
        xs.push(x as i32);
        x = x.wrapping_mul(6364136223846793005).wrapping_add(1);
    }
    to_opt(property_eq_single_chunk(xs))
}

// ---- emit helper ----

fn emit_locate_json(r: &crabcheck::profiling::LocateResult) {
    use crabcheck::quickcheck::ResultStatus;
    let status = match &r.run.status {
        ResultStatus::Failed { .. } => "Failed",
        ResultStatus::Finished => "Finished",
        ResultStatus::GaveUp => "GaveUp",
        ResultStatus::TimedOut => "TimedOut",
        ResultStatus::Aborted { .. } => "Aborted",
    };
    let top = if let Some(s) = r.top() {
        serde_json::json!({
            "rank": s.rank,
            "file": s.region.file,
            "function": s.region.function,
            "start_line": s.region.start_line,
            "end_line": s.region.end_line,
            "ochiai": s.region.suspiciousness.ochiai,
            "delta": s.region.delta,
            "panic_overlap": s.panic_overlap,
            "confidence": format!("{}", s.confidence),
            "confidence_rule": s.confidence_rule,
        })
    } else {
        serde_json::Value::Null
    };
    let top_5: Vec<_> = r
        .suspects
        .iter()
        .take(5)
        .map(|s| {
            serde_json::json!({
                "rank": s.rank,
                "file": s.region.file,
                "function": s.region.function,
                "start_line": s.region.start_line,
                "end_line": s.region.end_line,
                "confidence": format!("{}", s.confidence),
                "confidence_rule": s.confidence_rule,
                "panic_overlap": s.panic_overlap,
            })
        })
        .collect();
    let diags: Vec<_> = r.diagnostics.iter().map(|d| d.tag()).collect();
    let out = serde_json::json!({
        "status": status,
        "passed": r.run.passed,
        "discarded": r.run.discarded,
        "n_panics": r.n_panics,
        "n_suspects": r.suspects.len(),
        "top": top,
        "top_5": top_5,
        "diagnostics": diags,
    });
    println!("@@LOCATE@@ {}", out);
}

// ---- tests ----

#[test]
fn locate_path_next_backtrack() {
    let report = crabcheck::quickcheck_with_locate!(prop_path_next_backtrack, "im");
    eprintln!("{report}");
    emit_locate_json(&report);
}

#[test]
fn locate_range_off_by_one() {
    let report = crabcheck::quickcheck_with_locate!(prop_range_off_by_one, "im");
    eprintln!("{report}");
    emit_locate_json(&report);
}

#[test]
fn locate_rrb_debug_pop() {
    let report = crabcheck::quickcheck_with_locate!(prop_rrb_debug_pop, "im");
    eprintln!("{report}");
    emit_locate_json(&report);
}

#[test]
fn locate_rrb_density_check() {
    let report = crabcheck::quickcheck_with_locate!(prop_rrb_density_check, "im");
    eprintln!("{report}");
    emit_locate_json(&report);
}

#[test]
fn locate_ptr_eq_precedence() {
    let report = crabcheck::quickcheck_with_locate!(prop_ptr_eq_precedence, "im");
    eprintln!("{report}");
    emit_locate_json(&report);
}

#[test]
fn locate_eq_single_chunk() {
    let report = crabcheck::quickcheck_with_locate!(prop_eq_single_chunk, "im");
    eprintln!("{report}");
    emit_locate_json(&report);
}
