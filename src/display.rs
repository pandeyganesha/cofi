use gtk4::{ApplicationWindow, prelude::*};
use gtk4::{Application, glib, Button};
use std::cell::Cell;


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

    let button_increase = Button::builder()
        .label("Increase")
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    let number = Cell::new(0);

    button_increase.connect_clicked(move |_| number.set(number.get() + 1) );

    let window = ApplicationWindow::builder()
        .application(app)
        .title("My App")
        .child(&button_increase)
        .build();


    window.fullscreen();
    window.present();
}