use rust_multithread_server::{Request, Response, ThreadPool};
use std::{
    fs,
    net::{TcpListener, TcpStream},
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(6);

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }

    println!("Shutting down.");
}

fn handle_connection(mut stream: TcpStream) {
    let req = Request::new(&stream);
    let mut res = Response::new();

    match req.path.as_str() {
        "/" => {
            let contents = fs::read_to_string("src/views/index.html").unwrap();
            res.set_status_code(200);
            res.add_header("Content-Type".to_string(), "text/html".to_string());
            res.set_body(contents.as_bytes().to_vec());
        }
        "/about" => {
            let contents = fs::read_to_string("src/views/about.html").unwrap();
            res.set_status_code(200);
            res.add_header("Content-Type".to_string(), "text/html".to_string());
            res.set_body(contents.as_bytes().to_vec());
        }
        _ => {
            let contents = fs::read_to_string("src/views/404.html").unwrap();
            res.set_status_code(404);
            res.add_header("Content-Type".to_string(), "text/html".to_string());
            res.set_body(contents.as_bytes().to_vec());
        }
    }

    res.send(&mut stream).unwrap();
}
