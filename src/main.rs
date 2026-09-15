mod display;

use std::fs;

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
    display::scatter_on_screen(app_names);
    Ok(())
}
