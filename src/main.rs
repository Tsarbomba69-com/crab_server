mod crab;
use crab::App;

fn main() {
    let mut app: App = App::new();
    app.get("/", |req, res| -> () {});
    app.start_server(8080, || -> () {
        println!("Starting:{}", 8080);
    });
}
