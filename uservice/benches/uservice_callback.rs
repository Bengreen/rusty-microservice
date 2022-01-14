use criterion::{
    Criterion,
    BenchmarkId,
    criterion_group,
    criterion_main,
};

use log::info;
use tokio;

// use uservice::uservice::{start, UServiceConfig};
use uservice::{register_service, serviceStart, serviceStop};


async fn do_something(size: usize) {
    // Do something async with the size

    info!("hello {}", size);
}


fn uservice_callback(c: &mut Criterion) {
    use std::{thread, time};
    use warp::hyper;
    use warp::hyper::http::StatusCode;
    use warp::hyper::Client;

    use uservice::process;

    // pub async fn send_http_kill<C>(client: &Client<C>) {
    pub async fn send_http_kill() {
        let client = Client::new();
        let uri = "http://localhost:7979/health/kill".parse().unwrap();
        let resp = client.get(uri).await.unwrap();
        info!("Kill Response: {}", resp.status());
    }


    use std::sync::atomic::{AtomicI32, Ordering};

    static COUNT_NUM: AtomicI32 = AtomicI32::new(0);

    extern "C" fn init_me(a: i32) -> i32 {
        info!("i am the init function from main");
        let count_old = COUNT_NUM.swap(a, Ordering::SeqCst);
        println!("Init called from UService library with old value of {0}", count_old);

        count_old
    }
    extern "C" fn process_me(a: i32) -> i32 {
        // info!("i am the process function from main");
        let count_old = COUNT_NUM.fetch_add(a, Ordering::SeqCst);
        // println!("Process called from UService library with values set to {0}", count_old+a);

        count_old + a
    }

    fn process_wrap(a: i32) -> i32 {
        process_me(a)
    }

    let thandle = std::thread::spawn(move || {

        // Write an uninit function that is called at unregister stage. read back teh global count from this

        register_service(init_me, process_me);

        println!("Registered init and process");

        println!("Loaded background thread for running server");
        // let config = UServiceConfig {
        //     name: String::from("test0"),
        // };
        serviceStart();
        println!("Server thread stopped");

        //Unregister callbacks

        let count_last = COUNT_NUM.load(Ordering::SeqCst);
        println!("Count of {}", count_last);
    });


    let size: usize = 1024;

    let rt_b = tokio::runtime::Runtime::new().unwrap();
    let client = Client::new();
    let uri: hyper::Uri = "http://localhost:7979/health/alive".parse().unwrap();


    c.bench_with_input(BenchmarkId::new("http", size),&size,  |b, &s| {
        b.to_async(&rt_b).iter(|| async {
            let resp = client.get(uri.clone()).await.unwrap();
            assert!(resp.status() == StatusCode::OK);
            do_something(s).await;
        });
    });

    let mut group = c.benchmark_group("Callbacks");
    for i in [20i32].iter() {

        // Direct call of C API
        group.bench_with_input(BenchmarkId::new("Direct", i), i,
            |b, i| b.iter(|| process_me(*i)));

        // Call process on uservice
        group.bench_with_input(BenchmarkId::new("UService", i), i,
            |b, i| b.iter(|| process(*i)));

        // Call wrapped C API (representing similar to uservice as it has function as indirection)
        group.bench_with_input(BenchmarkId::new("Wrapped", i), i,
            |b, i| b.iter(|| process_wrap(*i)));

        }
    group.finish();



    // Shutdown the Service
    thread::sleep(time::Duration::from_secs(3));
    println!("About to stop service");
    serviceStop();
    // rt_b.block_on(async {
    //     // send_http_kill(&client).await;
    //     send_http_kill().await;
    // });
    thandle.join().expect("UService thread complete");

    println!("uService shutdown happily");

}








criterion_group!(benches,  uservice_callback);
criterion_main!(benches);
