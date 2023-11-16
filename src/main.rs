use std::sync::atomic::AtomicUsize;
use axum::{routing::{get, post}, Router};
use maud::{Markup, html, DOCTYPE};


async fn index() -> Markup {
    html! {
        (DOCTYPE)
        head {
            script src="https://unpkg.com/htmx.org@1.9.6"{}
        }
        body {
            p #counter {
               "0"
            }
            button hx-post="counter" hx-target="#counter" hx-swap="outerHTML"{
                "Add one"
            } 
        }
    }
}

async fn counter() -> Markup {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    return html! {
        p #counter {
            (COUNTER.load(std::sync::atomic::Ordering::Relaxed))
        }
    }
    
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(index))
        .route("/counter", post(counter));
    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
