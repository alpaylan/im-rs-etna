# im-rs — ETNA Tasks

Total tasks: 24

## Task Index

| Task | Variant | Framework | Property | Witness |
|------|---------|-----------|----------|---------|
| 001 | `eq_single_chunk_005193a_1` | proptest | `EqSingleChunk` | `witness_eq_single_chunk_case_small_vec` |
| 002 | `eq_single_chunk_005193a_1` | quickcheck | `EqSingleChunk` | `witness_eq_single_chunk_case_small_vec` |
| 003 | `eq_single_chunk_005193a_1` | crabcheck | `EqSingleChunk` | `witness_eq_single_chunk_case_small_vec` |
| 004 | `eq_single_chunk_005193a_1` | hegel | `EqSingleChunk` | `witness_eq_single_chunk_case_small_vec` |
| 005 | `path_next_backtrack_41d99725_1` | proptest | `PathNextBacktrack` | `witness_path_next_backtrack_case_ordmap_1000` |
| 006 | `path_next_backtrack_41d99725_1` | quickcheck | `PathNextBacktrack` | `witness_path_next_backtrack_case_ordmap_1000` |
| 007 | `path_next_backtrack_41d99725_1` | crabcheck | `PathNextBacktrack` | `witness_path_next_backtrack_case_ordmap_1000` |
| 008 | `path_next_backtrack_41d99725_1` | hegel | `PathNextBacktrack` | `witness_path_next_backtrack_case_ordmap_1000` |
| 009 | `ptr_eq_precedence_f744912_1` | proptest | `PtrEqPrecedence` | `witness_ptr_eq_precedence_case_diverged_outer` |
| 010 | `ptr_eq_precedence_f744912_1` | quickcheck | `PtrEqPrecedence` | `witness_ptr_eq_precedence_case_diverged_outer` |
| 011 | `ptr_eq_precedence_f744912_1` | crabcheck | `PtrEqPrecedence` | `witness_ptr_eq_precedence_case_diverged_outer` |
| 012 | `ptr_eq_precedence_f744912_1` | hegel | `PtrEqPrecedence` | `witness_ptr_eq_precedence_case_diverged_outer` |
| 013 | `range_off_by_one_3f4e01a4_1` | proptest | `RangeOffByOne` | `witness_range_off_by_one_case_odd_upper_bound` |
| 014 | `range_off_by_one_3f4e01a4_1` | quickcheck | `RangeOffByOne` | `witness_range_off_by_one_case_odd_upper_bound` |
| 015 | `range_off_by_one_3f4e01a4_1` | crabcheck | `RangeOffByOne` | `witness_range_off_by_one_case_odd_upper_bound` |
| 016 | `range_off_by_one_3f4e01a4_1` | hegel | `RangeOffByOne` | `witness_range_off_by_one_case_odd_upper_bound` |
| 017 | `rrb_debug_pop_1209e823_1` | proptest | `RrbDebugPop` | `witness_rrb_debug_pop_case_release_pop_front` |
| 018 | `rrb_debug_pop_1209e823_1` | quickcheck | `RrbDebugPop` | `witness_rrb_debug_pop_case_release_pop_front` |
| 019 | `rrb_debug_pop_1209e823_1` | crabcheck | `RrbDebugPop` | `witness_rrb_debug_pop_case_release_pop_front` |
| 020 | `rrb_debug_pop_1209e823_1` | hegel | `RrbDebugPop` | `witness_rrb_debug_pop_case_release_pop_front` |
| 021 | `rrb_density_check_cb431a6_1` | proptest | `RrbDensityCheck` | `witness_rrb_density_check_case_level2_rrb` |
| 022 | `rrb_density_check_cb431a6_1` | quickcheck | `RrbDensityCheck` | `witness_rrb_density_check_case_level2_rrb` |
| 023 | `rrb_density_check_cb431a6_1` | crabcheck | `RrbDensityCheck` | `witness_rrb_density_check_case_level2_rrb` |
| 024 | `rrb_density_check_cb431a6_1` | hegel | `RrbDensityCheck` | `witness_rrb_density_check_case_level2_rrb` |

## Witness Catalog

- `witness_eq_single_chunk_case_small_vec` — base passes, variant fails
- `witness_path_next_backtrack_case_ordmap_1000` — base passes, variant fails
- `witness_ptr_eq_precedence_case_diverged_outer` — base passes, variant fails
- `witness_range_off_by_one_case_odd_upper_bound` — base passes, variant fails
- `witness_rrb_debug_pop_case_release_pop_front` — base passes, variant fails
- `witness_rrb_density_check_case_level2_rrb` — base passes, variant fails
