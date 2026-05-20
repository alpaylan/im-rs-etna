use crabcheck::profiling::quickcheck;
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

// Mirror src/bin/etna.rs's seeded cc closures: multiplicative mixing from
// the usize/seed input into the u32/u32 property args, so the distribution
// reaches the bug-triggering sizes in the same way the existing adapter does.

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() < 3 {
        return;
    }
    let result = match (args[1].as_str(), args[2].as_str()) {
        ("crabcheck", "PathNextBacktrack") => quickcheck(|seed: usize| {
            let s = seed as u32;
            {
                to_opt(property_path_next_backtrack(
                    s,
                    s.wrapping_mul(13),
                    s.wrapping_mul(11).wrapping_add(1),
                ))
            }
        }),
        ("crabcheck", "RangeOffByOne") => quickcheck(|(n, hi): (usize, usize)| {
            {
                to_opt(property_range_off_by_one(
                    (n as u32).wrapping_mul(17),
                    (hi as u32).wrapping_mul(13),
                ))
            }
        }),
        ("crabcheck", "RrbDebugPop") => quickcheck(|n: usize| {
            to_opt(property_rrb_debug_pop((n as u32).wrapping_mul(31)))
        }),
        ("crabcheck", "RrbDensityCheck") => quickcheck(|seed: usize| {
            // Mirror src/bin/etna.rs's widened cc dispatcher: project `seed`
            // onto (n_removed, size_factor, op_mode) so the distribution
            // covers small/medium Vectors plus several op modes.
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
        }),
        ("crabcheck", "PtrEqPrecedence") => quickcheck(|(n, slot): (usize, usize)| {
            {
                to_opt(property_ptr_eq_precedence(
                    (n as u32).wrapping_mul(17),
                    (slot as u32).wrapping_mul(7),
                ))
            }
        }),
        ("crabcheck", "EqSingleChunk") => quickcheck(|seed: usize| {
            let seed = seed as u32;
            let len = (seed as usize % 29) + 32;
            let mut xs = Vec::with_capacity(len);
            let mut x: u64 = seed as u64;
            for _ in 0..len {
                xs.push(x as i32);
                x = x.wrapping_mul(6364136223846793005).wrapping_add(1);
            }
            to_opt(property_eq_single_chunk(xs))
        }),
        (a, b) => panic!("Unknown: {a} {b}"),
    };
    println!("Result: {:?}", result);
}
