mod Crab;
use Crab::{render, App, Response};

fn main() {
    let mut app = App::new();
    app.get("/", |req| -> Response { render("login") });
    app._start_server(8080, || -> () {
        println!("Starting server at PORT:{}", 8080);
    });
}
