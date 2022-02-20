use gtk::{Application, ApplicationWindow, Entry, HeaderBar, Image, traits::{ContainerExt, GtkWindowExt, HeaderBarExt}};

pub struct UI {
    pub app: Application,
    pub window: ApplicationWindow,
    pub url_entry: Entry,
    pub content_area: Image
}

impl UI {
    pub fn new(app: Application) -> Self {
        let window = ApplicationWindow::builder()
            .application(&app)
            .title("Moon")
            .default_width(1200)
            .default_height(600)
            .build();

        let url_entry = Entry::builder()
            .placeholder_text("Enter URL")
            .primary_icon_name("search-symbolic")
            .hexpand(true)
            .build();

        let content_area = Image::builder()
            .hexpand(true)
            .vexpand(true)
            .build();

        let header_bar = HeaderBar::builder()
            .show_close_button(true)
            .build();

        header_bar.set_custom_title(Some(&url_entry));
        window.set_titlebar(Some(&header_bar));
        window.add(&content_area);

        Self {
            app,
            window,
            url_entry,
            content_area
        }
    }
}
