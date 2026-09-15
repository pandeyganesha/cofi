use gtk4::Orientation::{Vertical};
use gtk4::{ApplicationWindow, prelude::*};
use gtk4::{Application, glib, Label, Box};


const APP_ID: &str = "dev.pandey.cofi";

pub fn scatter_on_screen(app_names: Vec<String>) -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(build_ui(app_names));
    app.run()
}

fn build_ui(app_names: Vec<String>) -> impl Fn(&Application) {
    move |app| {

    let canvas = Box::new(Vertical, 5);

    for app_name in &app_names {
        canvas.append(&Label::new(Some(app_name)));
    }

    let window = ApplicationWindow::builder()
        .application(app)
        .title("My App")
        .child(&canvas)
        .build();

    window.fullscreen();
    window.present();
    }
}