use eframe::egui;

pub struct PPURenderer {
    pub height: usize,
    pub width: usize,

    pub pixels: Vec<u8>,
}

impl PPURenderer {
    pub fn new() -> Self {
        Self {
            height: 240,
            width: 256,
            pixels: vec![0; 240 * 256 * 3],
        }
    }

    pub fn new_custom_size(width: usize, height: usize) -> Self {
        Self {
            height,
            width,
            pixels: vec![0; height * width * 3],
        }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: (u8, u8, u8)) {
        let index = (y * self.width + x) * 3;
        self.pixels[index] = color.0;
        self.pixels[index + 1] = color.1;
        self.pixels[index + 2] = color.2;
    }

    pub fn get_color_image(&self) -> egui::ColorImage {
        egui::ColorImage::from_rgb([self.width, self.height], &self.pixels)
    }
}
