use std::{
    io::{prelude::*, BufReader},
    net::TcpStream,
    sync::{mpsc, Arc, Mutex},
    thread,
};

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv();

            match message {
                Ok(job) => {
                    println!("Worker {id} got a job; executing.");

                    job();
                }
                Err(_) => {
                    println!("Worker {id} disconnected; shutting down.");
                    break;
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

type Header = (String, String);

pub struct Request {
    pub method: String,
    pub path: String,
    pub headers: Vec<Header>,
}

impl Request {
    pub fn new(mut stream: &TcpStream) -> Request {
        let buf_reader = BufReader::new(&mut stream);

        let request_lines: Vec<String> = buf_reader
            .lines()
            .map(|x| x.unwrap())
            .take_while(|line| !line.is_empty())
            .collect();

        let first_line = request_lines.first().unwrap();
        let parts: Vec<&str> = first_line.split_whitespace().collect();

        let method = parts[0].to_string();
        let path = parts[1].to_string();

        let headers: Vec<Header> = request_lines
            .iter()
            .skip(1)
            .map(|line| {
                let parts: Vec<&str> = line.split(":").collect();
                (parts[0].to_string(), parts[1].trim().to_string())
            })
            .collect();

        Request {
            method,
            path,
            headers,
        }
    }
}

pub struct Response {
    status_code: u16,
    headers: Vec<Header>,
    body: Vec<u8>,
}

impl Response {
    pub fn new() -> Response {
        Response {
            status_code: 200,
            headers: Vec::new(),
            body: Vec::new(),
        }
    }

    pub fn set_status_code(&mut self, status_code: u16) {
        self.status_code = status_code;
    }

    pub fn add_header(&mut self, name: String, value: String) {
        self.headers.push((name, value));
    }

    pub fn set_body(&mut self, body: Vec<u8>) {
        self.body = body;
    }

    pub fn send(&self, stream: &mut TcpStream) -> std::io::Result<()> {
        let status_line = format!("HTTP/1.1 {}\r\n", self.status_code);
        stream.write(status_line.as_bytes())?;

        for (name, value) in &self.headers {
            let header_line = format!("{}: {}\r\n", name, value);
            stream.write(header_line.as_bytes())?;
        }

        let content_length = format!("Content-Length: {}\r\n", self.body.len());
        // Content-Length is a required header for HTTP/1.1
        stream.write(format!("Content-Length{}\r\n", content_length).as_bytes())?;

        stream.write(b"\r\n")?;
        stream.write(&self.body)?;

        Ok(())
    }
}

pub fn handler(mut stream: TcpStream, callback: impl FnOnce(Request, Response) -> Response) {
    let request = Request::new(&stream);

    let response = Response::new();

    let response = callback(request, response);

    response.send(&mut stream).unwrap()
}
