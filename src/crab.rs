use std::collections::HashMap;
use std::io::Error;
use std::{
    io::Read,
    net::{SocketAddr, TcpListener, TcpStream},
};

pub struct App {
    routes: HashMap<String, fn(req: Request, res: Response) -> ()>,
}

pub struct Request<'a> {
    method: String,
    uri: &'a str,
    headers: HashMap<String, String>,
    content_type: &'a str,
    http_version: &'a str,
    body: HashMap<String, String>,
    ip: &'a str,
}

pub struct Response<'a> {
    status_code: u8,
    headers: HashMap<String, String>,
    content_type: &'a str,
    content_length: u8,
}

impl App {
    pub fn new() -> App {
        App {
            routes: HashMap::new(),
        }
    }

    fn handle_connection(mut stream: TcpStream) {
        let mut buffer = [0; 1024];
        let mut request_line: String = String::new();
        let mut body: String = String::new();
        let mut headers: HashMap<String, String> = HashMap::new();

        match stream.read(&mut buffer) {
            Ok(length) => println!("Content-Length: {}", length),
            Err(e) => println!("Error: {}", e),
        };

        (headers, request_line, body) = App::parse_request(&buffer);
        println!("Request: {}", String::from_utf8_lossy(&buffer[..]));
    }

    fn parse_request(buffer: &[u8]) -> (HashMap<String, String>, String, String) {
        let request_msg = String::from_utf8(buffer.to_vec()).unwrap();
        let mut headers: HashMap<String, String> = HashMap::new();
        let request_line = request_msg.lines().nth(0).unwrap().to_string();
        let index: usize = request_msg.find("\r\n\r\n").unwrap();
        let body = request_msg.get(index..request_msg.len()).unwrap();

        request_msg.lines().for_each(|line| {
            let _line: Vec<&str> = line.split(": ").collect();
            let k = _line.first().copied().unwrap().to_string();
            let v = _line.last().copied().unwrap().to_string();

            if k.is_empty()
                || v.is_empty()
                || k.contains(&request_line)
                || v.contains(&request_line)
                || k.contains("\0")
                || v.contains("\0")
            {
                return;
            }

            headers.insert(k, v);
        });

        (headers, request_line, body.to_string())
    }

    pub fn start_server(&self, port: u16, callback: fn() -> ()) -> () {
        let listener: Result<TcpListener, Error> =
            TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], port)));

        match listener {
            Ok(listener) => {
                callback();

                for stream in listener.incoming() {
                    match stream {
                        Ok(stream) => App::handle_connection(stream),
                        Err(e) => println!("Error: {}", e),
                    }

                    println!("Connection established!");
                }
            }
            Err(e) => println!("ERROR: {:?}", e),
        }
    }

    pub fn get(&mut self, uri: &str, callback: fn(req: Request, res: Response) -> ()) -> () {
        self.routes.insert(uri.to_string(), callback);
    }
}
