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
pub fn property_path_next_backtrack(size_hint: u32, lo_hint: u32, span_hint: u32) -> PropertyResult {
    // Mirror the test added in the original fix (41d99725): build an OrdMap
    // large enough to form a multi-level B-tree (NODE_SIZE^2 * 5 = 20480
    // entries), and sweep `range(..)` queries over NODE_SIZE*5 different
    // lower-bound values near the tree's left edge and right edge. For many
    // of these bounds `lo` lands on an absent key whose descent terminates
    // at a leaf with no `children[index]` and no `keys.get(index)` — the
    // exact shape of the buggy `path_next` `None` arm. The buggy version
    // returns an empty path and truncates the iterator; the fix backtracks
    // up the ancestors and keeps iterating.
    use std::collections::BTreeMap;
    const NODE_SIZE: usize = 64;
    let n = NODE_SIZE * NODE_SIZE * 5;
    let span = (span_hint % 256 + 1) as usize;
    let lo_start = (lo_hint as usize) % (NODE_SIZE * 5);
    let _ = size_hint;
    let data = (1..n).filter(|i| i % 2 == 0).map(|i| (i, ()));
    let bmap: BTreeMap<usize, ()> = data.clone().collect();
    let omap: OrdMap<usize, ()> = data.collect();
    // Two sweep windows: near the start of the key space and near the end,
    // covering both forward and backward edge leaves.
    for &base in &[lo_start, n.saturating_sub(NODE_SIZE * 5) + lo_start] {
        for step in 0..(NODE_SIZE * 5) {
            let lo = base + step;
            let hi = lo + span;
            let got = omap.range(lo..hi).count();
            let want = bmap.range(lo..hi).count();
            if got != want {
                return PropertyResult::Fail(format!(
                    "range({lo}..{hi}): got {got} keys, expected {want}"
                ));
            }
            let got_upper = omap.range(..lo).count();
            let want_upper = bmap.range(..lo).count();
            if got_upper != want_upper {
                return PropertyResult::Fail(format!(
                    "range(..{lo}): got {got_upper} keys, expected {want_upper}"
                ));
            }
        }
    }
    PropertyResult::Pass
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
    // Build a Vector with a non-trivial middle RRB tree, then pop_front past
    // the outer/inner buffers so `Size::pop` is exercised on internal nodes.
    // With the fix, `pop_front`'s side effect on the size table is preserved
    // in release mode. With the bug, `size_table.pop_front()` lives inside
    // `debug_assert_eq!` and is elided in release, leaving the size table
    // stale. Subsequent random-access indexing reads a corrupt Size::Table
    // entry and panics with index-out-of-bounds.
    let n = (n % 2048).saturating_add(512) as usize;
    let run = || -> Result<(), String> {
        let mut v: Vector<i32> = (0..(n as i32 * 3)).collect();
        for _ in 0..n {
            v.pop_front();
        }
        let by_iter: Vec<i32> = v.iter().cloned().collect();
        if by_iter.len() != v.len() {
            return Err(format!(
                "iter length {} disagrees with Vector::len {}",
                by_iter.len(),
                v.len()
            ));
        }
        for (i, expected) in by_iter.iter().enumerate() {
            match v.get(i) {
                Some(got) if got == expected => {}
                Some(got) => {
                    return Err(format!("index {i}: iter says {expected}, get says {got}"));
                }
                None => {
                    return Err(format!(
                        "index {i}: iter has value but get returned None (len={})",
                        v.len()
                    ));
                }
            }
        }
        Ok(())
    };
    match catch_unwind(AssertUnwindSafe(run)) {
        Ok(Ok(())) => PropertyResult::Pass,
        Ok(Err(m)) => PropertyResult::Fail(m),
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
    let run = || -> Result<(), String> {
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
                    return Err(format!(
                        "index {i} mismatch: iter says {want}, get says {got}"
                    ));
                }
                None => {
                    return Err(format!("index {i} missing; len = {}", result.len()));
                }
            }
        }
        Ok(())
    };
    match catch_unwind(AssertUnwindSafe(run)) {
        Ok(Ok(())) => PropertyResult::Pass,
        Ok(Err(m)) => PropertyResult::Fail(m),
        Err(_) => PropertyResult::Fail("Vector op panicked under buggy RRB density".into()),
    }
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
    // Need enough elements so the Vector is in the `Single` chunk form rather
    // than the small `Inline` form — below the inline threshold both sides hit
    // the `_ => iter().eq()` fallback which is correct for either version.
    // CHUNK_SIZE is 64; 32..=60 safely forces `Single` without crossing into
    // `Full` RRB territory.
    if xs.len() < 32 || xs.len() > 60 {
        return PropertyResult::Discard;
    }
    // Two independent `collect` calls produce two `Single` vectors with the same
    // elements but distinct chunk identities. Under the fix they compare equal
    // (fallthrough to `iter().eq`). Under the bug they compare unequal because
    // `cmp_chunk` is a pointer test that returns false.
    let a: Vector<i32> = xs.iter().cloned().collect();
    let b: Vector<i32> = xs.iter().cloned().collect();
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
