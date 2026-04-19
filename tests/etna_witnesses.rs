// Deterministic witness tests for ETNA variants.
//
// Each `witness_<name>_case_<tag>` passes on the base commit and fails under
// the corresponding `etna/<variant>` branch (where the historical bug has been
// re-injected via a patch). Witnesses call `property_<name>` directly with
// frozen inputs; they do not touch framework machinery (no proptest, no
// quickcheck, no RNG, no clocks).

use im::etna::{
    property_eq_single_chunk, property_path_next_backtrack, property_ptr_eq_precedence,
    property_range_off_by_one, property_rrb_debug_pop, property_rrb_density_check, PropertyResult,
};

fn expect_pass(r: PropertyResult, what: &str) {
    match r {
        PropertyResult::Pass => {}
        PropertyResult::Fail(m) => panic!("{}: property failed: {}", what, m),
        PropertyResult::Discard => panic!("{}: unexpected discard", what),
    }
}

// Variant: path_next_backtrack_41d99725_1
//
// `Node::path_next` without the backtrack loop stops range iteration early
// when a search terminates mid-tree. A 1000-element OrdMap with range 100..200
// exercises the missing backtrack.
#[test]
fn witness_path_next_backtrack_case_ordmap_1000() {
    // lo=0, span=7 sweeps absent keys adjacent to each leaf boundary in the
    // first few nodes. The buggy `path_next` truncates the iterator when the
    // descent lands past the last key of an interior leaf, so the reference
    // BTreeMap produces a larger count.
    expect_pass(
        property_path_next_backtrack(0, 0, 7),
        "OrdMap range near leaf boundaries",
    );
}

// Variant: range_off_by_one_3f4e01a4_1
//
// `Node::path_prev` using `index` instead of `index - 1` when the upper bound
// falls on an absent key lets the iterator leak one key past the requested
// upper bound.
#[test]
fn witness_range_off_by_one_case_odd_upper_bound() {
    // Keys are {0, 2, 4, ..., 1998}. 501 is absent; buggy path_prev leaks 502.
    expect_pass(
        property_range_off_by_one(1000, 501),
        "OrdMap range(..501) on even-keyed map",
    );
}

// Variant: rrb_debug_pop_1209e823_1
//
// A side-effectful `size_table.pop_front()` inside `debug_assert_eq!` compiles
// out in release mode, leaving the size table stale after every left-side pop.
// Any large enough sequence of `pop_front` on a Vector triggers the downstream
// panic.
#[test]
fn witness_rrb_debug_pop_case_release_pop_front() {
    // A Vector of ~3000 elements forces a non-trivial middle RRB tree; after
    // many pop_front calls the internal size table has transitioned from
    // Size::Size to Size::Table. Under the bug the Left/Table arm's
    // `debug_assert_eq!` no longer mutates the table in release builds, so
    // random-access `get(i)` disagrees with iteration once the corrupt
    // size-table entry is indexed.
    expect_pass(
        property_rrb_debug_pop(1000),
        "pop_front consistency across RRB middle tree",
    );
}

// Variant: rrb_density_check_cb431a6_1
//
// `Node::parent` using `is_full()` rather than `is_completely_dense(level - 1)`
// mislabels level-2 nodes with sparse grandchildren as uniform-sized, and the
// resulting Size::Size accounting returns wrong indices.
#[test]
fn witness_rrb_density_check_case_level2_rrb() {
    expect_pass(
        property_rrb_density_check(0),
        "262k-element RRB with sparse leaves",
    );
}

// Variant: ptr_eq_precedence_f744912_1
//
// Without parentheses, `&&` binds tighter than `||`, so `ptr_eq` effectively
// ignores differences in outer chunks once middle pointers are shared.
#[test]
fn witness_ptr_eq_precedence_case_diverged_outer() {
    expect_pass(
        property_ptr_eq_precedence(200, 0),
        "ptr_eq after set on shared-middle vector",
    );
}

// Variant: eq_single_chunk_005193a_1
//
// The `Single==Single` fast path returns `cmp_chunk(left, right)` directly in
// the buggy version, skipping the elementwise fallback. Two vectors with the
// same elements but distinct internal chunk identities then compare unequal.
#[test]
fn witness_eq_single_chunk_case_small_vec() {
    // A 40-element i32 vector lives in the `Single` chunk form, where the
    // specialized `PartialEq` arm fires. Below the inline threshold both
    // sides fall through to the correct `iter().eq()` fallback.
    expect_pass(
        property_eq_single_chunk((0..40).collect()),
        "Single/Single eq via different build paths",
    );
}
