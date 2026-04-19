# im-rs — ETNA Tasks

Total tasks: 24

ETNA tasks are **mutation/property/witness triplets**. Each row below is one runnable task. The `<PropertyKey>` token in the command column uses the PascalCase key recognised by `src/bin/etna.rs`; passing `All` runs every property for the named framework in a single invocation.

## Property keys

| Property | PropertyKey |
|----------|-------------|
| `property_path_next_backtrack` | `PathNextBacktrack` |
| `property_range_off_by_one` | `RangeOffByOne` |
| `property_rrb_debug_pop` | `RrbDebugPop` |
| `property_rrb_density_check` | `RrbDensityCheck` |
| `property_ptr_eq_precedence` | `PtrEqPrecedence` |
| `property_eq_single_chunk` | `EqSingleChunk` |

## Task Index

| Task | Variant | Framework | Property | Witness | Command |
|------|---------|-----------|----------|---------|---------|
| 001 | `path_next_backtrack_41d99725_1` | proptest    | `property_path_next_backtrack` | `witness_path_next_backtrack_case_ordmap_1000` | `cargo run --release --bin etna -- proptest PathNextBacktrack` |
| 002 | `path_next_backtrack_41d99725_1` | quickcheck  | `property_path_next_backtrack` | `witness_path_next_backtrack_case_ordmap_1000` | `cargo run --release --bin etna -- quickcheck PathNextBacktrack` |
| 003 | `path_next_backtrack_41d99725_1` | crabcheck   | `property_path_next_backtrack` | `witness_path_next_backtrack_case_ordmap_1000` | `cargo run --release --bin etna -- crabcheck PathNextBacktrack` |
| 004 | `path_next_backtrack_41d99725_1` | hegel       | `property_path_next_backtrack` | `witness_path_next_backtrack_case_ordmap_1000` | `cargo run --release --bin etna -- hegel PathNextBacktrack` |
| 005 | `range_off_by_one_3f4e01a4_1`     | proptest    | `property_range_off_by_one`     | `witness_range_off_by_one_case_odd_upper_bound` | `cargo run --release --bin etna -- proptest RangeOffByOne` |
| 006 | `range_off_by_one_3f4e01a4_1`     | quickcheck  | `property_range_off_by_one`     | `witness_range_off_by_one_case_odd_upper_bound` | `cargo run --release --bin etna -- quickcheck RangeOffByOne` |
| 007 | `range_off_by_one_3f4e01a4_1`     | crabcheck   | `property_range_off_by_one`     | `witness_range_off_by_one_case_odd_upper_bound` | `cargo run --release --bin etna -- crabcheck RangeOffByOne` |
| 008 | `range_off_by_one_3f4e01a4_1`     | hegel       | `property_range_off_by_one`     | `witness_range_off_by_one_case_odd_upper_bound` | `cargo run --release --bin etna -- hegel RangeOffByOne` |
| 009 | `rrb_debug_pop_1209e823_1`         | proptest    | `property_rrb_debug_pop`         | `witness_rrb_debug_pop_case_release_pop_front` | `cargo run --release --bin etna -- proptest RrbDebugPop` |
| 010 | `rrb_debug_pop_1209e823_1`         | quickcheck  | `property_rrb_debug_pop`         | `witness_rrb_debug_pop_case_release_pop_front` | `cargo run --release --bin etna -- quickcheck RrbDebugPop` |
| 011 | `rrb_debug_pop_1209e823_1`         | crabcheck   | `property_rrb_debug_pop`         | `witness_rrb_debug_pop_case_release_pop_front` | `cargo run --release --bin etna -- crabcheck RrbDebugPop` |
| 012 | `rrb_debug_pop_1209e823_1`         | hegel       | `property_rrb_debug_pop`         | `witness_rrb_debug_pop_case_release_pop_front` | `cargo run --release --bin etna -- hegel RrbDebugPop` |
| 013 | `rrb_density_check_cb431a6_1`      | proptest    | `property_rrb_density_check`     | `witness_rrb_density_check_case_level2_rrb`    | `cargo run --release --bin etna -- proptest RrbDensityCheck` |
| 014 | `rrb_density_check_cb431a6_1`      | quickcheck  | `property_rrb_density_check`     | `witness_rrb_density_check_case_level2_rrb`    | `cargo run --release --bin etna -- quickcheck RrbDensityCheck` |
| 015 | `rrb_density_check_cb431a6_1`      | crabcheck   | `property_rrb_density_check`     | `witness_rrb_density_check_case_level2_rrb`    | `cargo run --release --bin etna -- crabcheck RrbDensityCheck` |
| 016 | `rrb_density_check_cb431a6_1`      | hegel       | `property_rrb_density_check`     | `witness_rrb_density_check_case_level2_rrb`    | `cargo run --release --bin etna -- hegel RrbDensityCheck` |
| 017 | `ptr_eq_precedence_f744912_1`      | proptest    | `property_ptr_eq_precedence`     | `witness_ptr_eq_precedence_case_diverged_outer`| `cargo run --release --bin etna -- proptest PtrEqPrecedence` |
| 018 | `ptr_eq_precedence_f744912_1`      | quickcheck  | `property_ptr_eq_precedence`     | `witness_ptr_eq_precedence_case_diverged_outer`| `cargo run --release --bin etna -- quickcheck PtrEqPrecedence` |
| 019 | `ptr_eq_precedence_f744912_1`      | crabcheck   | `property_ptr_eq_precedence`     | `witness_ptr_eq_precedence_case_diverged_outer`| `cargo run --release --bin etna -- crabcheck PtrEqPrecedence` |
| 020 | `ptr_eq_precedence_f744912_1`      | hegel       | `property_ptr_eq_precedence`     | `witness_ptr_eq_precedence_case_diverged_outer`| `cargo run --release --bin etna -- hegel PtrEqPrecedence` |
| 021 | `eq_single_chunk_005193a_1`        | proptest    | `property_eq_single_chunk`       | `witness_eq_single_chunk_case_small_vec`       | `cargo run --release --bin etna -- proptest EqSingleChunk` |
| 022 | `eq_single_chunk_005193a_1`        | quickcheck  | `property_eq_single_chunk`       | `witness_eq_single_chunk_case_small_vec`       | `cargo run --release --bin etna -- quickcheck EqSingleChunk` |
| 023 | `eq_single_chunk_005193a_1`        | crabcheck   | `property_eq_single_chunk`       | `witness_eq_single_chunk_case_small_vec`       | `cargo run --release --bin etna -- crabcheck EqSingleChunk` |
| 024 | `eq_single_chunk_005193a_1`        | hegel       | `property_eq_single_chunk`       | `witness_eq_single_chunk_case_small_vec`       | `cargo run --release --bin etna -- hegel EqSingleChunk` |

