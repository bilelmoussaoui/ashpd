use crate::{
    zvariant::{self, DeserializeDict, Type},
    Error,
};

#[derive(DeserializeDict, Clone, Copy, PartialEq, Type)]
/// A color as a RGB tuple.
///
/// **Note** the values are normalized in the [0.0, 1.0] range.
#[zvariant(signature = "dict")]
pub struct Color {
    color: [f64; 3],
}

impl Color {
    /// Red.
    pub fn red(&self) -> f64 {
        self.color[0]
    }

    /// Green.
    pub fn green(&self) -> f64 {
        self.color[1]
    }

    /// Blue.
    pub fn blue(&self) -> f64 {
        self.color[2]
    }
}

#[cfg(feature = "gtk4")]
impl From<Color> for gtk4::gdk::RGBA {
    fn from(color: Color) -> Self {
        gtk4::gdk::RGBA::builder()
            .red(color.red() as f32)
            .green(color.green() as f32)
            .blue(color.blue() as f32)
            .build()
    }
}

impl TryFrom<zvariant::OwnedValue> for Color {
    type Error = Error;

    fn try_from(value: zvariant::OwnedValue) -> Result<Self, Self::Error> {
        value
            .try_into()
            .map_err(|_| crate::Error::ParseError("Failed to parse color"))
    }
}

impl std::fmt::Debug for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Color")
            .field("red", &self.red())
            .field("green", &self.green())
            .field("blue", &self.blue())
            .finish()
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "({}, {}, {})",
            self.red(),
            self.green(),
            self.blue()
        ))
    }
}
