# im-rs — Injected Bugs

Total mutations: 6

## Bug Index

| # | Name | Variant | File | Injection | Fix Commit |
|---|------|---------|------|-----------|------------|
| 1 | `path_next_backtrack` | `path_next_backtrack_41d99725_1` | `patches/path_next_backtrack_41d99725_1.patch` | `patch` | `41d9972538d49ffa3964e3a94109619ce053ef36` |
| 2 | `range_off_by_one` | `range_off_by_one_3f4e01a4_1` | `patches/range_off_by_one_3f4e01a4_1.patch` | `patch` | `3f4e01a43254fe228d1ce64e47dfaf4edc8f4f19` |
| 3 | `rrb_debug_pop` | `rrb_debug_pop_1209e823_1` | `patches/rrb_debug_pop_1209e823_1.patch` | `patch` | `1209e823b633c7ac73ae686896382b7207a907ac` |
| 4 | `rrb_density_check` | `rrb_density_check_cb431a6_1` | `patches/rrb_density_check_cb431a6_1.patch` | `patch` | `cb431a612a39fb7973f7a7218771b5b3fd43d979` |
| 5 | `ptr_eq_precedence` | `ptr_eq_precedence_f744912_1` | `patches/ptr_eq_precedence_f744912_1.patch` | `patch` | `f7449127b83327b82d82690e8333c552014149c8` |
| 6 | `eq_single_chunk` | `eq_single_chunk_005193a_1` | `patches/eq_single_chunk_005193a_1.patch` | `patch` | `005193a268e4e9939a50eff0fe6e4f232a1facf4` |

## Property Mapping

| Variant | Property | Witness(es) |
|---------|----------|-------------|
| `path_next_backtrack_41d99725_1` | `property_path_next_backtrack` | `witness_path_next_backtrack_case_ordmap_1000` |
| `range_off_by_one_3f4e01a4_1` | `property_range_off_by_one` | `witness_range_off_by_one_case_odd_upper_bound` |
| `rrb_debug_pop_1209e823_1` | `property_rrb_debug_pop` | `witness_rrb_debug_pop_case_release_pop_front` |
| `rrb_density_check_cb431a6_1` | `property_rrb_density_check` | `witness_rrb_density_check_case_level2_rrb` |
| `ptr_eq_precedence_f744912_1` | `property_ptr_eq_precedence` | `witness_ptr_eq_precedence_case_diverged_outer` |
| `eq_single_chunk_005193a_1` | `property_eq_single_chunk` | `witness_eq_single_chunk_case_small_vec` |

## Framework Coverage

| Property | proptest | quickcheck | crabcheck | hegel |
|----------|---------:|-----------:|----------:|------:|
| `property_path_next_backtrack` | ✓ | ✓ | ✓ | ✓ |
| `property_range_off_by_one` | ✓ | ✓ | ✓ | ✓ |
| `property_rrb_debug_pop` | ✓ | ✓ | ✓ | ✓ |
| `property_rrb_density_check` | ✓ | ✓ | ✓ | ✓ |
| `property_ptr_eq_precedence` | ✓ | ✓ | ✓ | ✓ |
| `property_eq_single_chunk` | ✓ | ✓ | ✓ | ✓ |

## Bug Details

### 1. path_next_backtrack
- **Variant**: `path_next_backtrack_41d99725_1`
- **Location**: `patches/path_next_backtrack_41d99725_1.patch` (target `src/nodes/btree.rs`)
- **Property**: `property_path_next_backtrack`
- **Witness(es)**: `witness_path_next_backtrack_case_ordmap_1000`
- **Fix commit**: `41d9972538d49ffa3964e3a94109619ce053ef36` — Fix btree::Node::path_{next/prev}
- **Invariant violated**: `OrdMap::range(lo..hi).count()` must equal `std::collections::BTreeMap::range(lo..hi).count()` for the same key set, regardless of tree shape.
- **How the mutation triggers**: In `Node::path_next` the `Err(index) → children[index] = None → keys.get(index) = None` arm returns `Vec::new()` instead of backtracking up ancestors. When `lo` lands past the last key of a leaf, the iterator truncates mid-descent and `range().count()` reports fewer keys than the reference.

