#[cfg(test)]
#[macro_use]
extern crate quickcheck;
extern crate rayon;

use std::cmp::Ordering;
use std::mem;
use rayon::prelude::*;

#[inline]
pub fn bitonic_sort_by<T: Send, F: Send + Sync + Fn(&T, &T) -> Ordering>(slice: &mut [T], by: F) {
    do_bitonic_sort_by(
        slice,
        &|left, right| by(right, left) == Ordering::Greater,
        true,
    )
}

#[inline]
pub fn bitonic_sort_by_key<T: Send, K, F: Send + Sync + Fn(&T) -> K>(slice: &mut [T], key: F)
where
    K: Ord,
{
    bitonic_sort_by(slice, |left, right| key(left).cmp(&key(right)));
}

#[inline]
pub fn bitonic_sort<T: Send>(slice: &mut [T])
where
    T: Ord,
{
    bitonic_sort_by(slice, Ord::cmp);
}

fn do_bitonic_sort_by<T: Send, F: Send + Sync + Fn(&T, &T) -> bool>(
    slice: &mut [T],
    by: &F,
    up: bool,
) {
    assert!(is_zero_or_pow2(slice.len()));
    if slice.len() <= 1 {
        return;
    }

    let half = slice.len() / 2;
    let (left, right) = slice.split_at_mut(half);
    rayon::join(|| do_bitonic_sort_by(left, by, true), || {
        do_bitonic_sort_by(right, by, false)
    });
    bitonic_merge_by(left, right, by, up);
}

#[inline]
fn bitonic_split_and_merge_by<T: Send, F: Send + Sync + Fn(&T, &T) -> bool>(
    slice: &mut [T],
    by: &F,
    up: bool,
) {
    let half = slice.len() / 2;
    let (left, right) = slice.split_at_mut(half);
    bitonic_merge_by(left, right, by, up);
}

fn bitonic_merge_by<T: Send, F: Send + Sync + Fn(&T, &T) -> bool>(
    left: &mut [T],
    right: &mut [T],
    by: &F,
    up: bool,
) {
    left.par_iter_mut().zip(right.par_iter_mut()).for_each(
        |(a, b)| {
            if by(a, b) != up {
                mem::swap(a, b);
            }
        },
    );
    if left.len() > 1 {
        rayon::join(move || bitonic_split_and_merge_by(left, by, up), move || {
            bitonic_split_and_merge_by(right, by, up)
        });
    }
}

fn is_zero_or_pow2(x: usize) -> bool {
    (x & (x.wrapping_sub(1)) == 0)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::hash::Hash;
    use std::cmp::Ordering;

    use super::{bitonic_sort_by, bitonic_sort, bitonic_sort_by_key};


    fn next_pow2(mut v: usize) -> usize {
        v = v.wrapping_sub(1);
        v |= v >> 1;
        v |= v >> 2;
        v |= v >> 4;
        v |= v >> 8;
        v |= v >> 16;
        v.wrapping_add(1)
    }

    fn frequencies<'a, T: Hash + Eq + 'a>(original: &'a [T]) -> HashMap<&'a T, usize> {
        let mut frequencies = HashMap::with_capacity(original.len());
        for item in original {
            *frequencies.entry(item).or_insert(0) += 1;
        }
        frequencies
    }

    fn is_sorted_by<T: Hash + Eq, F: Fn(&T, &T) -> Ordering>(
        original: &[T],
        sorted: &[T],
        by: F,
    ) -> bool {
        if frequencies(original) != frequencies(sorted) {
            return false;
        }

        let result = sorted.iter().zip(sorted.iter().skip(1)).all(
            |(current, next)| {
                by(current, next) != Ordering::Greater
            },
        );
        result
    }

    fn is_sorted_by_key<T: Hash + Eq, K: Ord, F: Fn(&T) -> K>(
        original: &[T],
        sorted: &[T],
        key: F,
    ) -> bool {
        is_sorted_by(original, sorted, |left, right| key(left).cmp(&key(right)))
    }

    fn is_sorted<T: Ord + Hash + Eq>(original: &[T], sorted: &[T]) -> bool {
        is_sorted_by(original, sorted, Ord::cmp)
    }

    fn make_pow2_vec(mut xs: Vec<u32>) -> Vec<u32> {
        let pow2len = next_pow2(xs.len());
        xs.resize(pow2len, 0u32);
        xs
    }

    quickcheck! {
        fn test_sort(xs: Vec<u32>) -> bool {
            let xs = make_pow2_vec(xs);
            let mut sorted = xs.clone();
            bitonic_sort(&mut sorted);
            is_sorted(&xs, &sorted)
        }

        fn test_sort_by(xs: Vec<u32>) -> bool {
            let xs = make_pow2_vec(xs);
            fn by(left: &u32, right: &u32) -> Ordering {
                if left % 2 == 0 {
                    if right % 2 == 0 {
                        left.cmp(&right)
                    } else {
                        Ordering::Less
                    }
                } else if right % 2 == 0 {
                    Ordering::Greater
                } else {
                    right.cmp(&left)
                }
            }
            let mut sorted = xs.clone();
            bitonic_sort_by(&mut sorted, by);
            is_sorted_by(&xs, &sorted, by)
        }

        fn test_sort_by_key(xs: Vec<u32>) -> bool {
            let xs = make_pow2_vec(xs);
            fn key(item: &u32) -> i64 {
                -i64::from(*item)
            }
            let mut sorted = xs.clone();
            bitonic_sort_by_key(&mut sorted, key);
            is_sorted_by_key(&xs, &sorted, key)
        }
    }
}
