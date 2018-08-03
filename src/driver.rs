use gfx;

pub trait DisplayDriver {
    fn size(&self) -> gfx::Size;
    fn show_frame(&mut self, frame: gfx::Frame);
    fn get_frame(&mut self) -> gfx::Frame;
}
