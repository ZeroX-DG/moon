use gtk::gio::Menu;
use gtk::{Entry, HeaderBar, Image, MenuButton, Popover, prelude::*};
use gtk::{Application, ApplicationWindow};

fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .default_width(320)
        .default_height(200)
        .title("Moon")
        .build();

    build_header_bar(&window);
    window.show_all();
}

fn build_header_bar(window: &ApplicationWindow) {
    let header_bar = HeaderBar::builder()
        .has_subtitle(false)
        .can_focus(false)
        .show_close_button(true)
        .build();

    let menu_button = MenuButton::builder()
        .use_popover(true)
        .image(&Image::builder().icon_name("open-menu-symbolic").build())
        .build();

    let menu = Menu::new();
    menu.append(Some("Dump DOM tree"), None);
    let menu_popover = Popover::from_model(Some(&menu_button), &menu);

    let url_entry = Entry::builder()
        .placeholder_text("Search or enter a URL")
        .primary_icon_name("search-symbolic")
        .hexpand(true)
        .build();

    menu_button.set_popover(Some(&menu_popover));
    header_bar.pack_start(&menu_button);
    header_bar.set_custom_title(Some(&url_entry));
    window.set_titlebar(Some(&header_bar));
}

pub fn start_main() {
    let app = Application::builder()
        .application_id("org.moon.MoonBrowser")
        .build();

    app.connect_activate(|app| {
        build_ui(app);
    });

    app.run();
}
