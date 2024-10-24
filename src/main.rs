use routes::Routes;
use rust_multithread_server::{handler, ThreadPool};
use std::net::TcpListener;
mod routes;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(6);

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| {
            handler(stream, |req, res| match req.path.as_str() {
                "/" => Routes::home_handler(req, res),
                "/about" => Routes::about_handler(req, res),
                _ => Routes::not_found_handler(req, res),
            });
        });
    }

    println!("Shutting down.");
}
