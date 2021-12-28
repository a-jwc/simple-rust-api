#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;

use rocket::request::FromRequest;
use rocket::serde::{Serialize, Deserialize};
use rocket::{Request, State};
use serde_json::Value;
use std::fs::File;

#[derive(Serialize, Deserialize)]
struct Recipes {
    name: String,
    ingredients: Vec<String>,
    instructions: Vec<String>,
}

impl FromRequest<'_> for str {
    type Error = <type>;


    fn from_request< 'life0, 'async_trait>(request: & 'r Request< 'life0>) ->  core::pin::Pin<Box<dyncore::future::Future<Output = rocket::request::Outcome<Self,Self::Error> > + core::marker::Send+ 'async_trait> >where 'r: 'async_trait, 'life0: 'async_trait,Self: 'async_trait {
        todo!()
    }

}

#[get("/")]
fn index() -> &'static str {
    "this is index"
}

#[get("/allRecipes")]
fn all_recipes() -> String {
    let rdr = File::open("static/data.json").expect("Failed to open data.json");
    let recipes: Value =
        serde_json::from_reader(rdr).expect("Failed to convert rdr into serde_json::Value");
    return recipes.to_string();
}

#[get("/recipes")]
fn recipe_names(key: &str, json: &State<Value>) -> Option<String> {
    if let Some(value) = json.get(key) {
        Some(String::from(
            value.as_str().expect("Failed to convert value"),
        ))
    } else {
        None
    }
}

#[catch(404)]
fn not_found(req: &Request) -> String {
    format!("could not find '{}'", req.uri())
}

#[launch]
fn rocket() -> _ {
    let rdr = File::open("static/data.json").expect("Failed to open data.json");
    let json: Value =
        serde_json::from_reader(rdr).expect("Failed to convert rdr into serde_json::Value");
    rocket::build()
        .manage(json)
        .register("/", catchers![not_found])
        .mount("/", routes![index, recipe_names])
}
