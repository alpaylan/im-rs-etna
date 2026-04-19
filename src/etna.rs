//! ETNA framework-neutral property functions for im-rs.
//!
//! Each `property_<name>` is a pure function taking concrete, owned inputs and
//! returning `PropertyResult`. Framework adapters (proptest/quickcheck/crabcheck/hegel)
//! in `src/bin/etna.rs` and deterministic witness tests in `tests/etna_witnesses.rs`
//! both call these functions directly — there is no re-implementation of the
//! invariant inside any adapter.

#![allow(missing_docs)]

use crate::{OrdMap, Vector};
use std::panic::{catch_unwind, AssertUnwindSafe};

pub enum PropertyResult {
    Pass,
    Fail(String),
    Discard,
}

/// Range iteration over an `OrdMap` returns exactly the keys in `[lo, hi)`,
/// regardless of how many internal B-tree boundaries the range crosses.
///
/// Detects `path_next_backtrack` (fix commit `41d99725`): when `Node::path_next`
/// reaches the end of a node without a child to recurse into, the buggy version
/// returns an empty path instead of backtracking up to the ancestor, causing the
/// range iterator to stop early.
pub fn property_path_next_backtrack(size: u32, lo: u32, span: u32) -> PropertyResult {
    if size == 0 || span == 0 {
        return PropertyResult::Discard;
    }
    let size = (size % 4096) as i32 + 2;
    let lo = (lo as i32) % size;
    let hi_excl = lo.saturating_add((span as i32) % size);
    if hi_excl <= lo || hi_excl > size {
        return PropertyResult::Discard;
    }
    let map: OrdMap<i32, i32> = (0..size).map(|i| (i, i)).collect();
    let collected: Vec<i32> = map.range(lo..hi_excl).map(|(k, _)| *k).collect();
    let expected: Vec<i32> = (lo..hi_excl).collect();
    if collected == expected {
        PropertyResult::Pass
    } else {
        PropertyResult::Fail(format!(
            "range({lo}..{hi_excl}) over size {size} returned {} keys, expected {}",
            collected.len(),
            expected.len()
        ))
    }
}

/// Upper-bound of an `OrdMap::range(..=hi)` or `range(..hi)` must exclude keys
/// outside the requested range, even when `hi` is not itself a key.
///
/// Detects `range_off_by_one` (fix `3f4e01a4`, issue #143): `Node::path_prev`
/// in the buggy version uses `index` instead of `index - 1` when the search
/// lands at an absent key, causing the iterator to include the key immediately
/// past the requested upper bound.
pub fn property_range_off_by_one(size: u32, hi_factor: u32) -> PropertyResult {
    if size == 0 {
        return PropertyResult::Discard;
    }
    let size = (size % 4096) as i32 + 2;
    let map: OrdMap<i32, i32> = (0..size).map(|i| (i * 2, i)).collect();
    let hi = ((hi_factor % (size as u32 * 2)) as i32) | 1; // force odd so it is NOT a key
    let collected: Vec<i32> = map.range(..hi).map(|(k, _)| *k).collect();
    if let Some(&last) = collected.last() {
        if last >= hi {
            return PropertyResult::Fail(format!(
                "range(..{hi}) leaked key {last} past upper bound"
            ));
        }
    }
    PropertyResult::Pass
}

/// Sequences of `Vector::pop_front` in release mode must not crash, regardless
/// of whether intermediate steps update an internal size table.
///
/// Detects `rrb_debug_pop` (fix `1209e823`, issue #72): the buggy code places
/// the size-table mutating `pop_front` inside `debug_assert_eq!`, whose body
/// is compiled out of release mode. The first few `pop_front()` calls then
/// leave the size table stale, and subsequent operations panic with an index
/// out-of-bounds.
pub fn property_rrb_debug_pop(n: u32) -> PropertyResult {
    let n = (n % 2048).saturating_add(256) as usize;
    let build_and_pop = || {
        let mut v: Vector<i32> = (0..(n as i32 * 2)).collect();
        for _ in 0..n {
            v.pop_front();
        }
        v.len()
    };
    match catch_unwind(AssertUnwindSafe(build_and_pop)) {
        Ok(len) if len == n => PropertyResult::Pass,
        Ok(len) => PropertyResult::Fail(format!("expected length {n} after pops, got {len}")),
        Err(_) => PropertyResult::Fail("Vector::pop_front panicked in release mode".into()),
    }
}

