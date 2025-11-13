#[derive(Clone, Copy)]
pub struct RGB {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl RGB {
    #[must_use]
    pub const fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }
}

#[derive(Clone, Copy)]
pub struct ColorSettings {
    pub debug: RGB,
    pub info: RGB,
    pub warn: RGB,
    pub error: RGB,
}

impl Default for ColorSettings {
    fn default() -> Self {
        Self {
            debug: RGB::new(38, 45, 56),
            info: RGB::new(15, 115, 255),
            warn: RGB::new(247, 155, 35),
            error: RGB::new(255, 0, 0),
        }
    }
}
