#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;

use rocket::response::{self, Responder, Response};
use rocket::serde::json::{Json, Value};
use rocket::serde::{Deserialize, Serialize};
use rocket::{Request, State};

use std::fs::{self, File};
use std::io::Write;

#[catch(404)]
fn not_found(req: &Request) -> String {
    format!("could not find '{}'", req.uri())
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
struct Recipes {
    name: String,
    ingredients: Vec<String>,
    instructions: Vec<String>,
}

impl std::fmt::Display for Recipes {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Serialize, Deserialize, Debug)]

struct RecipeNames {
    recipeNames: Vec<String>,
}

impl std::fmt::Display for RecipeNames {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.recipeNames)
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Details {
    ingredients: Vec<String>,
    numSteps: u32,
}

impl<'r> Responder<'r, 'static> for Details {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        Response::build().ok()
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

impl<'r> Responder<'r, 'static> for RecipeNames {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        Response::build().ok()
    }
}

#[get("/recipes")]
fn recipe_names(json: &State<Value>) -> Result<Value, String> {
    let mut all_recipe_names = Vec::new();
    let recipes = json.get("recipes").expect("could not find recipes");
    let recipe = recipes.to_string();
    let data: Vec<Recipes> = serde_json::from_str(&recipe).unwrap_or_default();
    for ele in data.iter() {
        all_recipe_names.push(&ele.name);
    }
    let result: serde_json::Value = serde_json::json!( {
        "recipeNames": all_recipe_names,
    });
    Ok(result)
}

#[get("/recipes/details/<name>")]
fn get_recipe_details(json: &State<Value>, name: &str) -> Result<Value, String> {
    let mut result: serde_json::Value = serde_json::json!({});
    let recipes = json.get("recipes").expect("could not find recipes");
    let recipe = recipes.to_string();
    let data: Vec<Recipes> = serde_json::from_str(&recipe).unwrap();
    for ele in data.iter() {
        if ele.name.to_string() == name {
            let details: serde_json::Value = serde_json::json!({
              "ingredients": ele.ingredients,
              "numSteps": ele.instructions.len()
            });
            result = serde_json::json!({ "details": details });
            break;
        } else {
            result = serde_json::json!({});
        }
    }
    Ok(result)
}

// TODO: preserve formatting; 
// TODO: if recipe exists, do not add
#[post("/recipes", format = "json", data = "<item>")]
fn add_recipe(json: &State<Value>, item: Json<Recipes>) -> Option<()> {
    let mut file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .append(false)
        .create(false)
        .open("static/data.json")
        .expect("unable to open");
    let mut all_recipe_names = Vec::new();
    let recipes = json.get("recipes")?;
    let recipe = recipes.to_string();
    let mut all_recipes: Vec<Recipes> = serde_json::from_str(&recipe).unwrap_or_default();
    for ele in all_recipes.iter() {
        all_recipe_names.push(&ele.name);
    }
    if !all_recipe_names.contains(&&item.name) {
        let new_recipe = item.into_inner();
        all_recipes.push(new_recipe);
        let result = serde_json::json!({ "recipes": all_recipes });
        serde_json::to_writer_pretty(&mut file, &result).unwrap_or_default();
        file.flush().unwrap_or_default();
        Some(())
    } else {
        None
    }
}

#[put("/recipes")]
fn edit_recipe() {

}

#[launch]
fn rocket() -> _ {
    let rdr = File::open("static/data.json").expect("Failed to open data.json");
    let json: Value =
        serde_json::from_reader(rdr).expect("Failed to convert rdr into serde_json::Value");
    rocket::build()
        .manage(json)
        .register("/", catchers![not_found])
        .mount(
            "/",
            routes![
                index,
                recipe_names,
                all_recipes,
                get_recipe_details,
                add_recipe,
                edit_recipe
            ],
        )
}
