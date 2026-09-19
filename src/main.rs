mod display;

use std::fs;
use freedesktop_entry_parser::parse_entry;

// It should read a value for any given key under Desktop Entry
fn get_value_for_key(path: &String, key: &String) -> String {
    let entry = match parse_entry(path){
        Ok(entry) => entry,
        Err(_) => return String::from(""),
    };

    let value = entry
    .get("Desktop Entry", key)
    .and_then(|values| values.first())
    .map(String::as_str)
    .unwrap_or("");

    value.to_string()

}

fn main() -> std::io::Result<()> {
    let mut app_paths: Vec<String> = Vec::new();

    for entry in fs::read_dir("/usr/share/applications")? {
        let entry = entry ?;
        let path = entry.path();

        if path.is_file(){
            app_paths.push(path.to_string_lossy().to_string());
        }
    }
    // println!("{:#?}", app_paths);
    // Generic names which are not set to NoDisplay and are not empty
    let filtered_apps: Vec<String> = app_paths
    .into_iter()
    .filter(|app_path| get_value_for_key(&app_path, &String::from("NoDisplay")).to_lowercase() != "true")
    .map(|app_path| get_value_for_key(&app_path, &String::from("Name")))
    .filter(|name| !name.is_empty())
    .collect();

    println!("{:#?}", filtered_apps);
    println!("{:#?}", filtered_apps.len());


    display::scatter_on_screen(filtered_apps);
    Ok(())
}
