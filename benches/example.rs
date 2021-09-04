use criterion::{black_box, criterion_group, criterion_main, Criterion};

use rustyhello::{UServiceConfig, UService, start_async, send_http_kill};
use rustyhello::k8slifecycle::{HealthCheck};

fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 1,
        1 => 1,
        n => fibonacci(n-1) + fibonacci(n-2),
    }
}

// pub fn criterion_benchmark(c: &mut Criterion) {
//     c.bench_function("fib 20", |b| b.iter(|| fibonacci(black_box(20))));
// }



pub fn http_benchmark(c: &mut Criterion) {

    use warp::hyper::Client;
    use warp::hyper;
    use warp::hyper::http::StatusCode;
    use std::{thread, time};




    println!("Loading uService");


    // Spin service up in its own thread
    let thandle = std::thread::spawn(move || {

        let rt_u = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Runtime created in current thread");

        let _guard = rt_u.enter();
        rt_u.block_on(async {

            let local = tokio::task::LocalSet::new();

            local.spawn_local( async {
                let config = UServiceConfig {
                    name: String::from("test0"),
                };
                let uservice = UService::new(&config.name);
                let liveness = HealthCheck::new("liveness");
                let readyness = HealthCheck::new("readyness");
                start_async(&uservice, &liveness, &readyness).await;
            });

            local.await;
            println!("Finished benchmark");
        });
        println!("UService thread completed");
    });

    // Wait for 10 secs in case you need to enable access on your computer
    thread::sleep(time::Duration::from_secs(10));

    let rt_b = tokio::runtime::Runtime::new().unwrap();

    let client = Client::new();
    let uri: hyper::Uri = "http://localhost:7979/health/alive".parse().unwrap();
    c.bench_function("http alive", |b| {
        b.to_async(&rt_b).iter(|| async {
            let resp = client.get(uri.clone()).await.unwrap();
            assert!(resp.status() == StatusCode::OK);
        });
    });

    let uri: hyper::Uri = "http://localhost:8080/sample/sample1".parse().unwrap();
    c.bench_function("http sample", |b| {
        b.to_async(&rt_b).iter(|| async {
            let resp = client.get(uri.clone()).await.unwrap();
            assert!(resp.status() == StatusCode::OK);
        });
    });

    c.bench_function("http sample concurrent", |b| {
        let concurrency  = 3;

        b.to_async(&rt_b).iter(|| async {
            let mut parallel = Vec::new();
            for _i in 0..concurrency {
                parallel.push(async {
                        let resp = client.get(uri.clone()).await.unwrap();
                        resp.status()
                    });
            }
            futures::future::join_all(parallel).await;
        });
    });

    thread::sleep(time::Duration::from_secs(3));


    rt_b.block_on(async {
        send_http_kill().await;
    });

    thandle.join().expect("UService thread complete");

}


criterion_group!(benches,
    // criterion_benchmark,
     http_benchmark);
criterion_main!(benches);
