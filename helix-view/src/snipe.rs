use helix_core::unicode::width::UnicodeWidthStr;

#[derive(Debug)]
/// Info box used in editor. Rendering logic will be in other crate.
pub struct Snipe {
    /// Title shown at top.
    pub title: String,
    /// Items
    pub items: Vec<(String, String)>,
    /// Body width.
    pub width: u16,
    /// Body height.
    pub height: u16,
}

impl Snipe {
    pub fn new(title: &str, body: &[(String, String)]) -> Self {
        if body.is_empty() {
            return Self {
                title: title.to_string(),
                height: 1,
                width: title.len() as u16,
                items: vec![],
            };
        }

        let key_width = body.iter().map(|(item, _)| item.width()).max().unwrap();
        let buffer_width = body.iter().map(|(_, item)| item.width()).max().unwrap();

        Self {
            title: title.to_string(),
            width: key_width as u16 + buffer_width as u16 + 4,
            height: body.len() as u16,
            items: body
                .iter()
                .map(|(k, v)| (String::from(k), String::from(v)))
                .collect(),
        }
    }
}
