use criterion::{
    // black_box,
    criterion_group,
    criterion_main,
    // Throughput,
    BenchmarkId,
    Criterion,
};

use log::info;
use std::sync::atomic::{AtomicI32, Ordering};
use uservice::{process, register_service};

static COUNT_NUM: AtomicI32 = AtomicI32::new(0);

fn ffi_calls(c: &mut Criterion) {
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
    fn fibonacci(n: u64) -> u64 {
        match n {
            0 => 1,
            1 => 1,
            n => fibonacci(n - 1) + fibonacci(n - 2),
        }
    }

    extern "C" fn init_me(a: i32) -> i32 {
        info!("i am the init function from main");
        let count_old = COUNT_NUM.swap(a, Ordering::SeqCst);
        println!(
            "Init called from UService library with old value of {0}",
            count_old
        );

        count_old
    }

    extern "C" fn process_me(a: i32) -> i32 {
        // info!("i am the process function from main");
        let _count_old = COUNT_NUM.fetch_add(1, Ordering::SeqCst);
        // println!("Process called from UService library with values set to {0}", count_old+a);

        fibonacci_fast(a.try_into().unwrap()).try_into().unwrap()
    }

    fn process_wrap(a: i32) -> i32 {
        process_me(a)
    }

    register_service(init_me, process_me);

    let mut group = c.benchmark_group("Process Calls");
    for i in [20i32, 21i32, 22i32].iter() {
        // Direct call of C API
        group.bench_with_input(BenchmarkId::new("Direct", i), i, |b, i| {
            b.iter(|| process_me(*i))
        });

        // Call process on uservice
        group.bench_with_input(BenchmarkId::new("FFI", i), i, |b, i| b.iter(|| process(*i)));

        // Call wrapped C API (representing similar to uservice as it has function as indirection)
        group.bench_with_input(BenchmarkId::new("Wrapped", i), i, |b, i| {
            b.iter(|| process_wrap(*i))
        });
    }
    group.finish();
}

criterion_group!(benches, ffi_calls);
criterion_main!(benches);
