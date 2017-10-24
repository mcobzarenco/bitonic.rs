#![feature(test)]

extern crate test;
extern crate bitonic;

use test::{Bencher, black_box};


fn bench_sorter<F>(b: &mut Bencher, size: u32, sorter: F)
where
    F: Fn(&mut [u32]),
{
    let mut v = Vec::with_capacity(size as usize);
    let mut r = 0u32;
    b.iter(|| {
        v.clear();
        for _ in 0..size {
            r = r.wrapping_mul(1664525).wrapping_add(1013904223);
            v.push(r % size);
        }
        black_box(sorter(v.as_mut_slice()));
    });
}

fn std_stable(slice: &mut [u32]) {
    slice.sort();
}

fn std_unstable(slice: &mut [u32]) {
    slice.sort_unstable();
}

#[bench]
fn std_stable_32768(b: &mut Bencher) {
    bench_sorter(b, 32768, std_stable);
}

#[bench]
fn std_unstable_32768(b: &mut Bencher) {
    bench_sorter(b, 32768, std_stable);
}

#[bench]
fn std_bitonic_32768(b: &mut Bencher) {
    bench_sorter(b, 32768, bitonic::bitonic_sort);
}



#[bench]
fn std_stable_128(b: &mut Bencher) {
    bench_sorter(b, 128, std_stable);
}

#[bench]
fn std_unstable_128(b: &mut Bencher) {
    bench_sorter(b, 128, std_stable);
}

#[bench]
fn std_bitonic_128(b: &mut Bencher) {
    bench_sorter(b, 128, bitonic::bitonic_sort);
}