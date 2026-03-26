# im-rs Injected Bugs

This workload contains 6 bugs derived from real historical fixes in
[bodil/im-rs](https://github.com/bodil/im-rs), an immutable collections
library for Rust featuring `Vector` (RRB tree), `HashMap` (HAMT), and
`OrdMap` (B-tree).

Each bug is injected using [marauders](http://github.com/alpaylan/marauders)
comment syntax. In the base (default) configuration all code is correct and
all 115 tests pass. Activating a variant via `M_<variant>=active` introduces
the historical bug.

## Method

1. Scanned 506 commits for bug-fix patterns (commit messages containing
   "fix", "bug", issue references).
2. Identified 13 candidates with clear before/after code changes.
3. Ranked by mutation-injection fitness (single-site change, expressible
   in marauders syntax, code still present in HEAD).
4. Injected mutations, converted to functional syntax, and tested each
   variant against the full test suite.
5. Kept only mutations that are detectable (cause at least one test
   failure). Wrote targeted unit tests where the existing suite was
   insufficient.

Bugs that were investigated but excluded:

- **sort_var_scope** (0fc79ba6) — variable scope mutation incompatible
  with marauders functional syntax (variable declarations can't be wrapped
  in `match` expressions).
- **remove_action_priority** (cf08eec) — both priority orderings produce
  valid B-trees; functionally equivalent.
- **rrb_index_lookup** (1e8c3e4) — `while` vs `if` in size table lookup;
  existing tests don't create RRB trees complex enough to trigger.
- **eq_precedence** (f744912) — same precedence bug as `ptr_eq_precedence`
  but in `PartialEq`; existing test coverage too weak.
- **sparse_left_push** (617c782 / issue #77) — the code path is dead in
  the current codebase due to subsequent refactoring.

---

## Bug 1: `path_next_backtrack`

|                |                                          |
| -------------- | ---------------------------------------- |
| **File**       | `src/nodes/btree.rs:397`                 |
| **Variant**    | `path_next_backtrack_1`                  |
| **Tags**       | `btree`, `iterator`, `missing-backtrack` |
| **Fix commit** | `41d99725`                               |
| **Type**       | Missing backtrack logic                  |
| **Test mode**  | debug                                    |

### Description

`Node::path_next` finds the path to the next key in a B-tree after a
given key. When the search reaches the end of a node (the key falls past
all entries and there is no child to recurse into), the correct behavior
is to backtrack up the accumulated path to find the next ancestor with
remaining keys.

The bug returns `Vec::new()` (an empty path) instead of backtracking,
causing range iterators to silently stop early when they cross certain
internal node boundaries.

### Correct code

```rust
None => {
    while let Some((node, idx)) = path.last() {
        if node.keys.len() == *idx {
            path.pop();
        } else {
            break;
        }
    }
    path
}
```

### Buggy code

```rust
None => Vec::new()
```

### Reproduction

```rust
// Range iteration over a large OrdMap — the iterator stops early
// when it would need to backtrack across a node boundary.
let map: OrdMap<i32, i32> = (0..1000).map(|i| (i, i)).collect();
let range: Vec<_> = map.range(100..200).collect();
assert_eq!(range.len(), 100); // fails with bug: returns fewer elements
```

### Detecting tests

- `ord::map::test::range_iter_big`

---

## Bug 2: `range_off_by_one`

|                |                                                   |
| -------------- | ------------------------------------------------- |
| **File**       | `src/nodes/btree.rs:457`                          |
| **Variant**    | `range_off_by_one_1`                              |
| **Tags**       | `btree`, `range`, `off-by-one`, `issue-143`       |
| **Fix commit** | `3f4e01a4` (c0609ceb)                             |
| **Issue**      | [#143](https://github.com/bodil/im-rs/issues/143) |
| **Type**       | Off-by-one                                        |
| **Test mode**  | debug                                             |

### Description

`Node::path_prev` finds the path to the previous key before a given key.
When the search lands between two keys at an `Err(index)` position and
the child at that index is `None`, it should push `(self, index - 1)` —
the key just before the search position. The bug pushes `(self, index)`
instead, which is one position too far right.

This causes `OrdMap::range` to include keys outside the requested range
when the upper bound doesn't exist as an exact key in the tree.

### Correct code

```rust
path.push((self, index - 1));
```

### Buggy code

```rust
path.push((self, index));
```

### Reproduction

```rust
let map: OrdMap<i32, i32> = (0..1000).map(|i| (i * 2, i)).collect();
// Upper bound 501 doesn't exist (keys are even). With the bug, the
// range includes key 502 which should be excluded.
let range: Vec<_> = map.range(..501).collect();
assert!(range.last().map(|(k, _)| **k).unwrap_or(0) <= 500);
```

### Detecting tests

- `ord::map::test::range_iter_big`
- `ord::map::test::ranged_iter`

---

## Bug 3: `rrb_debug_pop`

|                |                                                  |
| -------------- | ------------------------------------------------ |
| **File**       | `src/nodes/rrb.rs:110`                           |
| **Variant**    | `rrb_debug_pop_1`                                |
| **Tags**       | `rrb`, `debug-assert`, `side-effect`, `issue-72` |
| **Fix commit** | `1209e823` (e7296d20)                            |
| **Issue**      | [#72](https://github.com/bodil/im-rs/issues/72)  |
| **Type**       | Side effect inside debug assertion               |
| **Test mode**  | release (`--release`)                            |

### Description

The RRB tree's `Size::pop` method removes an entry from the size table
when a child is removed. The `pop_front()` call is a side effect — it
mutates the size table. In the buggy version, this side effect is placed
inside `debug_assert_eq!`:

```rust
debug_assert_eq!(value, size_table.pop_front());
```

In debug mode this works correctly because `debug_assert_eq!` evaluates
both arguments. In release mode, `debug_assert_eq!` is compiled away
entirely, so `pop_front()` never executes. The size table is never
updated, causing subsequent operations to use stale size data.

This is a textbook example of why side effects should never be placed
inside debug assertions.

### Correct code

```rust
let first = size_table.pop_front();
debug_assert_eq!(value, first);
```

### Buggy code

```rust
debug_assert_eq!(value, size_table.pop_front());
```

### Reproduction

```rust
// Only manifests in release mode (cargo test --release).
// Any sequence of Vector operations that triggers size table
// updates on the left side will eventually crash.
let mut v: Vector<i32> = (0..10000).collect();
for _ in 0..5000 {
    v.pop_front();
}
// panics at src/vector/focus.rs with "index out of bounds" or SIGABRT
```

### Detecting tests

- Crashes with SIGABRT in release mode (any test exercising vector
  append/split at sufficient scale).

---

## Bug 4: `rrb_density_check`

|                |                                                          |
| -------------- | -------------------------------------------------------- |
| **File**       | `src/nodes/rrb.rs:281`                                   |
| **Variant**    | `rrb_density_check_1`                                    |
| **Tags**       | `rrb`, `density`, `wrong-predicate`                      |
| **Fix commit** | `cb431a6`                                                |
| **Issue**      | [#55](https://github.com/bodil/im-rs/issues/55) (closes) |
| **Type**       | Wrong predicate                                          |
| **Test mode**  | debug                                                    |

### Description

`Node::parent` constructs a parent node from a chunk of children and
must decide whether to use `Size::Size` (uniform sizing, O(1) index
lookup) or `Size::Table` (explicit size table, handles non-uniform
children). The decision hinges on checking whether each non-last child
is completely dense.

The bug uses `is_full()` instead of `is_completely_dense(level - 1)`.
At level 0 (leaves) these are equivalent — a full leaf IS completely
dense. But at level >= 1, a node can be *full* (has 64 children) without
being *completely dense* (some grandchildren may be sparse). Using the
wrong predicate causes the parent to assume uniform child sizes when
they aren't uniform, leading to incorrect index calculations.

This only manifests with trees large enough to have level-2+ nodes
(>262K elements) built through operations that create sparse internal
nodes (append, split).

### Correct code

```rust
if size.is_size()
    && !child.is_completely_dense(level - 1)
    && it.peek().is_some()
{
    size = Size::table_from_size(&pool.size_pool, level, size.size());
}
```

### Buggy code

```rust
if size.is_size()
    && !child.is_full()
    && it.peek().is_some()
{
    size = Size::table_from_size(&pool.size_pool, level, size.size());
}
```

### Reproduction

```rust
use im::vector::Vector;

// Build a 263K-element vector (needs level-2 RRB tree)
let total = 64 * 64 * 64 + 640; // NODE_SIZE^3 + extra
let mut v: Vector<i32> = (0..total as i32).collect();
// Remove scattered elements to create sparse leaves
for i in (0..200).rev() {
    let idx = (i * (total / 200)) + 7;
    if idx < v.len() { v.remove(idx); }
}
// Split and re-append triggers merge_rebalance at level >= 2,
// where parent() with wrong predicate produces bad Size tracking
let split = v.len() / 3;
let (left, right) = v.split_at(split);
let mut result = left;
result.append(right);
// Index lookups return wrong elements
let collected: Vec<_> = result.iter().cloned().collect();
for (i, expected) in collected.iter().enumerate() {
    assert_eq!(Some(expected), result.get(i)); // fails
}
```

### Detecting tests

- `vector::test::rrb_density_check_append` (added as part of this workload)

---

## Bug 5: `ptr_eq_precedence`

|                |                                                   |
| -------------- | ------------------------------------------------- |
| **File**       | `src/vector/mod.rs:367`                           |
| **Variant**    | `ptr_eq_precedence_1`                             |
| **Tags**       | `vector`, `ptr_eq`, `precedence`, `issue-131`     |
| **Fix commit** | `f744912`                                         |
| **Issue**      | [#131](https://github.com/bodil/im-rs/issues/131) |
| **Type**       | Operator precedence                               |
| **Test mode**  | debug                                             |

### Description

`Vector::ptr_eq` checks structural equality by comparing internal
pointers. The final condition checks whether both middles are empty OR
share the same pointer. The correct expression requires parentheses
around the `||` because `&&` binds tighter:

```
a && b && c && d && ((empty_l && empty_r) || ptr_eq(mid_l, mid_r))
```

Without the outer parentheses, `||` has lower precedence than `&&`,
so the expression parses as:

```
(a && b && c && d && (empty_l && empty_r)) || ptr_eq(mid_l, mid_r)
```

This causes `ptr_eq` to return `true` whenever two vectors happen to
share the same middle tree pointer, regardless of whether their outer
chunks differ. The bug was introduced in commit `6d0b068b` and then
copied into `ptr_eq` when it was added.

### Correct code

```rust
cmp_chunk(&left.outer_f, &right.outer_f)
    && cmp_chunk(&left.inner_f, &right.inner_f)
    && cmp_chunk(&left.inner_b, &right.inner_b)
    && cmp_chunk(&left.outer_b, &right.outer_b)
    && ((left.middle.is_empty() && right.middle.is_empty())
        || Ref::ptr_eq(&left.middle, &right.middle))
```

### Buggy code

```rust
cmp_chunk(&left.outer_f, &right.outer_f)
    && cmp_chunk(&left.inner_f, &right.inner_f)
    && cmp_chunk(&left.inner_b, &right.inner_b)
    && cmp_chunk(&left.outer_b, &right.outer_b)
    && (left.middle.is_empty() && right.middle.is_empty())
    || Ref::ptr_eq(&left.middle, &right.middle)
```

### Reproduction

```rust
let mut a: Vector<i32> = (0..200).collect();
let b = a.clone(); // shares middle pointer
a.set(0, 999);     // modifies outer_f, middle still shared
// With the bug, ptr_eq returns true because middle pointers match,
// even though outer_f differs.
assert!(!a.ptr_eq(&b)); // fails with bug
```

### Detecting tests

- `vector::test::issue_131`
- `vector::test::ptr_eq`

---

## Bug 6: `eq_single_chunk`

|                |                                          |
| -------------- | ---------------------------------------- |
| **File**       | `src/vector/mod.rs:1752`                 |
| **Variant**    | `eq_single_chunk_1`                      |
| **Tags**       | `vector`, `equality`, `missing-fallback` |
| **Fix commit** | `005193a`                                |
| **Type**       | Missing fallback                         |
| **Test mode**  | debug                                    |

### Description

`PartialEq` for `Vector` has a fast path for the `(Single, Single)` case
that compares the internal chunk pointers directly via `cmp_chunk`. This
is a structural optimization — if the chunks share the same pointer, the
vectors are equal without element-by-element comparison.

However, two vectors with identical elements but *different* internal
structure (different chunk pointers) must still compare as equal. The
correct implementation falls through to `self.iter().eq(other.iter())`
when `cmp_chunk` returns false. The buggy version returns the `cmp_chunk`
result directly, causing vectors with identical contents but different
internal layouts to compare as unequal.

This affects any code path that creates structurally different vectors
with the same elements (e.g., building the same vector via different
sequences of push/pop operations).

### Correct code

```rust
(Single(_, left), Single(_, right)) => {
    if cmp_chunk(left, right) {
        return true;
    }
    self.iter().eq(other.iter())
}
```

### Buggy code

```rust
(Single(_, left), Single(_, right)) => {
    cmp_chunk(left, right)
}
```

### Reproduction

```rust
let a: Vector<i32> = vec![1, 2, 3].into_iter().collect();
let mut b: Vector<i32> = vec![0, 1, 2, 3].into_iter().collect();
b.pop_front(); // same elements, different internal chunk layout
assert_eq!(a, b); // fails with bug
```

### Detecting tests

- `lib_test::update_in`
- `ser::test::ser_vector`
- `tests::vector::comprehensive`
- `vector::test::chunks`
- `vector::test::iter_mut`
- `vector::test::focus`
- `vector::test::chunks_mut`
