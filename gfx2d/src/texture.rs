use super::*;
use gfx::Factory;
use image;
use std::path::Path;
use std::convert::AsRef;

#[derive(Debug, Clone)]
pub struct Texture(TextureHandle, ShaderResourceView, Sampler);

impl Texture {
    pub fn load<P>(g: &mut Gfx2dContext, fname: P, filter: FilterMethod, wrap: WrapMode, color_key: Option<Color>) -> Texture
        where P: AsRef<Path>
    {
        // TODO: if wrap is repeat make it power of 2 so it works on webgl 1.0
        // TODO: handle image loading errors?

        let mut img = image::open(fname).unwrap().to_rgba();

        if let Some(color) = color_key {
            remove_color_key(&mut img, color);
        }

        premultiply_image(&mut img);

        let dimensions = (img.width() as u16, img.height() as u16);
        create_texture(&mut g.fct, &mut g.enc, dimensions, &img, filter, wrap)
    }

    pub fn new(g: &mut Gfx2dContext, (w, h): (u16, u16),
        data: &[u8], filter: FilterMethod, wrap: WrapMode) -> Texture
    {
        create_texture(&mut g.fct, &mut g.enc, (w, h), data, filter, wrap)
    }

    pub fn dimensions(&self) -> (u16, u16) {
        let (w, h, _, _) = self.0.get_info().kind.get_dimensions();
        (w, h)
    }

    pub fn handle(&self) -> (ShaderResourceView, Sampler) {
        (self.1.clone(), self.2.clone())
    }

    pub fn is(&self, other: &Texture) -> bool {
        self.0 == other.0
    }
}

pub fn create_texture(fct: &mut GlFactory, enc: &mut GlEncoder, (w, h): (u16, u16),
    data: &[u8], filter: FilterMethod, wrap: WrapMode) -> Texture
{
    let k = D2(w, h as u16, AaMode::Single);
    let (t, v) = fct.create_texture_immutable_u8::<Rgba8>(k, Mipmap::Allocated, &[data]).unwrap();
    let s = fct.create_sampler(SamplerInfo::new(filter, wrap));
    enc.generate_mipmap(&v);
    Texture(t, v, s)
}

pub fn premultiply_image(img: &mut image::RgbaImage) {
    for (_, _, pixel) in img.enumerate_pixels_mut() {
        let a = pixel[3] as f32 / 255.0;

        *pixel = image::Rgba([
            (pixel[0] as f32 * a) as u8,
            (pixel[1] as f32 * a) as u8,
            (pixel[2] as f32 * a) as u8,
            pixel[3],
        ]);
    }
}

pub fn remove_color_key(img: &mut image::RgbaImage, color_key: Color) {
    for (_, _, pixel) in img.enumerate_pixels_mut() {
        if rgba(pixel[0], pixel[1], pixel[2], pixel[3]) == color_key {
            *pixel = image::Rgba([0, 0, 0, 0]);
        }
    }
}
