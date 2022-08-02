mod crab;
use crab::{render, App, Response};

fn main() {
    let mut app = App::new();
    app.get("/", |req| -> Response { render("login") });
    app.start_server(8080, || -> () {
        println!("Starting server at PORT:{}", 8080);
    });
}
