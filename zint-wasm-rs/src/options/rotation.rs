/// Symbol rotation breakpoints.
/// 
/// Rotations that aren't 90° multiples aren't supported by zint.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(i32)]
pub enum Rotation {
    #[default]
    Deg0 = 0,
    Deg90 = 90,
    Deg180 = 180,
    Deg270 = 270,
}
impl From<Rotation> for i32 {
    fn from(value: Rotation) -> Self {
        value as i32
    }
}
