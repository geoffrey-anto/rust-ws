use std::fs;

use rust_multithread_server::{Request, Response};

pub struct Routes {}

impl Routes {
    pub fn home_handler(_req: Request, mut res: Response) -> Response {
        let contents = fs::read_to_string("src/views/index.html").unwrap();
        res.set_status_code(200);
        res.add_header("Content-Type".to_string(), "text/html".to_string());
        res.set_body(contents.as_bytes().to_vec());
        res
    }

    pub fn about_handler(_req: Request, mut res: Response) -> Response {
        let contents = fs::read_to_string("src/views/about.html").unwrap();
        res.set_status_code(200);
        res.add_header("Content-Type".to_string(), "text/html".to_string());
        res.set_body(contents.as_bytes().to_vec());
        res
    }

    pub fn not_found_handler(_req: Request, mut res: Response) -> Response {
        let contents = fs::read_to_string("src/views/404.html").unwrap();
        res.set_status_code(404);
        res.add_header("Content-Type".to_string(), "text/html".to_string());
        res.set_body(contents.as_bytes().to_vec());
        res
    }
}
