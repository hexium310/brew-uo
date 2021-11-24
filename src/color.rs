use colored::Color;

pub struct VersionColor {
    pub major: Color,
    pub minor: Color,
    pub other: Color,
}

pub const VERSION_COLOR: VersionColor = VersionColor {
    major: Color::Red,
    minor: Color::Blue,
    other: Color::Green,
};