/// After an `append` across a large-enough RRB tree that exercises level >= 2
/// sparse children, random-access indexing must round-trip against iteration.
///
/// Detects `rrb_density_check` (fix `cb431a6`, issue #55): the buggy `parent`
/// uses `is_full` instead of `is_completely_dense`, wrongly opting for uniform
/// `Size::Size` accounting when a nominally-full internal node has sparse
/// grandchildren. Index lookups then disagree with linear iteration on large
/// append-split sequences.
pub fn property_rrb_density_check(n_removed: u32) -> PropertyResult {
    // These constants are chosen to force a level-2 RRB tree with sparse leaves.
    // 64^3 = 262_144; +640 guarantees a full top-level node plus leftovers.
    let total = 64 * 64 * 64 + 640;
    let keep_every = (n_removed % 16 + 1) as usize;
    let mut v: Vector<i32> = (0..total as i32).collect();
    for i in (0..200).rev() {
        let idx = (i * (total / 200)) + 7 + keep_every;
        if idx < v.len() {
            v.remove(idx);
        }
    }
    let split_at = v.len() / 3;
    let (left, right) = v.split_at(split_at);
    let mut result = left;
    result.append(right);
    let expected: Vec<i32> = result.iter().cloned().collect();
    for (i, want) in expected.iter().enumerate() {
        match result.get(i) {
            Some(got) if got == want => {}
            Some(got) => {
                return PropertyResult::Fail(format!(
                    "index {i} mismatch: iter says {want}, get says {got}"
                ));
            }
            None => {
                return PropertyResult::Fail(format!("index {i} missing; len = {}", result.len()));
            }
        }
    }
    PropertyResult::Pass
}

/// `Vector::ptr_eq` must return `false` once two sibling vectors have diverged
/// in any of their outer or inner chunks, even if they still share a middle
/// tree pointer.
///
/// Detects `ptr_eq_precedence` (fix `f744912`, issue #131): without parentheses
/// around the final `||`, the `&&` conjunction short-circuits to the ptr-eq
/// check on the middles, silently ignoring any diverging chunk.
pub fn property_ptr_eq_precedence(size: u32, slot: u32) -> PropertyResult {
    let size = (size % 1024 + 128) as i32;
    let slot = (slot % size as u32) as usize;
    let mut a: Vector<i32> = (0..size).collect();
    let b = a.clone();
    a.set(slot, -999);
    // After a single `set`, `a` has diverged at index `slot`. ptr_eq must say false.
    if a.ptr_eq(&b) {
        return PropertyResult::Fail(format!(
            "ptr_eq returned true after set(slot={slot}) diverged the vectors"
        ));
    }
    PropertyResult::Pass
}

/// `PartialEq` on `Vector` must report equality by element sequence regardless
/// of internal layout (Single vs Full, shared chunks or not).
///
/// Detects `eq_single_chunk` (fix `005193a`): the buggy `Single==Single` arm
/// returned `cmp_chunk(left, right)` without the elementwise fallback, so two
/// structurally distinct single-chunk vectors with the same elements compared
/// unequal.
pub fn property_eq_single_chunk(xs: Vec<i32>) -> PropertyResult {
    if xs.is_empty() {
        return PropertyResult::Discard;
    }
    // Build `a` directly from xs; build `b` by prepending then popping a sentinel
    // so the internal chunk layout diverges while element contents match.
    let a: Vector<i32> = xs.iter().cloned().collect();
    let mut b: Vector<i32> = Vector::new();
    b.push_front(i32::MIN);
    for x in xs.iter().cloned() {
        b.push_back(x);
    }
    b.pop_front();
    if a == b {
        PropertyResult::Pass
    } else {
        PropertyResult::Fail(format!(
            "vectors with identical elements compared unequal (len={} vs {})",
            a.len(),
            b.len()
        ))
    }
}
