mod Crab;
use Crab::{render, App, Response};

fn main() {
    let mut app: App = App::new();
    app.get("/", |req| -> Response { render("login") });
    app.start_server(8080, || -> () {
        println!("Starting:{}", 8080);
    });
}
