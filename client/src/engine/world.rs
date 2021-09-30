use crate::render::components::{Camera, Position};
use hecs::{Entity, World};
use simple_error::{bail, SimpleError};

pub trait WorldCameraExt {
    fn make_active_camera(&mut self, entity: Entity) -> Result<(), SimpleError>;
    fn get_active_camera(&self) -> Option<Entity>;
    fn get_camera_and_camera_position(&self) -> (Camera, Position);
}

impl WorldCameraExt for World {
    fn make_active_camera(&mut self, entity: Entity) -> Result<(), SimpleError> {
        let mut set_camera = false;
        if let Ok(mut camera) = self.get_mut::<Camera>(entity) {
            camera.is_active = true;
            set_camera = true;
        }

        if set_camera {
            for (id, mut camera_to_disable) in self.query::<&mut Camera>().iter() {
                if id != entity {
                    camera_to_disable.is_active = false;
                }
            }

            return Ok(());
        }

        bail!(
            "Entity {:?} either does not exist or does not hold a camera",
            entity
        );
    }

    fn get_active_camera(&self) -> Option<Entity> {
        let mut cam = None;

        for (id, camera) in self.query::<&Camera>().iter() {
            if camera.is_active {
                cam = Some(id);
                break;
            }
        }

        cam
    }

    fn get_camera_and_camera_position(&self) -> (Camera, Position) {
        let mut cam = Camera::default();
        let mut cam_position = Position::new(0.0, 0.0);
        let mut entity_holding_camera: Option<Entity> = None;

        for (id, camera) in self.query::<&Camera>().iter() {
            if camera.is_active {
                cam = *camera;
                entity_holding_camera = Some(id);
            }
        }

        if let Some(entity) = entity_holding_camera {
            if let Ok(position) = self.get_mut::<Position>(entity) {
                cam_position = *position;
            }
        }

        (cam, cam_position)
    }
}
