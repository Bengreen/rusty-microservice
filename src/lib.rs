use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};

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