### 2. range_off_by_one
- **Variant**: `range_off_by_one_3f4e01a4_1`
- **Location**: `patches/range_off_by_one_3f4e01a4_1.patch` (target `src/nodes/btree.rs`)
- **Property**: `property_range_off_by_one`
- **Witness(es)**: `witness_range_off_by_one_case_odd_upper_bound`
- **Fix commit**: `3f4e01a43254fe228d1ce64e47dfaf4edc8f4f19` — Fix OrdMap::range including keys outside of requested range if end bound doesn't exist in tree.
- **Invariant violated**: `OrdMap::range(..hi)` must never yield a key `k >= hi`.
- **How the mutation triggers**: `Node::path_prev` uses `keys.get(index)` instead of `keys.get(index - 1)` when the upper bound is absent from the tree, so the iterator leaks the key immediately past `hi`.

### 3. rrb_debug_pop
- **Variant**: `rrb_debug_pop_1209e823_1`
- **Location**: `patches/rrb_debug_pop_1209e823_1.patch` (target `src/nodes/rrb.rs`)
- **Property**: `property_rrb_debug_pop`
- **Witness(es)**: `witness_rrb_debug_pop_case_release_pop_front`
- **Fix commit**: `1209e823b633c7ac73ae686896382b7207a907ac` — RRB size table: don't pop conditional on cfg(debug_assertions)
- **Invariant violated**: After a sequence of `Vector::pop_front` operations, `v.get(i)` must agree with `v.iter().nth(i)` for every valid `i`.
- **How the mutation triggers**: The `Size::Table` Left-pop arm places the side-effectful `size_table.pop_front()` inside `debug_assert_eq!`. In release builds the entire macro body is compiled out, the size table is never popped, and later random-access indexing consults a stale entry and panics with index-out-of-bounds.

### 4. rrb_density_check
- **Variant**: `rrb_density_check_cb431a6_1`
- **Location**: `patches/rrb_density_check_cb431a6_1.patch` (target `src/nodes/rrb.rs`)
- **Property**: `property_rrb_density_check`
- **Witness(es)**: `witness_rrb_density_check_case_level2_rrb`
- **Fix commit**: `cb431a612a39fb7973f7a7218771b5b3fd43d979` — Many Vector merging/splitting bug fixes.
- **Invariant violated**: After `remove`/`split_at`/`append` on a level-2+ RRB tree, `v.get(i)` must agree with `v.iter()` for all indices.
- **How the mutation triggers**: `Node::parent` uses `child.is_full()` instead of `child.is_completely_dense(level - 1)` to decide whether to keep `Size::Size` accounting. When a level-2 node has a nominally-full child whose grandchildren are sparse, it wrongly stays `Size::Size`, and subsequent index arithmetic lands off-by-several.

### 5. ptr_eq_precedence
- **Variant**: `ptr_eq_precedence_f744912_1`
- **Location**: `patches/ptr_eq_precedence_f744912_1.patch` (target `src/vector/mod.rs`)
- **Property**: `property_ptr_eq_precedence`
- **Witness(es)**: `witness_ptr_eq_precedence_case_diverged_outer`
- **Fix commit**: `f7449127b83327b82d82690e8333c552014149c8` — Fix logic error in Vector::ptr_eq & PartialEq
- **Invariant violated**: After `a.set(i, x)` the cloned vector `b = a.clone()` diverges at index `i`, so `a.ptr_eq(&b)` must be `false`.
- **How the mutation triggers**: Without parentheses around the final `||`, the `&&` conjunction binds tighter. `ptr_eq` reduces to an identity test on the middle tree pointer alone and silently ignores diverging outer or inner chunks.

### 6. eq_single_chunk
- **Variant**: `eq_single_chunk_005193a_1`
- **Location**: `patches/eq_single_chunk_005193a_1.patch` (target `src/vector/mod.rs`)
- **Property**: `property_eq_single_chunk`
- **Witness(es)**: `witness_eq_single_chunk_case_small_vec`
- **Fix commit**: `005193a268e4e9939a50eff0fe6e4f232a1facf4` — Fix incomplete `eq` implementation for single chunks.
- **Invariant violated**: `PartialEq` on two `Vector<i32>` values holding the same element sequence must return `true`, regardless of internal layout (`Single` vs `Full`, shared or fresh chunks).
- **How the mutation triggers**: The specialized `Single == Single` arm returns `cmp_chunk(left, right)` directly; `cmp_chunk` returns `false` when the two chunks have distinct identities. Two structurally-different single-chunk vectors with identical elements then compare unequal. Only reachable on the `has_specialisation` nightly cfg — base build uses the fallback `iter().eq()`, which is always correct.
