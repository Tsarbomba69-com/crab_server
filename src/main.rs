mod crab;
use crab::{render, App, Response};

fn main() {
    let mut app = App::new();
    app.get("/", |_req| -> Response { render("login") });
    app.post("/", |req| -> Response {
        println!("headers: {:#?}, \nbody: {:#?}", req.headers, req.body);
        render("Login")
    });
    app.start_server(8080, || -> () {
        println!("Starting server at PORT:{}", 8080);
    });
}
