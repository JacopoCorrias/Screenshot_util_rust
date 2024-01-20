use eframe::egui::Key;
#[derive(Debug)]
pub struct KeyBindings {
    pub save: Key,
    pub cancel: Key,
    pub new: Key,
    pub crop: Key,
    pub fullscreen: Key,
    pub clipboard: Key,
}
impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            save: Key::S,
            cancel: Key::Z,
            new: Key::N,
            crop: Key::X,
            fullscreen: Key::F,
            clipboard: Key::C,
        }
    }
}

impl KeyBindings {
    pub fn new() -> KeyBindings{
        KeyBindings::default()
    }
    pub fn is_key_assigned(&self, key: Key) -> bool {
        self.fullscreen == key
            || self.new == key
            || self.save == key
            || self.cancel == key
            || self.crop == key
    }
}