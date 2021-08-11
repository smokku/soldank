use super::*;
use gfx2d::gfx2d_extra::load_image_rgba;
use std::ops::Range;

pub struct MapGraphics {
    pub batch: DrawBatch,
    pub background: Range<usize>,
    pub polys_back: Range<usize>,
    pub polys_front: Range<usize>,
    pub scenery_back: Range<usize>,
    pub scenery_mid: Range<usize>,
    pub scenery_front: Range<usize>,
}

fn is_prop_active(map: &MapFile, prop: &MapProp) -> bool {
    prop.active && prop.level <= 2 && prop.style > 0 && prop.style as usize <= map.scenery.len()
}

fn is_background_poly(poly: &MapPolygon) -> bool {
    matches!(
        poly.polytype,
        PolyType::Background | PolyType::BackgroundTransition
    )
}

fn add_poly(batch: &mut DrawBatch, poly: &MapPolygon, texture: &Texture) {
    let (a, b, c) = (&poly.vertices[0], &poly.vertices[1], &poly.vertices[2]);

    batch.add(
        Some(texture),
        &[
            vertex(
                vec2(a.x, a.y),
                vec2(a.u, a.v),
                rgba(a.color.r, a.color.g, a.color.b, a.color.a),
            ),
            vertex(
                vec2(b.x, b.y),
                vec2(b.u, b.v),
                rgba(b.color.r, b.color.g, b.color.b, b.color.a),
            ),
            vertex(
                vec2(c.x, c.y),
                vec2(c.u, c.v),
                rgba(c.color.r, c.color.g, c.color.b, c.color.a),
            ),
        ],
    );
}

fn add_scenery(batch: &mut DrawBatch, (prop, sprite): (&MapProp, &Sprite)) {
    let color = rgba(prop.color.r, prop.color.g, prop.color.b, prop.alpha);
    let mut sprite = sprite.clone();
    sprite.width = prop.width as f32;
    sprite.height = prop.height as f32;

    batch.add_sprite(
        &sprite,
        color,
        Transform::FromOrigin {
            pos: vec2(prop.x, prop.y),
            scale: vec2(prop.scale_x, prop.scale_y),
            rot: (-prop.rotation, vec2(0.0, 1.0)),
        },
    );
}

impl MapGraphics {
    pub fn background(&mut self) -> DrawSlice {
        self.batch.slice(self.background.clone())
    }
    pub fn polys_back(&mut self) -> DrawSlice {
        self.batch.slice(self.polys_back.clone())
    }
    pub fn polys_front(&mut self) -> DrawSlice {
        self.batch.slice(self.polys_front.clone())
    }
    pub fn scenery_back(&mut self) -> DrawSlice {
        self.batch.slice(self.scenery_back.clone())
    }
    pub fn scenery_mid(&mut self) -> DrawSlice {
        self.batch.slice(self.scenery_mid.clone())
    }
    pub fn scenery_front(&mut self) -> DrawSlice {
        self.batch.slice(self.scenery_front.clone())
    }

    pub fn empty() -> MapGraphics {
        MapGraphics {
            batch: DrawBatch::new_static(),
            background: 0..0,
            polys_back: 0..0,
            polys_front: 0..0,
            scenery_back: 0..0,
            scenery_mid: 0..0,
            scenery_front: 0..0,
        }
    }

