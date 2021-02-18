```rust
// gfx2d reference

::rgba(r, g, b, a) -> Color
::rgb(r, g, b) -> Color     // rgb & rgba take u8 params
::vertex(pos: Vec2, texcoords: Vec2, color: Color) -> Vertex
::gfx2d_extra::load_image_rgba(filename) -> image::RgbaImage
::gfx2d_extra::premultiply_image(&mut image::RgbaImage)
::gfx2d_extra::remove_color_key(&mut image::RgbaImage, Color)

Gfx2dContext
    ::initialize(title: &str, width: u32, height: u32) -> Gfx2dContext
    .wnd: glutin::GlWindow
    .evt: glutin::EventsLoop
    .clear(Color)
    .draw(DrawSlice, &Mat2d)
    .present()
DrawBatch
    ::new()
    ::new_static()
    .clear()
    .add(Option<&Texture>, &[Vertex;3])
    .add_quad(Option<&Texture>, &[Vertex;4])
    .add_sprite(&Sprite, Color, Transform)
    .split() -> Range<usize> // range from last split (or start) to last added stuff
    .slice(Range<usize>) -> DrawSlice
    .all() -> DrawSlice
Sprite
    .width: f32
    .height: f32
    .texcoords_x: (f32, f32)
    .texcoords_y: (f32, f32)
    .texture: Option<Texture>
    ::new(w, h, (tx0, tx1), (ty0, ty1), Option<&Texture>) -> Sprite
    ::from_texture(&Texture, pixel_ratio: Vec2) -> Sprite
Transform // enum
    .matrix() -> Mat2d
    ::none() -> Transform::Pos(vec2(0, 0))
    ::pos(x, y) -> Transform::Pos(vec2(x, y))
    ::origin(pos, scale, (rot, center)) -> Transform::FromOrigin
    ::pivot(pivot, pos, scale, rot) -> Transform::WithPivot
    ::ortho(left, right, top, bottom) -> Transform::Ortho

    // enum variants
    Pos(Vec2)
    FromOrigin{pos: Vec2, scale: Vec2, rot: (Rad, Vec2)}
    WithPivot{pivot: Vec2, pos: Vec2, scale: Vec2, rot: Rad}
    Ortho{left, right, top, bottom}

    // FromOrigin:
    // Rotation is done around a rotation center but position and scale are done
    // from the origin (top-left corner in the case of sprites).

    // WithPivot:
    // Position, rotation and scale are all done relative to a pivot point.
Texture
    ::load(
        &mut Gfx2dContext,
        filename,          // anything that coerces to Path (AsRef<Path>)
        FilterMethod,      // FilterMethod::{Scale, Mipmap, Bilinear, Trilinear, Anisotropic(u8)}
        WrapMode,          // WrapMode::{Tile, Mirror, Clamp, Border}
        color_key: Option<Color>) -> Texture
    ::new(&mut Gfx2dContext, (w, h), data: &[u8], FilterMethod, WrapMode) -> Texture // u16 for w,h
    .dimensions() -> (u16, u16)
Spritesheet
    .textures: Vec<Texture>
    .sprites: Vec<Sprite>   // sprites in same order as load input
    ::empty() -> Spritesheet
    ::new(&mut Gfx2dContext, padding: i32, FilterMethod, &[SpriteInfo]) -> Spritesheet
SpriteInfo
    filename: PathBuf
    pixel_ratio: Vec2
    color_key: Option<Color>
    ::new(filename, pixel_ratio, color_key)
Color
    .r(), .g(), .b(), .a() -> u8
    .set_r(u8), .set_g(u8), .set_b(u8), .set_a(u8)
Vertex {
    pos: [f32;4],
    texcoords: [f32;4],
    color: [U8Norm;4]
}

math (submodule)
    ::rad(angle) -> Rad     // alias to cgmath::Rad<f32>
    ::deg(angle) -> Deg     // alias to cgmath::Deg<f32>
    ::vec2(x, y) -> Vec2    // alias to cgmath::Vector2<f32>
    ::vec3(x, y, z) -> Vec3 // alias to cgmath::Vector3<f32>
    Mat2d((f32,f32,f32), (f32,f32,f32))
        .0 // 1st row
        .1 // 2nd row - 3rd row is implicit, always (0,0,1)
        .to_3x3() -> [[f32;3];3] // column major result
        ::identity() -> Mat2d
        ::translate(x, y) -> Mat2d
        ::scale(x, y) -> Mat2d
        ::rotate(angle) -> Mat2d
        Mat2d * Mat2d -> Mat2d
        Mat2d * Vec2 -> Vec2
```
