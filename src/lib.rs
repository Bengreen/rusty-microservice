use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use warp::Filter;

pub fn greeting(name: &str) -> String {
    format!("Hello {}!", name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn greeting_contains_name() {
        let result = greeting("Carol");
        assert!(result.contains("Carol"));
    }
}

pub fn simple_listen() {
    println!("Starting simple listen");
    let listener = TcpListener::bind("0.0.0.0:7878").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        println!("Connection established");
        handle_connection(stream);
    }
}

pub fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];

    stream.read(&mut buffer).unwrap();

    println!("Request: {}", String::from_utf8_lossy(&buffer[..]));
    let contents = "Hello";

    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
        contents.len(),
        contents
    );

    println!("{}", response);

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

pub async fn warp_listen() {
    println!("start my warp");

    let k8s_alive = warp::path!("alive").map(|| {
        println!("Requesting for alive");
        format!("Alive")
    });
    let k8s_ready = warp::path!("ready").map(|| {
        println!("Requesting for ready");
        format!("Ready")
    });

    let hello = warp::path!("hello" / String).map(|name| {
        println!("got here for {}", name);
        format!("Hello, {}!", name)
    });

    let routes = warp::get().and(hello.or(k8s_alive).or(k8s_ready));

    warp::serve(routes).run(([0, 0, 0, 0], 7878)).await;
}

pub fn tokio_start() {
    println!("start my tokio");

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("build runtime");

    let local = tokio::task::LocalSet::new();
    local.block_on(&rt, warp_listen());
}
