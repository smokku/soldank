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

    pub fn render_frame(&mut self, context: &mut Gfx2dContext, state: &MainState, soldier: &Soldier,
        elapsed: f64, frame_percent: f32)
    {
        let (w, h) = (state.game_width, state.game_height);
        let dx = state.camera.x - w/2.0;
        let dy = state.camera.y - h/2.0;
        let transform = Transform::ortho(dx, dx + w, dy, dy + h).matrix();

        self.batch.clear();
        self.batch.add_tinted_sprite(&self.soldier, rgba(255, 255, 0, 128), Transform::WithPivot {
            pivot: vec2(5.0, 10.0),
            pos: vec2(soldier.skeleton.pos[1].x, soldier.skeleton.pos[1].y),
            scale: vec2(1.0, 1.0),
            rot: 0.0,
        });

        context.clear(rgb(0, 0, 0));
        context.draw(self.map.background(), &Transform::ortho(0.0, 1.0, dy, dy + h).matrix());
        context.draw(self.map.polys_back(), &transform);
        context.draw(self.map.scenery_back(), &transform);
        context.draw(self.batch.all(), &transform);
        context.draw(self.map.scenery_mid(), &transform);
        context.draw(self.map.polys_front(), &transform);
        context.draw(self.map.scenery_front(), &transform);

        // time precision test
        {
            let screen = Transform::ortho(0.0, 1280.0, 0.0, 720.0);
            let a = state.soldier_parts.old_pos[2].x;
            let b = state.soldier_parts.pos[2].x;
            let x = (a + (b - a) * frame_percent) % 1280.0;

            self.batch.clear();
            self.batch.add_quads(None, &[[
                vertex(vec2(x + 0.0,   0.0), vec2(0.0, 0.0), rgb(255, 0, 0)),
                vertex(vec2(x + 1.0,   0.0), vec2(0.0, 0.0), rgb(255, 0, 0)),
                vertex(vec2(x + 1.0, 720.0), vec2(0.0, 0.0), rgb(255, 0, 0)),
                vertex(vec2(x + 0.0, 720.0), vec2(0.0, 0.0), rgb(255, 0, 0)),
            ]]);

            context.draw(self.batch.all(), &screen.matrix());
        }
    }
}
