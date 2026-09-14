use gtk4::{ApplicationWindow, prelude::*};
use gtk4::{Application, glib, Label};


const APP_ID: &str = "dev.pandey.cofi";

pub fn scatter_on_screen(app_names: &[String]) -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();

    for app_name in app_names {
        println!("{app_name}");
    }

    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &Application){

    let label = Label::new(Some("Hello Ganesh"));

    let window = ApplicationWindow::builder()
        .application(app)
        .title("My App")
        .child(&label)
        .build();

    window.fullscreen();
    window.present();
}