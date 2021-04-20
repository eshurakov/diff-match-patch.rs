use criterion::{black_box, criterion_group, criterion_main, Criterion};
use diff_match_patch::Dmp;

use std::fs;

fn test(text1: &str, text2: &str) {
    let dmp = Dmp::new();
    dmp.diff_main(text1, text2, true);
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let file1 = fs::read_to_string("./benches/Speedtest1.txt").unwrap();
    let file2 = fs::read_to_string("./benches/Speedtest2.txt").unwrap();

    c.bench_function("diff-main-speedtest", |b| b.iter(|| test(black_box(&file1), black_box(&file2)) ));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);