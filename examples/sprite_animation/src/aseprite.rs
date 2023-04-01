#[derive(Debug, serde::Deserialize)]
pub struct AsepriteAnimation {
    pub meta: Meta,
    pub frames: Vec<AnimationFrameData>,
}

#[derive(Debug, serde::Deserialize)]
pub struct Meta {
    pub size: Size,
}

#[derive(Debug, serde::Deserialize)]
pub struct Size {
    pub w: u64,
    pub h: u64,
}

#[derive(Debug, serde::Deserialize)]
pub struct AnimationFrameData {
    pub frame: Frame,
    pub duration: u64,
}

#[derive(Debug, serde::Deserialize)]
pub struct Frame {
    pub x: u64,
    pub y: u64,
    pub w: u64,
    pub h: u64,
}
