// https://github.com/TylerGlaiel/FrameTimingControl/blob/master/frame_timer.cpp
use super::*;

//these are loaded from Settings in production code
const UPDATE_RATE: f64 = 60.;

//compute how many ticks one update should be
const FIXED_DELTATIME: f64 = 1.0 / UPDATE_RATE;
pub(crate) const DESIRED_FRAMETIME: f64 = 1.0 / UPDATE_RATE;

//these are to snap deltaTime to vsync values if it's close enough
const VSYNC_MAXERROR: f64 = 0.02;
const TIME_60HZ: f64 = 1.0 / 60.; //since this is about snapping to common vsync values
const SNAP_FREQUENCIES: [f64; 5] = [
    TIME_60HZ,      //60fps
    TIME_60HZ * 2., //30fps
    TIME_60HZ * 3., //20fps
    TIME_60HZ * 4., //15fps
    (TIME_60HZ + 1.) / 2., //120fps //120hz, 240hz, or higher need to round up, so that adding 120hz twice guaranteed is at least the same as adding time_60hz once
                           // (time_60hz + 2.) / 3., //180fps //that's where the +1 and +2 come from in those equations
                           // (time_60hz + 3.) / 4., //240fps //I do not want to snap to anything higher than 120 in my engine, but I left the math in here anyway
];

impl<G: Game> Runner<G> {
    pub(crate) fn frame_timer(&mut self, ctx: &mut mq::Context) -> f64 {
        //frame timer
        let current_frame_time: f64 = mq::date::now();
        let mut delta_time = current_frame_time - self.frame_time;
        self.frame_time = current_frame_time;

        //handle unexpected timer anomalies (overflow, extra slow frames, etc)
        if delta_time > DESIRED_FRAMETIME * 8. {
            //ignore extra-slow frames
            delta_time = DESIRED_FRAMETIME;
        }
        if delta_time < 0. {
            delta_time = 0.;
        }

        //vsync time snapping
        for snap in SNAP_FREQUENCIES {
            if f64::abs(delta_time - snap) < VSYNC_MAXERROR {
                delta_time = snap;
                break;
            }
        }

        //delta time averaging
        self.time_averager.push(delta_time);
        delta_time = self.time_averager.iter().sum::<f64>() / self.time_averager.len() as f64;

        //add to the accumulator
        self.frame_accumulator += delta_time;

        //spiral of death protection
        if self.frame_accumulator > DESIRED_FRAMETIME * 8. {
            self.resync = true;
        }

        //timer resync if requested
        if self.resync {
            self.frame_accumulator = 0.;
            delta_time = DESIRED_FRAMETIME;
            self.resync = false;
        }

        let mut consumed_delta_time = delta_time;

        while self.frame_accumulator >= DESIRED_FRAMETIME {
            if consumed_delta_time > DESIRED_FRAMETIME {
                //cap variable update's dt to not be larger than fixed update
                let eng = Engine {
                    now: current_frame_time,
                    delta: FIXED_DELTATIME,
                    fps: self.fps(),
                    overstep_percentage: self.overstep_percentage,
                    quad_ctx: ctx,
                    egui_ctx: self.egui_mq.egui_ctx(),
                    mouse_over_ui: self.mouse_over_ui,
                    input: &mut self.input,
                    script: &mut self.script,
                    event_sender: &self.event_sender,
                };
                self.game.update(eng);
                consumed_delta_time -= DESIRED_FRAMETIME;
            }
            self.frame_accumulator -= DESIRED_FRAMETIME;
        }

        let eng = Engine {
            now: current_frame_time,
            delta: consumed_delta_time,
            fps: self.fps(),
            overstep_percentage: self.overstep_percentage,
            quad_ctx: ctx,
            egui_ctx: self.egui_mq.egui_ctx(),
            mouse_over_ui: self.mouse_over_ui,
            input: &mut self.input,
            script: &mut self.script,
            event_sender: &self.event_sender,
        };
        self.game.update(eng);

        // store frame_percentage to be used later by render
        self.frame_accumulator / DESIRED_FRAMETIME
    }
}
