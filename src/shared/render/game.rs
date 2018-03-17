use super::*;
use shared::state::MainState;
use shared::soldier::Soldier;
use shared::mapfile::MapFile;
use gfx2d::*;

pub struct GameGraphics {
    map: MapGraphics,
    batch: DrawBatch,
    soldier: Sprite,
}

impl GameGraphics {
    pub fn new(_context: &mut Gfx2dContext) -> GameGraphics {
        GameGraphics {
            map: MapGraphics::empty(),
            batch: DrawBatch::new(),
            soldier: Sprite {
                width: 10.0,
                height: 10.0,
                texcoords_x: (0.0, 0.0),
                texcoords_y: (0.0, 0.0),
                texture: None,
            },
        }
    }

    pub fn load_map(&mut self, context: &mut Gfx2dContext, map: &MapFile) {
        self.map = MapGraphics::new(context, map);
    }

    pub fn render_frame(&mut self, context: &mut Gfx2dContext, state: &MainState, soldier: &Soldier, (t, p): (f32, f32)) {
        let (w, h) = (state.game_width as f32, state.game_height as f32);
        let (w, h) = (1280.0 * (480.0 / 720.0), 720.0 * (480.0 / 720.0));

        let campos = {
            let a = vec2(state.camera_prev.x, state.camera_prev.y);
            let b = vec2(state.camera.x, state.camera.y);
            // a + (b - a) * p
            vec2(state.camera.x, state.camera.y)
        };

        let delta = campos - vec2(w, h)/2.0;
        let transform = Transform::ortho(delta.x, delta.x + w, delta.y, delta.y + h).matrix();

        let pos = vec2(state.soldier_parts.pos[1].x, state.soldier_parts.pos[1].y);
        let cam = pos + vec2(state.mouse.x, state.mouse.y);
        let transform = Transform::ortho(cam.x - w/2.0, cam.x + w/2.0, cam.y - h/2.0, cam.y + h/2.0).matrix();

        self.batch.clear();
        self.batch.add_tinted_sprite(&self.soldier, rgba(255, 255, 0, 128), Transform::WithPivot {
            pivot: vec2(5.0, 10.0),
            pos: pos,
            scale: vec2(1.0, 1.0),
            rot: 0.0,
        });

        context.clear(rgb(0, 0, 0));
        context.draw(self.map.background(), &Transform::ortho(0.0, 1.0, delta.y, delta.y + h).matrix());
        context.draw(self.map.polys_back(), &transform);
        context.draw(self.map.scenery_back(), &transform);
        context.draw(self.batch.all(), &transform);
        context.draw(self.map.scenery_mid(), &transform);
        context.draw(self.map.polys_front(), &transform);
        context.draw(self.map.scenery_front(), &transform);
    }
}
