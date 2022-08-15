use phf::phf_map;
use phf::Map;
use serde_urlencoded::from_bytes;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
#[derive(Debug, Clone, PartialEq)]
pub struct App {
    routes: HashMap<String, fn(req: Request) -> Response>,
}

static CONTENT_TYPES: Map<&'static str, &str> = phf_map! {
    "css" => "text/css",
    "js" => "application/javascript",
    "png" => "image/png",
    "html" => "text/html",
    "jpg" => "image/jpeg",
};

#[derive(Debug)]
pub struct Request {
    method: String,
    uri: String,
    pub headers: HashMap<String, String>,
    content_type: String,
    http_version: String,
    pub body: HashMap<String, String>,
    hostname: String,
}

#[derive(Debug)]
pub struct Response {
    status_code: usize,
    reason_phrase: String,
    headers: HashMap<String, String>,
    content_type: String,
    content_length: usize,
    contents: Vec<u8>,
}

pub fn render(view: &str) -> Response {
    let filename = format!("static\\HTML\\{}.html", view);
    let current_dir: &Path = Path::new(&filename);
    let path = env::current_dir().unwrap().join(current_dir);
    let html = fs::read_to_string(path).expect("Something went wrong reading the file");
    let headers: HashMap<String, String> =
        vec![("Date".to_string(), chrono::Local::now().to_string())]
            .into_iter()
            .collect();

    Response {
        status_code: 200,
        reason_phrase: String::from("Ok"),
        headers: headers,
        content_type: String::from("text/html; charset=UTF-8"),
        content_length: html.len(),
        contents: html.into_bytes(),
    }
}

// pub fn upload(file_string: &String, uri: &str) {
//     let file_dir = format!("src\\static{}", uri);
//     let current_dir: &Path = Path::new(&file_dir);
//     let path = env::current_dir().unwrap().join(current_dir);
//     if let Ok(mut file) = std::fs::File::create(path) {
//         file.write_all(file_string.as_bytes()).unwrap();
//     }
// }

async fn get_file(uri: String) -> Result<Vec<u8>, tokio::io::Error> {
    let file_dir = format!("src\\static{}", uri);
    let current_dir: &Path = Path::new(&file_dir);
    let path = env::current_dir().unwrap().join(current_dir);
    tokio::fs::read(path).await
}

async fn send(res: Response, stream: &mut TcpStream) -> () {
    let res_buffer = format!(
        "HTTP/1.1 {} {}\r\nContent-Length: {}\r\nContent-Type: {}\r\nDate: {}\r\n\r\n",
        res.status_code,
        res.reason_phrase,
        res.content_length,
        res.content_type,
        res.headers.get(&"Date".to_string()).unwrap()
    );

    match stream.write_all(res_buffer.as_bytes()).await {
        Ok(()) => println!("\nResponse: {} {}", res.status_code, res.reason_phrase),
        Err(e) => println!("Error: {}", e),
    };

    stream.write_all(res.contents.as_slice()).await.unwrap();
}

impl App {
    pub fn get(&mut self, uri: &str, callback: fn(req: Request) -> Response) -> () {
        self.routes.insert(format!("GET {}", uri), callback);
    }

    pub fn post(&mut self, uri: &str, callback: fn(req: Request) -> Response) -> () {
        self.routes.insert(format!("POST {}", uri), callback);
    }

