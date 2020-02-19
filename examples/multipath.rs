use async_std::task;
use tide::Request;
use http_types::Result;

async fn greet(req: Request<()>) -> Result<String> {
    let name = req.param("name").unwrap_or("world".to_owned());
    Ok(format!("Hello, {}!", name))
}

fn main() -> Result<()> {
    task::block_on(async {
        let mut app = tide::new();
        app.at("/hello").get(greet);
        app.at("/hello/:name").get(greet);
        app.listen("127.0.0.1:8080").await?;
        Ok(())
    })
}
