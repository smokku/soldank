use super::*;

impl<G: Game> mq::EventHandler for Runner<G> {
    fn update(&mut self, ctx: &mut mq::Context) {
        self.egui_mq.begin_frame(ctx);

        // spin the Game::update() needed number of frames
        self.overstep_percentage = self.frame_timer(ctx) as f32;

        self.script.drain_events();

        assert_eq!(
            self.input.queue.len(),
            0,
            "Undrained Engine::input.events after Game::update()"
        );
    }

    fn draw(&mut self, ctx: &mut mq::Context) {
        self.last_frame = mq::date::now();
        let last_second = self.last_frame.round();
        if (self.fps_second - last_second).abs() > f64::EPSILON {
            self.fps.push(self.fps_count);
            self.fps_second = last_second;
            self.fps_count = 0;
        }
        self.fps_count += 1;

        let eng = Engine {
            delta: 0.,
            fps: self.fps(),
            overstep_percentage: self.overstep_percentage,
            quad_ctx: ctx,
            egui_ctx: self.egui_mq.egui_ctx(),
            mouse_over_ui: self.mouse_over_ui,
            input: &mut self.input,
            script: &mut self.script,
            event_sender: &self.event_sender,
        };

        self.game.draw(eng);

        self.egui_mq.end_frame(ctx);
        {
            let mouse_over_ui = self.egui_mq.egui_ctx().wants_pointer_input();
            if self.mouse_over_ui != mouse_over_ui {
                self.mouse_over_ui = mouse_over_ui;
                ctx.show_mouse(self.mouse_over_ui);
            }
        }
        self.egui_mq.draw(ctx);

        ctx.commit_frame();
    }

    fn resize_event(&mut self, _ctx: &mut mq::Context, _width: f32, _height: f32) {}

    fn mouse_motion_event(&mut self, ctx: &mut mq::Context, x: f32, y: f32) {
        self.egui_mq.mouse_motion_event(ctx, x, y);
        self.input.set_mouse_position(x, y);
    }

    fn mouse_wheel_event(&mut self, ctx: &mut mq::Context, dx: f32, dy: f32) {
        self.egui_mq.mouse_wheel_event(ctx, dx, dy);
        if !self.mouse_over_ui {
            self.input.add_event(input::InputEvent::Wheel { dx, dy });
            if dx < 0. {
                self.handle_bind(
                    &KeyBind::Wheel(Direction::Left),
                    mq::KeyMods::default(),
                    true,
                );
            }
            if dx > 0. {
                self.handle_bind(
                    &KeyBind::Wheel(Direction::Right),
                    mq::KeyMods::default(),
                    true,
                );
            }
            if dy < 0. {
                self.handle_bind(&KeyBind::Wheel(Direction::Up), mq::KeyMods::default(), true);
            }
            if dy > 0. {
                self.handle_bind(
                    &KeyBind::Wheel(Direction::Down),
                    mq::KeyMods::default(),
                    true,
                );
            }
        }
    }

    fn mouse_button_down_event(
        &mut self,
        ctx: &mut mq::Context,
        button: mq::MouseButton,
        x: f32,
        y: f32,
    ) {
        self.egui_mq.mouse_button_down_event(ctx, button, x, y);
        if !self.mouse_over_ui {
            self.input.add_event(input::InputEvent::Mouse {
                down: true,
                button,
                x,
                y,
            });
            self.handle_bind(&KeyBind::Mouse(button), mq::KeyMods::default(), true);
        }
    }

    fn mouse_button_up_event(
        &mut self,
        ctx: &mut gfx2d::Context,
        button: mq::MouseButton,
        x: f32,
        y: f32,
    ) {
        self.egui_mq.mouse_button_up_event(ctx, button, x, y);
        if !self.mouse_over_ui {
            self.input.add_event(input::InputEvent::Mouse {
                down: false,
                button,
                x,
                y,
            });
            self.handle_bind(&KeyBind::Mouse(button), mq::KeyMods::default(), false);
        }
    }

    fn char_event(
        &mut self,
        _ctx: &mut mq::Context,
        character: char,
        _keymods: mq::KeyMods,
        _repeat: bool,
    ) {
        if self.mouse_over_ui || self.egui_mq.egui_ctx().wants_keyboard_input() {
            self.egui_mq.char_event(character);
        }
    }

    fn key_down_event(
        &mut self,
        ctx: &mut mq::Context,
        keycode: mq::KeyCode,
        keymods: mq::KeyMods,
        repeat: bool,
    ) {
        self.egui_mq.key_down_event(ctx, keycode, keymods);
        if !self.egui_mq.egui_ctx().wants_keyboard_input() {
            self.input.add_event(input::InputEvent::Key {
                down: true,
                keycode,
                keymods,
                repeat,
            });
            self.handle_bind(&KeyBind::Key(keycode), keymods, true);
        }

        match keycode {
            mq::KeyCode::Escape => ctx.request_quit(),
            _ => {}
        }
    }

    fn key_up_event(
        &mut self,
        ctx: &mut gfx2d::Context,
        keycode: mq::KeyCode,
        keymods: mq::KeyMods,
    ) {
        self.egui_mq.key_up_event(keycode, keymods);
        if !self.egui_mq.egui_ctx().wants_keyboard_input() {
            self.input.add_event(input::InputEvent::Key {
                down: false,
                keycode,
                keymods,
                repeat: false,
            });
            self.handle_bind(&KeyBind::Key(keycode), keymods, false);
        }

        match keycode {
            mq::KeyCode::Escape => ctx.request_quit(),
            _ => {}
        }
    }

    fn touch_event(
        &mut self,
        ctx: &mut mq::Context,
        phase: mq::TouchPhase,
        _id: u64,
        x: f32,
        y: f32,
    ) {
        if phase == mq::TouchPhase::Started {
            self.mouse_button_down_event(ctx, mq::MouseButton::Left, x, y);
        }

        if phase == mq::TouchPhase::Ended {
            self.mouse_button_up_event(ctx, mq::MouseButton::Left, x, y);
        }

        if phase == mq::TouchPhase::Moved {
            self.mouse_motion_event(ctx, x, y);
        }
    }

    fn raw_mouse_motion(&mut self, _ctx: &mut mq::Context, _dx: f32, _dy: f32) {}

    fn window_minimized_event(&mut self, _ctx: &mut mq::Context) {}

    fn window_restored_event(&mut self, _ctx: &mut mq::Context) {}

    fn quit_requested_event(&mut self, _ctx: &mut mq::Context) {}
}
