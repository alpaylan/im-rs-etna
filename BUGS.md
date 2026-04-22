# im-rs — Injected Bugs

Total mutations: 6

## Bug Index

| # | Variant | Name | Location | Injection | Fix Commit |
|---|---------|------|----------|-----------|------------|
| 1 | `eq_single_chunk_005193a_1` | `eq_single_chunk` | `src/vector/mod.rs` | `patch` | `005193a268e4e9939a50eff0fe6e4f232a1facf4` |
| 2 | `path_next_backtrack_41d99725_1` | `path_next_backtrack` | `src/nodes/btree.rs` | `patch` | `41d9972538d49ffa3964e3a94109619ce053ef36` |
| 3 | `ptr_eq_precedence_f744912_1` | `ptr_eq_precedence` | `src/vector/mod.rs` | `patch` | `f7449127b83327b82d82690e8333c552014149c8` |
| 4 | `range_off_by_one_3f4e01a4_1` | `range_off_by_one` | `src/nodes/btree.rs` | `patch` | `3f4e01a43254fe228d1ce64e47dfaf4edc8f4f19` |
| 5 | `rrb_debug_pop_1209e823_1` | `rrb_debug_pop` | `src/nodes/rrb.rs` | `patch` | `1209e823b633c7ac73ae686896382b7207a907ac` |
| 6 | `rrb_density_check_cb431a6_1` | `rrb_density_check` | `src/nodes/rrb.rs` | `patch` | `cb431a612a39fb7973f7a7218771b5b3fd43d979` |

## Property Mapping

| Variant | Property | Witness(es) |
|---------|----------|-------------|
| `eq_single_chunk_005193a_1` | `EqSingleChunk` | `witness_eq_single_chunk_case_small_vec` |
| `path_next_backtrack_41d99725_1` | `PathNextBacktrack` | `witness_path_next_backtrack_case_ordmap_1000` |
| `ptr_eq_precedence_f744912_1` | `PtrEqPrecedence` | `witness_ptr_eq_precedence_case_diverged_outer` |
| `range_off_by_one_3f4e01a4_1` | `RangeOffByOne` | `witness_range_off_by_one_case_odd_upper_bound` |
| `rrb_debug_pop_1209e823_1` | `RrbDebugPop` | `witness_rrb_debug_pop_case_release_pop_front` |
| `rrb_density_check_cb431a6_1` | `RrbDensityCheck` | `witness_rrb_density_check_case_level2_rrb` |

## Framework Coverage

| Property | proptest | quickcheck | crabcheck | hegel |
|----------|---------:|-----------:|----------:|------:|
| `EqSingleChunk` | ✓ | ✓ | ✓ | ✓ |
| `PathNextBacktrack` | ✓ | ✓ | ✓ | ✓ |
| `PtrEqPrecedence` | ✓ | ✓ | ✓ | ✓ |
| `RangeOffByOne` | ✓ | ✓ | ✓ | ✓ |
| `RrbDebugPop` | ✓ | ✓ | ✓ | ✓ |
| `RrbDensityCheck` | ✓ | ✓ | ✓ | ✓ |

## Bug Details

### 1. eq_single_chunk

- **Variant**: `eq_single_chunk_005193a_1`
- **Location**: `src/vector/mod.rs`
- **Property**: `EqSingleChunk`
- **Witness(es)**:
  - `witness_eq_single_chunk_case_small_vec`
- **Source**: Fix incomplete `eq` implementation for single chunks.
  > The nightly-only `has_specialisation` specialization for `Single == Single` delegated to `cmp_chunk`, which compares chunk *identities* — returning `false` for two structurally-identical-but-distinct chunks. The fix falls through to element-wise comparison when identities differ so that `Vector` equality agrees with iterator equality on all specialisation cfgs.
- **Fix commit**: `005193a268e4e9939a50eff0fe6e4f232a1facf4` — Fix incomplete `eq` implementation for single chunks.
- **Invariant violated**: `PartialEq` on two `Vector<i32>` values holding the same element sequence must return `true`, regardless of internal layout (`Single` vs `Full`, shared or fresh chunks).
- **How the mutation triggers**: The specialized `Single == Single` arm returns `cmp_chunk(left, right)` directly; `cmp_chunk` returns `false` when the two chunks have distinct identities. Two structurally-different single-chunk vectors with identical elements then compare unequal. Only reachable on the `has_specialisation` nightly cfg — base build uses the fallback `iter().eq()`, which is always correct.

### 2. path_next_backtrack

- **Variant**: `path_next_backtrack_41d99725_1`
- **Location**: `src/nodes/btree.rs`
- **Property**: `PathNextBacktrack`
- **Witness(es)**:
  - `witness_path_next_backtrack_case_ordmap_1000`
- **Source**: Fix btree::Node::path_{next/prev}
  > `Node::path_next` returned `Vec::new()` whenever its descent landed on a key the tree didn't contain and the current leaf had no candidate, instead of walking back up to find the next ancestor key. As a result `OrdMap::range(lo..hi)` silently under-counted when `lo` fell past the last key of some leaf node.
