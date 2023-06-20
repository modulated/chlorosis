use super::Byte;

#[derive(Debug)]
pub struct Screen {
    framebuffer: [Byte; 160 * 144],
}

impl Default for Screen {
    fn default() -> Self {
        Self {
            framebuffer: [Byte(0); 160 * 144],
        }
    }
}
