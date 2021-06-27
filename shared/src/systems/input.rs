use hecs::EntityRef;

use crate::{components::*, control::Control};

pub fn apply_input(entity: EntityRef, control: Control) {
    if let Some(mut position) = entity.get_mut::<Position>() {
        if control.contains(Control::LEFT) {
            position.x -= 1;
        }
        if control.contains(Control::RIGHT) {
            position.x += 1;
        }
        if control.contains(Control::UP) {
            position.y -= 1;
        }
        if control.contains(Control::DOWN) {
            position.y += 1;
        }
        log::trace!("position {:?} / {:?}", *position, control);
    }
}