- **Fix commit**: `41d9972538d49ffa3964e3a94109619ce053ef36` — Fix btree::Node::path_{next/prev}
- **Invariant violated**: `OrdMap::range(lo..hi).count()` must equal `std::collections::BTreeMap::range(lo..hi).count()` for the same key set, regardless of tree shape.
- **How the mutation triggers**: In `Node::path_next` the `Err(index) → children[index] = None → keys.get(index) = None` arm returns `Vec::new()` instead of backtracking up ancestors. When `lo` lands past the last key of a leaf, the iterator truncates mid-descent and `range().count()` reports fewer keys than the reference.

### 3. ptr_eq_precedence

- **Variant**: `ptr_eq_precedence_f744912_1`
- **Location**: `src/vector/mod.rs`
- **Property**: `PtrEqPrecedence`
- **Witness(es)**:
  - `witness_ptr_eq_precedence_case_diverged_outer`
- **Source**: Fix logic error in Vector::ptr_eq & PartialEq
  > `Vector::ptr_eq` was written as `a && b || c && d || e` without parentheses, so Rust's `&&` precedence caused the final `||` branch to act as a fallback that ignored outer/inner chunk divergence. After `a.set(i, x)` on a clone, `ptr_eq` reduced to comparing the middle tree pointer alone and returned `true`. The fix adds parentheses so all three pointer groups must match.
- **Fix commit**: `f7449127b83327b82d82690e8333c552014149c8` — Fix logic error in Vector::ptr_eq & PartialEq
- **Invariant violated**: After `a.set(i, x)` the cloned vector `b = a.clone()` diverges at index `i`, so `a.ptr_eq(&b)` must be `false`.
- **How the mutation triggers**: Without parentheses around the final `||`, the `&&` conjunction binds tighter. `ptr_eq` reduces to an identity test on the middle tree pointer alone and silently ignores diverging outer or inner chunks.

### 4. range_off_by_one

- **Variant**: `range_off_by_one_3f4e01a4_1`
- **Location**: `src/nodes/btree.rs`
- **Property**: `RangeOffByOne`
- **Witness(es)**:
  - `witness_range_off_by_one_case_odd_upper_bound`
- **Source**: Fix OrdMap::range including keys outside of requested range if end bound doesn't exist in tree.
  > `Node::path_prev` used `keys.get(index)` when the upper bound wasn't present in the tree, which pointed at the key one past `hi` instead of the key just before it. `OrdMap::range(..hi)` therefore leaked the next key into the iteration. The fix uses `keys.get(index - 1)` so the exclusive upper bound is honored.
- **Fix commit**: `3f4e01a43254fe228d1ce64e47dfaf4edc8f4f19` — Fix OrdMap::range including keys outside of requested range if end bound doesn't exist in tree.
- **Invariant violated**: `OrdMap::range(..hi)` must never yield a key `k >= hi`.
- **How the mutation triggers**: `Node::path_prev` uses `keys.get(index)` instead of `keys.get(index - 1)` when the upper bound is absent from the tree, so the iterator leaks the key immediately past `hi`.

### 5. rrb_debug_pop

- **Variant**: `rrb_debug_pop_1209e823_1`
- **Location**: `src/nodes/rrb.rs`
- **Property**: `RrbDebugPop`
- **Witness(es)**:
  - `witness_rrb_debug_pop_case_release_pop_front`
- **Source**: RRB size table: don't pop conditional on cfg(debug_assertions)
  > The `Size::Table` left-pop path put the side-effectful `size_table.pop_front()` call inside a `debug_assert_eq!`. In release builds the entire macro body was compiled out, so the size table never shrank, and later random-access indexing after `pop_front` read stale entries and panicked. The fix hoists the pop above the debug-only assertion.
- **Fix commit**: `1209e823b633c7ac73ae686896382b7207a907ac` — RRB size table: don't pop conditional on cfg(debug_assertions)
- **Invariant violated**: After a sequence of `Vector::pop_front` operations, `v.get(i)` must agree with `v.iter().nth(i)` for every valid `i`.
- **How the mutation triggers**: The `Size::Table` Left-pop arm places the side-effectful `size_table.pop_front()` inside `debug_assert_eq!`. In release builds the entire macro body is compiled out, the size table is never popped, and later random-access indexing consults a stale entry and panics with index-out-of-bounds.

### 6. rrb_density_check

- **Variant**: `rrb_density_check_cb431a6_1`
- **Location**: `src/nodes/rrb.rs`
- **Property**: `RrbDensityCheck`
- **Witness(es)**:
  - `witness_rrb_density_check_case_level2_rrb`
- **Source**: Many Vector merging/splitting bug fixes.
  > `Node::parent` used `child.is_full()` (a local capacity check) to decide whether to keep compact `Size::Size` accounting for a new parent. At level 2+, a nominally full child whose grandchildren are sparse still warrants the full `Size::Table` accounting — the fix substitutes `is_completely_dense(level - 1)` so sparse grand-children force `Size::Table`.
- **Fix commit**: `cb431a612a39fb7973f7a7218771b5b3fd43d979` — Many Vector merging/splitting bug fixes.
- **Invariant violated**: After `remove`/`split_at`/`append` on a level-2+ RRB tree, `v.get(i)` must agree with `v.iter()` for all indices.
- **How the mutation triggers**: `Node::parent` uses `child.is_full()` instead of `child.is_completely_dense(level - 1)` to decide whether to keep `Size::Size` accounting. When a level-2 node has a nominally-full child whose grandchildren are sparse, it wrongly stays `Size::Size`, and subsequent index arithmetic lands off-by-several.