    async fn handle_connection(&self, mut socket: TcpStream) -> () {
        let mut buffer = Vec::with_capacity(8_000_000);
        socket.readable().await.unwrap();
        let size = socket.try_read_buf(&mut buffer).unwrap();
        let (headers, request_line, body) = App::parse_request(&buffer, size);
        let vec: Vec<&str> = request_line.split(" ").collect();

        if vec.len() <= 1 {
            return;
        }

        let req: Request = Request {
            method: vec[0].to_string(),
            uri: vec[1].to_string(),
            http_version: vec[2].to_string(),
            headers: headers.clone(),
            body: App::parse_body(headers.get("Content-Type"), body).await,
            content_type: String::from(""),
            hostname: headers.get("Host").unwrap().clone(),
        };

        let k = format!("{} {}", req.method, req.uri);
        match self.routes.get(&k) {
            None => {
                let file = get_file(req.uri.clone()).await;
                match file {
                    Ok(file) => {
                        let ext = Path::new(req.uri.as_str())
                            .extension()
                            .unwrap()
                            .to_str()
                            .unwrap();
                        let headers: HashMap<String, String> =
                            vec![("Date".to_string(), chrono::Local::now().to_string())]
                                .into_iter()
                                .collect();
                        let res = Response {
                            status_code: 200,
                            reason_phrase: String::from("Ok"),
                            headers: headers,
                            content_type: CONTENT_TYPES.get(ext).unwrap().to_string(),
                            content_length: file.len(),
                            contents: file,
                        };
                        print!("Request: {} {} {}", req.method, req.uri, req.http_version);
                        send(res, &mut socket).await;
                    }
                    Err(_err) => println!("Not found!"),
                }
            }
            Some(callback) => {
                let (method, uri, http_version) = (
                    req.method.clone(),
                    req.uri.clone(),
                    req.http_version.clone(),
                );
                let res = callback(req);
                print!("Request: {} {} {}", method, uri, http_version);
                send(res, &mut socket).await;
            }
        }
    }

    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }

    async fn parse_body(content_type: Option<&String>, body: String) -> HashMap<String, String> {
        match content_type {
            Some(content_type) => {
                let ct = content_type.as_str();

                if ct.contains("application/x-www-form-urlencoded") {
                    let buffer = body.replace("\r\n\r\n", "");
                    let _body = from_bytes::<Vec<(String, String)>>(buffer.as_bytes()).unwrap();
                    return _body.into_iter().collect();
                }

                if ct.contains("multipart/form-data") {
                    let boundary = multer::parse_boundary(ct).unwrap();
                    let mut multipart = multer::Multipart::with_reader(body.as_bytes(), boundary);
                    let mut _body: HashMap<String, String> = HashMap::new();

                    // Iterate over the fields, use `next_field()` to get the next field.
                    while let Some(mut field) = multipart.next_field().await.unwrap() {
                        // Get field name.
                        let name = field.name().unwrap().to_string();

                        //1. Get the field's filename if provided in "Content-Disposition" header.
                        if let Some(file_name) = field.file_name() {
                            //2. If there is a file then create a temp file
                            let ext = Path::new(file_name).extension().unwrap().to_str().unwrap();
                            let time = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_millis();
                            let file_dir = format!("static\\temp\\temp_{}.{}", time, ext);
                            let current_dir: &Path = Path::new(&file_dir);
                            let path = env::current_dir().unwrap().join(current_dir);
                            let mut file = tokio::fs::File::create(path).await.unwrap();
                            //3. Foreach chunk, write to temp file
                            while let Some(chunk) = field.chunk().await.unwrap() {
                                file.write(&chunk[..]).await.unwrap();
                            }
                        } else {
                            //3.1. Create a key-value pair for body hash map
                            while let Some(chunk) = field.chunk().await.unwrap() {
                                _body.insert(
                                    name.clone(),
                                    String::from_utf8(chunk.to_vec()).unwrap(),
                                );
                            }
                        }
                    }

                    return _body;
                }
            }
            None => return HashMap::new(),
        }

        HashMap::new()
    }

    fn parse_request(buffer: &[u8], size: usize) -> (HashMap<String, String>, String, String) {
        let request_msg = String::from_utf8_lossy(&buffer[0..size]);
        let mut headers: HashMap<String, String> = HashMap::new();
        let request_line = request_msg.lines().nth(0).unwrap().to_string();
        let index: usize = request_msg.find("\r\n\r\n").unwrap_or(0);
        let body = request_msg
            .get(index..request_msg.len())
            .unwrap_or_default();
        let headers_section = request_msg.get(0..index).unwrap();
        headers_section.lines().for_each(|line| {
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

    #[tokio::main]
    pub async fn start_server(&self, port: u16, callback: fn() -> ()) {
        callback();
        let listener = TcpListener::bind(format!("127.0.0.1:{}", port))
            .await
            .unwrap();

        loop {
            let (socket, _) = listener.accept().await.unwrap();
            let app = self.clone();

            tokio::spawn(async move {
                app.handle_connection(socket).await;
            });
        }
    }
}
