mod display;

use std::fs;
use std::path::Path;
use freedesktop_entry_parser::parse_entry;

fn show_on_ui(path: &String) -> bool {
    let entry = match parse_entry(path){
        Ok(entry) => entry,
        Err(_) => return false,
    };

    let value = entry
    .section("Desktop Entry")
    .expect("Did not find Desktop Entry")
    .attr("NoDisplay")
    .get(0)
    .expect("No Display exist")
    .to_lowercase();

    match value.as_str() {
        "true" => true,
        _ => false
    }
}

fn main() -> std::io::Result<()> {
    let mut app_names: Vec<String> = Vec::new();

    for entry in fs::read_dir("/usr/share/applications")? {
        let entry = entry ?;
        let path = entry.path();

        if path.is_file(){
            if let Some(name) = path.file_name(){
                app_names.push(name.to_string_lossy().to_string());
            } 
        }
    }
    // let filtered_apps: Vec<String> = app_names
    // .iter()
    // .filter(|app| show_on_ui(app))
    // .collect();

    display::scatter_on_screen(app_names);
    Ok(())
}
