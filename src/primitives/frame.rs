// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FrameUniform {
    frame: u32,
}

impl FrameUniform {
    pub fn new() -> Self {
        Self { frame: 0 }
    }

    pub fn incr_frame(&mut self) {
        let new_frame = self.frame + 1;
        self.frame = new_frame % 100;
    }
}
