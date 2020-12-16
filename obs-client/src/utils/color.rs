/// Color represented by additive channels: Blue (b), Green (g), Red (r), and
/// Alpha (a).
#[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Eq, Ord)]
pub struct BGRA8 {
    pub b: u8,
    pub g: u8,
    pub r: u8,
    pub a: u8,
}
