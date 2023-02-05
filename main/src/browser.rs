use crate::render_client::RenderClient;

pub struct Browser {
    tabs: Vec<Tab>,
    active_tab_index: usize
}

impl Browser {
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            active_tab_index: 0
        }
    }

    pub fn new_tab(&mut self) -> usize {
        self.tabs.push(Tab::new());

        self.tabs.len() - 1
    }

    pub fn current_tab_mut(&mut self) -> Option<&mut Tab> {
        self.tabs.get_mut(self.active_tab_index)
    }

    pub fn current_tab(&self) -> Option<&Tab> {
        self.tabs.get(self.active_tab_index)
    }
}

pub struct Tab {
    render_client: RenderClient
}

impl Tab {
    pub fn new() -> Self {
        let render_client = RenderClient::new();
        render_client.wait_till_ready();
        Self {
            render_client
        }
    }

    pub fn goto_raw_url(&mut self, url: String) {
        self.render_client.load_raw_url(url)
    }
}