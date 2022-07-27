use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{Error, Write};
use std::path::Path;
use std::{
    io::Read,
    net::{SocketAddr, TcpListener, TcpStream},
};

pub struct App {
    routes: HashMap<String, fn(req: Request) -> Response>,
}

#[derive(Debug)]
pub struct Request<'a> {
    method: &'a str,
    uri: &'a str,
    headers: HashMap<String, String>,
    content_type: &'a str,
    http_version: &'a str,
    body: HashMap<String, String>,
    ip: &'a str,
}

#[derive(Debug)]
pub struct Response<'a> {
    status_code: u8,
    reason_phrase: &'a str,
    headers: HashMap<String, String>,
    content_type: &'a str,
    content_length: usize,
    contents: String,
}

pub fn render(view: &str) -> Response {
    let filename = format!("src\\static\\HTML\\{}.html", view);
    let current_dir: &Path = Path::new(&filename);
    let path = env::current_dir().unwrap().join(current_dir);
    let html = fs::read_to_string(path).expect("Something went wrong reading the file");

    let res: Response = Response {
        status_code: 200,
        reason_phrase: "Ok",
        headers: HashMap::new(),
        content_type: "text/plain",
        content_length: html.len(),
        contents: html,
    };

    res
}

impl App {
    pub fn new() -> App {
        App {
            routes: HashMap::new(),
        }
    }

    fn handle_connection(&self, mut stream: TcpStream) {
        let mut buffer = [0; 1024];
        let mut request_line: String = String::new();
        let mut body: String = String::new();
        let mut headers: HashMap<String, String> = HashMap::new();

        match stream.read(&mut buffer) {
            Ok(length) => println!("Content-Length: {}", length),
            Err(e) => println!("Error: {}", e),
        };

        (headers, request_line, body) = App::parse_request(&buffer);
        let vec: Vec<&str> = request_line.split(" ").collect();

        let req: Request = Request {
            method: vec[0].clone(),
            uri: vec[1].clone(),
            http_version: vec[2].clone(),
            headers: headers.clone(),
            body: App::parse_body(body),
            content_type: "",
            ip: "127.0.0.1",
        };

        let k = format!("{} {}", req.method, req.uri);
        // TODO: handle serving static files 
        match self.routes.get(k.as_str()) {
            None => println!("Not found!"),
            Some(callback) => {
                let res = callback(req);
                let res_buffer = format!(
                    "{} {} {}\r\nRequest: Content-Length: {}\r\n\r\n{}",
                    vec[2],
                    res.status_code,
                    res.reason_phrase,
                    res.content_length,
                    res.contents.as_str()
                );

                match stream.write_all(res_buffer.as_bytes()) {
                    Ok(()) => println!(
                        "Response: {} {} {} {}",
                        vec[0], vec[1], res.status_code, res.reason_phrase
                    ),
                    Err(e) => println!("Error: {}", e),
                };
            }
        }

        stream.flush();
    }

    fn parse_request(buffer: &[u8]) -> (HashMap<String, String>, String, String) {
        let request_msg = String::from_utf8(buffer.to_vec()).unwrap();
        let mut headers: HashMap<String, String> = HashMap::new();
        let request_line = request_msg.lines().nth(0).unwrap().to_string();
        let index: usize = request_msg.find("\r\n\r\n").unwrap_or(0);

        let body = request_msg
            .get(index..request_msg.len())
            .unwrap_or_default();

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

    fn parse_body(body: String) -> HashMap<String, String> {
        let mut _body: HashMap<String, String> = HashMap::new();
        _body.insert(String::from("empty"), body);
        _body
    }

    fn generate_response() {
        todo!();
    }

    pub fn start_server(&self, port: u16, callback: fn() -> ()) -> () {
        let listener: Result<TcpListener, Error> =
            TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], port)));

        match listener {
            Ok(listener) => {
                callback();

                for stream in listener.incoming() {
                    match stream {
                        Ok(stream) => self.handle_connection(stream),
                        Err(e) => println!("Error: {}", e),
                    }
                }
            }
            Err(e) => println!("ERROR: {:?}", e),
        }
    }

    pub fn get(&mut self, uri: &str, callback: fn(req: Request) -> Response) -> () {
        self.routes.insert(format!("GET {}", uri), callback);
    }
}
