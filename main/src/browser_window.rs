use gtk::gio::Menu;
use gtk::{Application, ApplicationWindow, Entry, HeaderBar, Image, MenuButton, Popover};
use gtk::prelude::*;

pub struct BrowserWindow {
    window: ApplicationWindow,
    url_entry: Entry,
    content_area: Image,
}

impl BrowserWindow {
    pub fn new(app: &Application) -> Self {
        let window = ApplicationWindow::builder()
            .application(app)
            .default_width(320)
            .default_height(200)
            .title("Moon")
            .build();
        Self {
            window,
            url_entry: Entry::new(),
            content_area: Image::new()
        }
    }

    pub fn initialize(&self) {
        self.build_header_bar();
        self.build_content_area();
        self.window.show_all();
    }

    fn build_header_bar(&self) {
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

        self.url_entry.set_placeholder_text(Some("Search or enter a URL"));
        self.url_entry.set_primary_icon_name(Some("search-symbolic"));
        self.url_entry.set_hexpand(true);

        menu_button.set_popover(Some(&menu_popover));
        header_bar.pack_start(&menu_button);
        header_bar.set_custom_title(Some(&self.url_entry));
        self.window.set_titlebar(Some(&header_bar));
    }

    fn build_content_area(&self) {
        self.content_area.set_hexpand(true);
        self.content_area.set_vexpand(true);
    }
}

