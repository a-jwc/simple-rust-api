#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use rocket::http::Status;
use rocket::serde::json::{Json, Value};
use rocket::serde::{Deserialize, Serialize};
use rocket::{Request, State};

use std::fs::{self, File};
use std::io::Write;
use std::sync::RwLock;

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

struct JsonState {
    json: Value,
}

type MutJsonState = RwLock<Value>;

impl JsonState {
    fn new(json: Value) -> MutJsonState {
        RwLock::new(json)
    }
}

fn get_recipes_json(json: &State<MutJsonState>) -> Result<String, (Status, String)> {
    let json = json.read().unwrap();
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

fn open_file(file_path: String) -> File {
    let file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .append(false)
        .create(false)
        .open(file_path)
        .expect("unable to open");
    file
}

fn add_to_vec(data: Vec<Recipes>) -> Vec<String> {
    let mut all_recipe_names: Vec<String> = Vec::new();
    for ele in data.iter() {
        all_recipe_names.push(ele.name.to_owned());
    }
    all_recipe_names
}

#[get("/")]
fn index() -> &'static str {
    "trunk-web-api"
}

#[get("/allRecipes")]
fn all_recipes(json: &State<MutJsonState>) -> String {
    let json = json.read().unwrap();
    json.to_string()
}

#[get("/recipes")]
fn get_recipe_names(json: &State<MutJsonState>) -> Result<Value, (Status, String)> {
    // Call get_recipes_json to convert our JSON `Value` into a `String`, otherwise returns an `Error`
    let recipes = match crate::get_recipes_json(json) {
        Ok(r) => r,
        Err(e) => return Err(e),
    };

    // Deserialize the `String` into a vector of `Recipes`
    let data: Vec<Recipes> = serde_json::from_str(&recipes).unwrap_or_default();

    // Call add_to_vec to convert the Vec<Recipes> into a form to construct a JSON `Value`
    let all_recipe_names = crate::add_to_vec(data);

    // Create a JSON `Value` with a key of "recipeNames" and a value of `Vec<String>`
    let result: serde_json::Value = serde_json::json!( {
        "recipeNames": all_recipe_names,
    });

    // Return the successful result
    Ok(result)
}

#[get("/recipes/details/<name>")]
fn get_recipe_details(json: &State<MutJsonState>, name: &str) -> Result<Value, (Status, String)> {
    // Call get_recipes_json to convert our JSON `Value` into a `String`, otherwise returns an `Error`
    let recipes = match crate::get_recipes_json(json) {
        Ok(r) => r,
        Err(e) => return Err(e),
    };

    // Deserialize the `String` into a vector of `Recipes`
    let data: Vec<Recipes> = serde_json::from_str(&recipes).unwrap_or_default();

    // Loop through the `Vec<Recipes>`, using an if-else to check for a name match
    for ele in data.iter() {
        if ele.name.to_string() == name {
            // If we get a hit, construct the JSON `Value`. We will call `.len()` on the instructions to get the number of steps needed for the recipe
            let details: serde_json::Value = serde_json::json!({
              "ingredients": ele.ingredients,
              "numSteps": ele.instructions.len()
            });
            // Store this in our `result` with the key `details` as specified in our problem statement, then return the successful result
            let result = serde_json::json!({ "details": details });
            return Ok(result);
        } else {
            // If we don't get a hit, do nothing
            {}
        }
    }
    // If we reach this point, the name could not be found in our JSON and an appropriate response is returned
    Err((Status::BadRequest, "Name not found".to_string()))
}

#[post("/recipes", format = "json", data = "<item>")]
fn add_recipe(json: &State<MutJsonState>, item: Json<Recipes>) -> Result<(), (Status, String)> {
    let recipes = match crate::get_recipes_json(json) {
        Ok(r) => r,
        Err(e) => return Err(e),
    };
    let mut all_recipes: Vec<Recipes> = serde_json::from_str(&recipes).unwrap_or_default();

    // Create a vector of recipe names
    let all_recipe_names = crate::add_to_vec(serde_json::from_str(&recipes).unwrap_or_default());

    // Check if the recipe does not exist
    if !all_recipe_names.contains(&item.name) {
        // Consume the json wrapper and return the item
        let new_recipe = item.into_inner();

        // Add the new recipe
        all_recipes.push(new_recipe);

        // Construct a json with "recipes" as the key
        let result = serde_json::json!({ "recipes": all_recipes });

        // Dereference and lock the thread for writing
        *json.write().unwrap() = result.clone();

        // Open the json file
        let mut file = crate::open_file("data/data.json".to_string());

        // Overwrite the file with our recipes
        serde_json::to_writer_pretty(&mut file, &result).unwrap_or_default();
        file.flush().unwrap_or_default();
        Ok(())
    } else {
        Err((Status::BadRequest, "Recipe already exists".to_string()))
    }
}

// Much of the code is the same for `add_recipe` so please look there for code explanations
// or check below for the explanation on code difference
#[put("/recipes", format = "json", data = "<item>")]
fn edit_recipe(json: &State<MutJsonState>, item: Json<Recipes>) -> Result<(), (Status, String)> {
    let recipes = match crate::get_recipes_json(json) {
        Ok(r) => r,
        Err(e) => return Err(e),
    };
    let mut all_recipes: Vec<Recipes> = serde_json::from_str(&recipes).unwrap_or_default();
    let all_recipe_names = crate::add_to_vec(serde_json::from_str(&recipes).unwrap_or_default());
    if all_recipe_names.contains(&item.name) {
        all_recipes.retain(|x| x.name != item.name);
        let new_recipe = item.into_inner();
        all_recipes.push(new_recipe);
        let result = serde_json::json!({ "recipes": all_recipes });

        // Dereference and lock the thread for writing
        *json.write().unwrap() = result.clone();
        let mut file = crate::open_file("data/data.json".to_string());
        serde_json::to_writer_pretty(&mut file, &result).unwrap_or_default();
        file.flush().unwrap_or_default();
        Ok(())
    } else {
        Err((Status::BadRequest, "Recipe does not exist".to_string()))
    }
}

#[launch]
fn rocket() -> _ {
    let rdr = crate::open_file("data/data.json".to_string());
    let json: Value =
        serde_json::from_reader(rdr).expect("Failed to convert rdr into serde_json::Value");
    rocket::build()
        .manage(JsonState::new(json))
        .register("/", catchers![not_found])
        .mount(
            "/",
            routes![
                index,
                get_recipe_names,
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
        let rdr = File::open("data/data.json").expect("Failed to open data.json");
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