## Witness catalog

Each witness is a deterministic concrete test. Base build: passes. Variant-active build: fails.

- `witness_path_next_backtrack_case_ordmap_1000` — `property_path_next_backtrack(0, 0, 7)` → `Pass` (builds 20480-entry OrdMap of even keys, sweeps NODE_SIZE*5 lower-bound offsets at both ends of the tree; the buggy `None` arm truncates the iterator when descent lands past the last key of a leaf).
- `witness_range_off_by_one_case_odd_upper_bound` — `property_range_off_by_one(1000, 501)` → `Pass` (keys {0, 2, …, 1998}; query `range(..501)` with 501 absent; buggy `path_prev` leaks 502).
- `witness_rrb_debug_pop_case_release_pop_front` — `property_rrb_debug_pop(1000)` → `Pass` (builds 4536-element Vector, pops 1512 from front, indexes remainder; buggy `debug_assert_eq!`-wrapped pop leaves a stale size table that indexing reads into out-of-bounds).
- `witness_rrb_density_check_case_level2_rrb` — `property_rrb_density_check(0)` → `Pass` (262784-element Vector, 200 removals near leaf boundaries, split+append; buggy `parent` keeps uniform `Size::Size` and index lookups disagree with iteration).
- `witness_ptr_eq_precedence_case_diverged_outer` — `property_ptr_eq_precedence(200, 0)` → `Pass` (328-element Vector cloned then `set(0, -999)`; buggy `ptr_eq` returns `true` because the shared middle short-circuits the final `||`).
- `witness_eq_single_chunk_case_small_vec` — `property_eq_single_chunk((0..40).collect())` → `Pass` (two 40-element `Single` vectors built via `collect` vs `push_front + pop_front` dance; buggy specialized `Single == Single` returns `cmp_chunk`, which is `false` for distinct chunk identities).
