use gtk::{
    traits::{ContainerExt, EntryExt},
    Button, Entry, Image,
};

use crate::app::get_app_runtime;

pub struct PrimaryBar {
    pub url_entry: Entry,
}

impl PrimaryBar {
    pub fn new(container: &gtk::Box) -> Self {
        let url_entry = Entry::builder()
            .placeholder_text("Enter URL")
            .primary_icon_name("search-symbolic")
            .hexpand(true)
            .margin_top(5)
            .margin_bottom(5)
            .margin_start(5)
            .build();

        url_entry.connect_activate(|entry| {
            let raw_url = entry.text().to_string();
            log::debug!("GOTO: {}", raw_url);
            get_app_runtime().update_state(move |state| {
                state.browser().goto(raw_url);
            });
        });

        let backward_btn = Button::builder()
            .relief(gtk::ReliefStyle::None)
            .image(&Image::from_icon_name(
                Some("go-previous"),
                gtk::IconSize::Button,
            ))
            .margin_top(5)
            .margin_bottom(5)
            .build();

        let forward_btn = Button::builder()
            .relief(gtk::ReliefStyle::None)
            .image(&Image::from_icon_name(
                Some("go-next"),
                gtk::IconSize::Button,
            ))
            .margin_top(5)
            .margin_bottom(5)
            .build();

        let reload_btn = Button::builder()
            .relief(gtk::ReliefStyle::None)
            .image(&Image::from_icon_name(
                Some("view-refresh"),
                gtk::IconSize::Button,
            ))
            .margin_top(5)
            .margin_bottom(5)
            .build();

        let bar = gtk::Box::builder()
            .height_request(40)
            .margin_start(5)
            .margin_end(5)
            .build();

        bar.add(&backward_btn);
        bar.add(&forward_btn);
        bar.add(&reload_btn);
        bar.add(&url_entry);
        container.add(&bar);

        Self { url_entry }
    }
}
