/// Box-model dimensions for each layout box
#[derive(Debug, Clone)]
pub struct Dimensions {
    pub content: ContentSize,
    pub padding: EdgeSizes,
    pub margin: EdgeSizes,
    pub border: EdgeSizes
}

/// Size of the content area (all in px)
#[derive(Debug, Clone)]
pub struct ContentSize {
    pub width: f32,
    pub height: f32
}

/// Edge size of the box (all in px)
#[derive(Debug, Clone)]
pub struct EdgeSizes {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32
}

impl Default for Dimensions {
    fn default() -> Self {
        Self {
            content: ContentSize {
                width: 0.0,
                height: 0.0
            },
            padding: Default::default(),
            border: Default::default(),
            margin: Default::default()
        }
    }
}

impl Default for EdgeSizes {
    fn default() -> Self {
        Self {
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
            left: 0.0 
        }
    }
}
