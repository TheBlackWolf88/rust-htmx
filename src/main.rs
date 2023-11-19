use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, patch, post},
    Form, Router,
};
use axum_macros::debug_handler;

use dotenv::dotenv;
use maud::{html, Markup, DOCTYPE};
use serde::Deserialize;
use sqlx::{MySqlPool, Result};

#[derive(Deserialize, Debug)]
struct TodoFormItem {
    todo: String,
}

#[derive(Deserialize, Debug)]
struct TodoDbItem {
    id: i32,
    todo: String,
    is_complete: i8,
}

async fn index(State(pool): State<Arc<AppState>>) -> Markup {
    let todos = load_db(pool.db.clone()).await;
    html! {
             (DOCTYPE)
             head {
                 script src="https://unpkg.com/htmx.org@1.9.6"{}
                 script src="https://unpkg.com/hyperscript.org@0.9.12"{}
                 link rel="shortcut icon" href="#";
             }
             body {
                 div {
                     div {
                         form hx-post="/add_todo" hx-target="#todos" hx-swap="beforeend" _="on htmx:afterRequest reset() me" {
                             input placeholder="What you doin'?" name="todo"{}
                             br;
                             button {
                                 "Add Todo"
                             }
                         }
                     }
                     h1 { "Todos: " }
                     ul id="todos"{
                         @for todo in &todos{
                             li id=(todo.id){
                                 @if todo.is_complete == 0 {
                                     input type="checkbox" value=(todo.is_complete) hx-swap="none" hx-patch=(format!("/todo/{}", todo.id)){}
                                 }
                                 @else {
                                     input type="checkbox" value=(todo.is_complete) hx-swap="none" hx-patch=(format!("/todo/{}", todo.id)) checked{}
                                 }
                                 span {(todo.todo)}
                                 button hx-delete=(format!("/todo/{}", todo.id)) hx-target="closest li" hx-swap="outerHTML" {"x"}
                             }
                         }
                     }
                 }
             }
         }
}

async fn load_db(pool: MySqlPool) -> Vec<TodoDbItem> {
    let res = sqlx::query_as!(TodoDbItem, "SELECT * FROM todos")
        .fetch_all(&pool)
        .await
        .ok()
        .unwrap();
    return res;
}

#[debug_handler]
async fn add_todo(State(pool): State<Arc<AppState>>, Form(payload): Form<TodoFormItem>) -> Markup {
    let res = sqlx::query("insert into todos (todo) values (?)")
        .bind(payload.todo.clone())
        .execute(&pool.db)
        .await
        .ok()
        .unwrap()
        .last_insert_id();
    return html! {
        li id=(res){
            input type="checkbox" value="0" hx-swap="none" hx-patch=(format!("/todo/{}", res)){}
            span {(payload.todo)}
            button hx-delete=(format!("/todo/{}", res)) hx-target="closest li" hx-swap="outerHTML" {"x"}
        }
    };
}

#[debug_handler]
async fn update_todo(State(pool): State<Arc<AppState>>, Path(id): Path<i32>) -> Markup {
    let item = sqlx::query_as!(TodoDbItem, "select * from todos where id=?", id.to_string())
        .fetch_one(&pool.db)
        .await;
    let new_state = !item.ok().unwrap().is_complete;
    println!("{}", new_state);
    let _ = sqlx::query_as!(
        TodoDbItem,
        "update todos set is_complete=? where id=?",
        new_state,
        id
    )
    .execute(&pool.db)
    .await;

    return html!();
}

#[debug_handler]
async fn delete_todo(State(pool): State<Arc<AppState>>, Path(id): Path<i32>) -> StatusCode {
    let _ = sqlx::query_as!(TodoDbItem, "delete from todos where id=?", id)
        .execute(&pool.db)
        .await;
    return StatusCode::OK;
}

pub struct AppState {
    db: MySqlPool,
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").expect("db url not set!!!");
    let pool = MySqlPool::connect(&db_url).await.unwrap();
    let app = Router::new()
        .route("/", get(index))
        .route("/add_todo", post(add_todo))
        .route("/todo/:id", patch(update_todo))
        .route("/todo/:id", delete(delete_todo))
        .with_state(Arc::new(AppState { db: pool }));
    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}
