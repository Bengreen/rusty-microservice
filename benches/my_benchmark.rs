use criterion::{
    Throughput,
    BenchmarkId,
    black_box,criterion_group,
    criterion_main,
    Criterion,
};
use std::iter;


fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 1,
        1 => 1,
        n => fibonacci(n-1) + fibonacci(n-2),
    }
}

fn fibonacci_fast(n: u64) -> u64 {
    let mut a = 0;
    let mut b = 1;

    match n {
        0 => b,
        _ => {
            for _ in 0..n {
                let c = a + b;
                a = b;
                b = c;
            }
            b
        }
    }
}


fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("fib 20", |b| b.iter(|| fibonacci(black_box(20))));
}

fn bench_fibs(c: &mut Criterion) {
    let mut group = c.benchmark_group("Fibonacci");
    for i in [20u64, 21u64, 22u64, 23u64].iter() {
        group.bench_with_input(BenchmarkId::new("Recursive", i), i,
            |b, i| b.iter(|| fibonacci(*i)));
        group.bench_with_input(BenchmarkId::new("Iterative", i), i,
            |b, i| b.iter(|| fibonacci_fast(*i)));
    }
    group.finish();
}



fn from_elem(c: &mut Criterion) {
    static KB: usize = 1024;

    let mut group = c.benchmark_group("from_elem");
    for size in [KB, 2 * KB, 4 * KB, 8 * KB, 16 * KB].iter() {
        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| iter::repeat(0u8).take(size).collect::<Vec<_>>());
        });
    }
    group.finish();
}


criterion_group!(benches, criterion_benchmark, from_elem, bench_fibs);
criterion_main!(benches);