    pub fn new(ctx: &mut Context, fs: &mut Filesystem, map: &MapFile) -> MapGraphics {
        let texture_file = filename_override(fs, "textures", &map.texture_name);

        let texture = if fs.is_file(texture_file.clone()) {
            let img = load_image_rgba(fs, texture_file);
            let width = img.width();
            let height = img.height();
            let bytes = img.into_raw();

            Texture::from_data_and_format(
                ctx,
                &bytes[..],
                TextureParams {
                    format: TextureFormat::RGBA8,
                    wrap: TextureWrap::Repeat,
                    filter: FilterMode::Linear,
                    width,
                    height,
                },
            )
        } else {
            Texture::from_data_and_format(
                ctx,
                &[255u8; 4],
                TextureParams {
                    format: TextureFormat::RGBA8,
                    wrap: TextureWrap::Clamp,
                    filter: FilterMode::Nearest,
                    width: 1,
                    height: 1,
                },
            )
        };

        let (scenery_used, sprite_index) = {
            let mut scenery_used = vec![false; map.scenery.len()];
            let mut sprite_index = vec![0usize; map.scenery.len()];

            for prop in &map.props {
                if is_prop_active(map, prop) {
                    scenery_used[prop.style as usize - 1] |= true;
                }
            }

            let mut n = 0;
            for (i, _) in map.scenery.iter().enumerate() {
                if scenery_used[i] {
                    sprite_index[i] = n;
                    n += 1;
                }
            }

            (scenery_used, sprite_index)
        };

        let sprites = {
            let scenery_info: Vec<SpriteInfo> = map
                .scenery
                .iter()
                .enumerate()
                .filter(|&(i, _)| scenery_used[i])
                .map(|(_, s)| {
                    let fname = filename_override(fs, "scenery-gfx", &s.filename);

                    let color_key = match fname.extension() {
                        Some(ext) => {
                            if ext == "bmp" || ext == "gif" {
                                Some(rgb(0, 255, 0))
                            } else {
                                None
                            }
                        }
                        _ => None,
                    };

                    SpriteInfo::new(fname, vec2(1.0, 1.0), color_key)
                })
                .collect();

            Spritesheet::new(ctx, fs, 8, FilterMode::Linear, &scenery_info).sprites
        };

        let props = {
            let mut sorted: [Vec<(&MapProp, &Sprite)>; 3] = [
                Vec::with_capacity(map.props.len()),
                Vec::with_capacity(map.props.len()),
                Vec::with_capacity(map.props.len()),
            ];

            for prop in &map.props {
                if is_prop_active(map, prop) {
                    sorted[prop.level as usize]
                        .push((prop, &sprites[sprite_index[prop.style as usize - 1]]));
                }
            }

            sorted
        };

        let mut batch = DrawBatch::new_static();

        let background = {
            let d = 25.0 * f32::max(map.sectors_division as f32, f32::ceil(0.5 * 480.0 / 25.0));
            let (top, btm) = (map.bg_color_top, map.bg_color_bottom);

            batch.add_quad(
                None,
                &[
                    vertex(vec2(0.0, -d), vec2(0.0, 0.0), rgb(top.r, top.g, top.b)),
                    vertex(vec2(1.0, -d), vec2(0.0, 0.0), rgb(top.r, top.g, top.b)),
                    vertex(vec2(1.0, d), vec2(0.0, 0.0), rgb(btm.r, btm.g, btm.b)),
                    vertex(vec2(0.0, d), vec2(0.0, 0.0), rgb(btm.r, btm.g, btm.b)),
                ],
            );

            batch.split()
        };

        map.polygons
            .iter()
            .filter(|&p| is_background_poly(p))
            .for_each(|poly| add_poly(&mut batch, poly, &texture));
        let polys_back = batch.split();

        props[0]
            .iter()
            .for_each(|prop| add_scenery(&mut batch, *prop));
        let scenery_back = batch.split();

        props[1]
            .iter()
            .for_each(|prop| add_scenery(&mut batch, *prop));
        let scenery_mid = batch.split();

        map.polygons
            .iter()
            .filter(|&p| !is_background_poly(p))
            .for_each(|poly| add_poly(&mut batch, poly, &texture));
        let polys_front = batch.split();

        props[2]
            .iter()
            .for_each(|prop| add_scenery(&mut batch, *prop));
        let scenery_front = batch.split();

        MapGraphics {
            batch,
            background,
            polys_back,
            polys_front,
            scenery_back,
            scenery_mid,
            scenery_front,
        }
    }
}
