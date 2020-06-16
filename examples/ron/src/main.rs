#![feature(proc_macro_hygiene)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate serde_derive;

#[cfg(test)] mod tests;

use std::sync::Mutex;
use std::collections::HashMap;

use rocket::State;
use rocket_contrib::ron::{Ron};

// The type to represent the ID of a message.
type ID = usize;

// We're going to store all of the messages here. No need for a DB.
type MessageMap = Mutex<HashMap<ID, String>>;

#[derive(Serialize, Deserialize)]
struct Message {
    id: Option<ID>,
    contents: String
}

#[derive(Serialize, Deserialize)]
struct ErrorMessage {
    status: String,
    reason: Option<String>
}

// TODO: This example can be improved by using `route` with multiple HTTP verbs.
#[post("/<id>", data = "<message>")]
fn new(id: ID, message: Ron<Message>, map: State<'_, MessageMap>) -> Ron<ErrorMessage> {
    let mut hashmap = map.lock().expect("map lock.");
    if hashmap.contains_key(&id) {
       Ron(ErrorMessage{status: "error".to_owned(), reason: Some("ID exists. Try put.".to_owned())})
    } else {
        hashmap.insert(id, message.0.contents);
        Ron(ErrorMessage{status: "ok".to_owned(), reason: None})
    }
}

#[put("/<id>", data = "<message>")]
fn update(id: ID, message: Ron<Message>, map: State<'_, MessageMap>) -> Option<Ron<ErrorMessage>> {
    let mut hashmap = map.lock().unwrap();
    if hashmap.contains_key(&id) {
        hashmap.insert(id, message.0.contents);
        Some(Ron(ErrorMessage{status: "ok".to_owned(), reason: None}))
    } else {
        None
    }
}

#[get("/<id>")]
fn get(id: ID, map: State<'_, MessageMap>) -> Option<Ron<Message>> {
    let hashmap = map.lock().unwrap();
    hashmap.get(&id).map(|contents| {
        Ron(Message {
            id: Some(id),
            contents: contents.clone()
        })
    })
}

#[catch(404)]
fn not_found() -> Ron<ErrorMessage> {
    Ron(ErrorMessage {
        status: "error".to_owned(),
        reason: Some("Resource was not found.".to_owned())
    })
}

fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount("/message", routes![new, update, get])
        .register(catchers![not_found])
        .manage(Mutex::new(HashMap::<ID, String>::new()))
}

fn main() {
    let _ = rocket().launch();
}
