use super::*;
use shared::state::MainState;
use shared::soldier::Soldier;
use shared::mapfile::MapFile;
use gfx2d::*;

pub struct GameGraphics {
    map: MapGraphics,
    batch: DrawBatch,
}

impl GameGraphics {
    pub fn new(_context: &mut Gfx2dContext) -> GameGraphics {
        GameGraphics {
            map: MapGraphics::empty(),
            batch: DrawBatch::new(),
        }
    }

    pub fn load_map(&mut self, context: &mut Gfx2dContext, map: &MapFile) {
        self.map = MapGraphics::new(context, map);
    }

    pub fn render_frame(&mut self, context: &mut Gfx2dContext, state: &MainState, soldier: &Soldier,
        _elapsed: f64, frame_percent: f32)
    {
        let (w, h) = (state.game_width, state.game_height);
        let dx = state.camera.x - w/2.0;
        let dy = state.camera.y - h/2.0;
        let transform = Transform::ortho(dx, dx + w, dy, dy + h).matrix();

        context.clear(rgb(0, 0, 0));
        context.draw(self.map.background(), &Transform::ortho(0.0, 1.0, dy, dy + h).matrix());
        context.draw(self.map.polys_back(), &transform);
        context.draw(self.map.scenery_back(), &transform);
        context.draw(self.map.scenery_mid(), &transform);
        context.draw(self.map.polys_front(), &transform);
        context.draw(self.map.scenery_front(), &transform);

        // skeleton points
        {
            self.batch.clear();

            for p in &soldier.skeleton.pos[1..25] {
                let m = Mat2d::translate(p.x, p.y);

                self.batch.add_quads(None, &[[
                    vertex(m * vec2(-1.0, -1.0), Vec2::zeros(), rgb(0, 0, 255)),
                    vertex(m * vec2( 1.0, -1.0), Vec2::zeros(), rgb(0, 0, 255)),
                    vertex(m * vec2( 1.0,  1.0), Vec2::zeros(), rgb(0, 0, 255)),
                    vertex(m * vec2(-1.0,  1.0), Vec2::zeros(), rgb(0, 0, 255)),
                ]]);
            }

            context.draw(self.batch.all(), &transform);
        }

        // cursor
        {
            let size = context.wnd.get_inner_size().unwrap();
            let size = vec2(size.0 as f32, size.1 as f32);
            let x = f32::floor(state.mouse.x * size.x / w);
            let y = f32::floor(state.mouse.y * size.y / h);
            let screen = Transform::ortho(0.0, size.x, 0.0, size.y).matrix();

            self.batch.clear();
            self.batch.add_quads(None, &[
                [
                    vertex(vec2(x, y) + vec2(0.0, -8.0), Vec2::zeros(), rgb(0, 0, 0)),
                    vertex(vec2(x, y) + vec2(1.0, -8.0), Vec2::zeros(), rgb(0, 0, 0)),
                    vertex(vec2(x, y) + vec2(1.0,  9.0), Vec2::zeros(), rgb(0, 0, 0)),
                    vertex(vec2(x, y) + vec2(0.0,  9.0), Vec2::zeros(), rgb(0, 0, 0)),
                ],
                [
                    vertex(vec2(x, y) + vec2(-8.0, 0.0), Vec2::zeros(), rgb(0, 0, 0)),
                    vertex(vec2(x, y) + vec2( 9.0, 0.0), Vec2::zeros(), rgb(0, 0, 0)),
                    vertex(vec2(x, y) + vec2( 9.0, 1.0), Vec2::zeros(), rgb(0, 0, 0)),
                    vertex(vec2(x, y) + vec2(-8.0, 1.0), Vec2::zeros(), rgb(0, 0, 0)),
                ],
            ]);

            context.draw(self.batch.all(), &screen);
        }

        // time precision test
        if false {
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
