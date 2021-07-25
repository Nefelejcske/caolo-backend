//! Sort an array of elements by their `MortonKey`
//!
use super::morton_key::MortonKey;
use std::mem::{size_of, swap};

const RADIX_MASK_LEN: usize = 8; // how many bits are considered at a time
const NUM_BUCKETS: usize = 1usize << (RADIX_MASK_LEN as usize);
const RADIX_MASK: u32 = (NUM_BUCKETS - 1) as u32;
const MORTON_BITS: usize = size_of::<MortonKey>() * 8;

pub fn sort<T: Default>(keys: &mut Vec<MortonKey>, values: &mut [T]) {
    debug_assert!(
        keys.len() == values.len(),
        "{} {}",
        keys.len(),
        values.len()
    );
    if keys.len() < 2 {
        return;
    }
    sort_radix(keys, values);
}

#[inline]
fn sort_radix<T: Default>(keys: &mut Vec<MortonKey>, values: &mut [T]) {
    debug_assert_eq!(keys.len(), values.len());
    let mut tmp: Vec<(usize, MortonKey)> = vec![Default::default(); keys.len() * 2];
    // double buffer (index, key) pairs
    let (mut tmp_a, mut tmp_b) = tmp.as_mut_slice().split_at_mut(keys.len());
    debug_assert_eq!(tmp_a.len(), tmp_b.len());
    for (i, k) in keys.iter().enumerate() {
        tmp_a[i] = (i, *k);
    }

    let mut swapbuffs = false;
    for k in (0..=MORTON_BITS).step_by(RADIX_MASK_LEN) {
        debug_assert!(k <= std::u8::MAX as usize);
        radix_pass(k as u32, tmp_a, tmp_b);
        swap(&mut tmp_a, &mut tmp_b);
        swapbuffs = !swapbuffs;
    }

    if swapbuffs {
        swap(&mut tmp_a, &mut tmp_b);
    }

    let mut vs = Vec::with_capacity(keys.len());
    keys.clear();

    for (i, key) in tmp_a {
        keys.push(*key);
        vs.push(std::mem::take(&mut values[*i]));
    }

    vs.swap_with_slice(values);
}

fn radix_pass(
    k: u32,
    keys: &[(usize, MortonKey)], // key, index pairs
    out: &mut [(usize, MortonKey)],
) {
    let mut buckets = [0; NUM_BUCKETS];
    // compute the length of each bucket
    keys.iter().for_each(|(_, key)| {
        let bucket = compute_bucket(k, *key);
        buckets[bucket] += 1;
    });

    // set the output offsets for each bucket
    // this will indicate the 1 after the last index a chunk will occupy
    let mut base = 0;
    for bucket in buckets.iter_mut() {
        *bucket += base;
        base = *bucket;
    }

    // write the output
    //
    debug_assert_eq!(keys.len(), out.len());

    keys.iter().rev().for_each(|(id, key)| {
        let bucket = compute_bucket(k, *key);
        buckets[bucket] -= 1;
        let index = buckets[bucket];
        debug_assert!(index < out.len());
        out[index] = (*id, *key);
    });
}

#[inline(always)]
fn compute_bucket(k: u32, MortonKey(key): MortonKey) -> usize {
    let (key, _) = key.overflowing_shr(k);
    let ind = key & RADIX_MASK;
    ind as usize
}

#[cfg(test)]
mod tests {
    use rand::prelude::SliceRandom;

    use super::*;

    #[test]
    fn test_sorting_sorted() {
        let control: Vec<MortonKey> = (0..(1 << 31))
            .step_by(2000)
            .chain(
                [
                    std::u32::MAX - 4,
                    std::u32::MAX - 3,
                    std::u32::MAX - 2,
                    std::u32::MAX - 1,
                    std::u32::MAX,
                ]
                .iter()
                .copied(),
            )
            .map(|i| MortonKey(i))
            .collect();

        let mut keys = control.clone();
        keys.shuffle(&mut rand::thread_rng());

        let mut vals = keys.clone();

        sort(&mut keys, vals.as_mut_slice());

        assert_eq!(keys, control);
        assert_eq!(vals, control);
    }
}
