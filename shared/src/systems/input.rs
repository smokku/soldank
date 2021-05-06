use hecs::EntityRef;

use crate::components::*;

pub fn apply_input(entity: EntityRef, control: &ControlComponent) {
    if let Some(position) = entity.get_mut::<Position>() {
        log::debug!("position {:?} / {:?}", *position, control);
    }
}
