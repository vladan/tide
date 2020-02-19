mod test_utils;
use async_std::prelude::*;
use async_std::task;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tide::{Body, Request, Response, StatusCode};

#[test]
fn hello_world() -> Result<(), http_types::Error> {
    task::block_on(async {
        let port = test_utils::find_port().await;
        let server = task::spawn(async move {
            let mut app = tide::new();
            app.at("/").get(move |mut req: Request<()>| async move {
                assert_eq!(req.body_string().await.unwrap(), "nori".to_string());
                assert!(req.local_addr().unwrap().contains(&port.to_string()));
                assert!(req.peer_addr().is_some());
                let mut res = Response::new(StatusCode::Ok);
                res.set_body("says hello");
                Ok(res)
            });
            app.listen(("localhost", port)).await?;
            Result::<(), http_types::Error>::Ok(())
        });

        let client = task::spawn(async move {
            task::sleep(Duration::from_millis(100)).await;
            let string = surf::get(format!("http://localhost:{}", port))
                .body_string("nori".to_string())
                .recv_string()
                .await
                .unwrap();
            assert_eq!(string, "says hello".to_string());
            Ok(())
        });

        server.race(client).await
    })
}

#[test]
fn hello_world_multipath() -> Result<(), http_types::Error> {
    task::block_on(async {
        let server = task::spawn(async {
            let mut app = tide::new();

            async fn greet(req: tide::Request<()>) -> Result<Response, http_types::Error> {
                let name = req.param("name").unwrap_or("nori".to_owned());
                Ok(tide::Response::new(StatusCode::Ok).body_string(format!("{} says hello", name)))
            }

            app.at("/").get(greet);
            app.at("/:name").get(greet);
            app.listen("localhost:8181").await?;
            Result::<(), http_types::Error>::Ok(())
        });

        let client = task::spawn(async {
            task::sleep(Duration::from_millis(200)).await;
            let res = surf::get("http://localhost:8181/").recv_string().await?;
            assert_eq!(res, "nori says hello".to_string());

            let res = surf::get("http://localhost:8181/nor").recv_string().await?;
            assert_eq!(res, "nor says hello".to_string());
            Ok(())
        });

        server.race(client).await
    })
}

#[test]
fn echo_server() -> Result<(), http_types::Error> {
    task::block_on(async {
        let port = test_utils::find_port().await;
        let server = task::spawn(async move {
            let mut app = tide::new();
            app.at("/").get(|req| async move { Ok(req) });

            app.listen(("localhost", port)).await?;
            Result::<(), http_types::Error>::Ok(())
        });

        let client = task::spawn(async move {
            task::sleep(Duration::from_millis(100)).await;
            let string = surf::get(format!("http://localhost:{}", port))
                .body_string("chashu".to_string())
                .recv_string()
                .await
                .unwrap();
            assert_eq!(string, "chashu".to_string());
            Ok(())
        });

        server.race(client).await
    })
}

#[test]
fn json() -> Result<(), http_types::Error> {
    #[derive(Deserialize, Serialize)]
    struct Counter {
        count: usize,
    }

    task::block_on(async {
        let port = test_utils::find_port().await;
        let server = task::spawn(async move {
            let mut app = tide::new();
            app.at("/").get(|mut req: Request<()>| async move {
                let mut counter: Counter = req.body_json().await.unwrap();
                assert_eq!(counter.count, 0);
                counter.count = 1;
                let mut res = Response::new(StatusCode::Ok);
                res.set_body(Body::from_json(&counter)?);
                Ok(res)
            });
            app.listen(("localhost", port)).await?;
            Result::<(), http_types::Error>::Ok(())
        });

        let client = task::spawn(async move {
            task::sleep(Duration::from_millis(100)).await;
            let counter: Counter = surf::get(format!("http://localhost:{}", &port))
                .body_json(&Counter { count: 0 })?
                .recv_json()
                .await
                .unwrap();
            assert_eq!(counter.count, 1);
            Ok(())
        });

        server.race(client).await
    })
}
