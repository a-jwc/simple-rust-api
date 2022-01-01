#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;

use rocket::response::{self, Responder, Response};
use rocket::serde::{Deserialize, Serialize};
use rocket::{Request, State};
use serde_json::{from_value, Value};
use std::fs::File;

#[derive(Serialize, Deserialize)]
struct Recipes {
    name: String,
    ingredients: Vec<String>,
    instructions: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct RecipeNames {
    title: String,
    names: Vec<String>,
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

impl<'r> Responder<'r, 'static> for RecipeNames {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        Response::build().ok()
    }
}

#[get("/recipes")]
fn recipe_names(json: &State<Value>) -> Option<RecipeNames> {
    let key = "name".to_string();
    let mut recipeNames: Vec<String> = vec![];
    if let Some(recipes) = json.get("recipes") {
        println!("recipes {:#?}", recipes);
        // while let Some(value) = recipes[0].get(&key) {
        //     println!("value {}", value);
        //     // Some(String::from(
        //     //     value.as_str().expect("Failed to convert value"),
        //     // ))
        //     if !recipeNames.contains(&value.to_string()) {
        //         recipeNames.push(value.to_string());
        //     }
        // }
        for ele in from_value(*recipes) {
            // if ele == "name".to_string() {
            //     recipeNames.push(ele);
            // }
            println!("{:#?}", ele);
        }
        let result = RecipeNames {
            title: "recipeNames".to_string(),
            names: recipeNames,
        };
        Some(result)
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
        .mount("/", routes![index, recipe_names, all_recipes])
}
