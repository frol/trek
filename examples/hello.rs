#[macro_use]
extern crate log;

use serde::{Deserialize, Serialize};

use futures::future::BoxFuture;
use trek::{into_box_dyn_handler, json, Context, Middleware, Resources, Response, Trek};

struct MiddlewareA {}
struct MiddlewareB {}

impl<State: Sync + Send + 'static> Middleware<Context<State>> for MiddlewareA {
    fn call<'a>(&'a self, cx: Context<State>) -> BoxFuture<'a, Response> {
        Box::pin(async move {
            info!("Middleware A: {}", "In");
            let res = cx.next().await;
            info!("Middleware A: {}", "Out");
            res
        })
    }
}

impl<State: Sync + Send + 'static> Middleware<Context<State>> for MiddlewareB {
    fn call<'a>(&'a self, cx: Context<State>) -> BoxFuture<'a, Response> {
        Box::pin(async move {
            info!("Middleware B: {}", "In");
            let res = cx.next().await;
            info!("Middleware B: {}", "Out");
            res
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct UserInfo {
    name: String,
    repo: String,
    id: u64,
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let mut app = Trek::new();

    app.router()
        .middleware(MiddlewareA {})
        .middleware(MiddlewareB {})
        .get("/", |_| async { "hello" })
        .get("/rust", |_| async { "rust" })
        .get("/2018", |_| async { "2018" })
        .get("/users/:name", |cx: Context<()>| {
            async move { cx.params::<String>().unwrap() }
        })
        .resources(
            "/users",
            &[
                (
                    Resources::Show,
                    into_box_dyn_handler(|_| async { "users show" }),
                ),
                (
                    Resources::Edit,
                    into_box_dyn_handler(|_| async { "users edit" }),
                ),
            ],
        )
        .get("/users/:name/repos/:repo/issues/:id", |cx: Context<()>| {
            async move { json(&cx.params::<UserInfo>().unwrap()) }
        });

    if let Err(e) = app.run("127.0.0.1:8000").await {
        error!("Error: {}", e);
    }
}
