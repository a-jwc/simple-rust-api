#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use rocket::http::Status;
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

fn get_recipes_json(json: &State<Value>) -> Result<String, (Status, String)> {
    let recipes = match json.get("recipes") {
        Some(r) => r.to_string(),
        None => {
            return Err((
                Status::BadGateway,
                "Could not find get top-level \"recipes\" property.".to_string(),
            ))
        }
    };
    Ok(recipes)
}

#[get("/")]
fn index() -> &'static str {
    "trunk-web-api"
}

#[get("/allRecipes")]
fn all_recipes() -> String {
    let rdr = File::open("static/data.json").expect("Failed to open data.json");
    let recipes: Value =
        serde_json::from_reader(rdr).expect("Failed to convert rdr into serde_json::Value");
    recipes.to_string()
}

impl<'r> Responder<'r, 'static> for RecipeNames {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        Response::build().ok()
    }
}

#[get("/recipes")]
fn recipe_names(json: &State<Value>) -> Result<Value, (Status, String)> {
    let mut all_recipe_names = Vec::new();
    let recipes = match crate::get_recipes_json(json) {
        Ok(r) => r,
        Err(e) => return Err(e),
    };
    let data: Vec<Recipes> = serde_json::from_str(&recipes).unwrap_or_default();
    for ele in data.iter() {
        all_recipe_names.push(&ele.name);
    }
    let result: serde_json::Value = serde_json::json!( {
        "recipeNames": all_recipe_names,
    });
    Ok(result)
}

#[get("/recipes/details/<name>")]
fn get_recipe_details(json: &State<Value>, name: &str) -> Result<Value, (Status, String)> {
    let mut result: serde_json::Value = serde_json::json!({});
    let recipes = match crate::get_recipes_json(json) {
      Ok(r) => r,
      Err(e) => return Err(e),
  };
    let data: Vec<Recipes> = serde_json::from_str(&recipes).unwrap_or_default();
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
fn add_recipe(json: &State<Value>, item: Json<Recipes>) -> Result<(), String> {
    let mut file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .append(false)
        .create(false)
        .open("static/data.json")
        .expect("unable to open");
    let mut all_recipe_names = Vec::new();
    let recipes = json.get("recipes").expect("Could not find recipes");
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
        Ok(())
    } else {
        Err("Recipe already exists".to_string())
    }
}

#[put("/recipes", format = "json", data = "<item>")]
fn edit_recipe(json: &State<Value>, item: Json<Recipes>) -> Result<(), String> {
    let mut file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .append(false)
        .create(false)
        .open("static/data.json")
        .expect("unable to open");
    let mut all_recipe_names = Vec::new();
    let recipes = json.get("recipes").expect("Could not find recipes");
    let recipe = recipes.to_string();
    let mut all_recipes: Vec<Recipes> = serde_json::from_str(&recipe).unwrap_or_default();
    for ele in all_recipes.iter() {
        all_recipe_names.push(&ele.name);
    }
    if all_recipe_names.contains(&&item.name) {
        all_recipes.retain(|x| x.name != item.name);
        let new_recipe = item.into_inner();
        all_recipes.push(new_recipe);
        let result = serde_json::json!({ "recipes": all_recipes });
        serde_json::to_writer_pretty(&mut file, &result).unwrap_or_default();
        file.flush().unwrap_or_default();
        Ok(())
    } else {
        Err("Recipe does not exist".to_string())
    }
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

#[cfg(test)]
mod test {
    use std::{fs::File, io};

    use super::rocket;
    use rocket::http::{Header, Status};
    use rocket::local::blocking::Client;
    use serde_json::Value;
    mod constants;

    fn get_data_json() -> String {
        let rdr = File::open("static/data.json").expect("Failed to open data.json");
        let recipes: Value =
            serde_json::from_reader(rdr).expect("Failed to convert rdr into serde_json::Value");
        recipes.to_string()
    }

    fn remove_whitespace(s: &str) -> String {
        s.chars().filter(|c| !c.is_whitespace()).collect()
    }

    #[test]
    fn get_index() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let mut response = client.get("/").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_string().unwrap(), "trunk-web-api");
    }

    #[test]
    fn get_recipe_names_200_success() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let mut response = client.get("/recipes").dispatch();
        let recipe_names =
            r#"{"recipeNames":["scrambledEggs","garlicPasta","chai","butteredBagel"]}"#;
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_string().unwrap(), recipe_names);
    }

    #[test]
    fn get_recipe_details_200_success() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let mut response = client.get("/recipes/details/garlicPasta").dispatch();
        let recipe_details = r#"{"details":{"ingredients":["500mL water","100g spaghetti","25mL olive oil","4 cloves garlic","Salt"],"numSteps":5}}"#;
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_string().unwrap(), recipe_details);
    }

    #[test]
    fn get_recipe_details_200_no_recipe_found() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let mut response = client.get("/recipes/details/notARecipe").dispatch();
        let recipe_details = r#"{}"#;
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_string().unwrap(), recipe_details);
    }

    #[test]
    fn post_add_recipe_200_success() {
        let file = get_data_json();
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let content_type = Header::new("Content-Type", "application/json");
        let response = client
            .post("/recipes")
            .header(content_type)
            .body(constants::BAGEL_RECIPE)
            .dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(
            remove_whitespace(&file),
            remove_whitespace(constants::ADD_BUTTERED_BAGEL)
        );
    }

    // #[test]
    // fn post_add_recipe_200_no_recipe_found() {
    //     let client = Client::tracked(rocket()).expect("valid rocket instance");
    //     let mut response = client.get("/recipes").dispatch();
    //     let recipe_details = r#"{}"#;
    //     assert_eq!(response.status(), Status::Ok);
    //     assert_eq!(response.into_string().unwrap(), recipe_details);
    // }
}
