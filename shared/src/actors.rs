use std::{cell::RefCell, rc::Rc};

use naia_derive::ActorType;

use point::PointActor;

#[derive(ActorType, Clone)]
pub enum NetworkActor {
    PointActor(Rc<RefCell<PointActor>>),
}

pub mod point {
    use nanoserde::{DeBin, SerBin};

    use naia_derive::Actor;
    use naia_shared::{Actor, Property};

    use super::NetworkActor;

    #[derive(Clone, PartialEq, DeBin, SerBin)]
    pub enum PointActorColor {
        Red,
        Blue,
        Yellow,
    }

    impl Default for PointActorColor {
        fn default() -> Self {
            PointActorColor::Red
        }
    }

    #[derive(Actor)]
    #[type_name = "NetworkActor"]
    pub struct PointActor {
        #[interpolate]
        #[predict]
        pub x: Property<u16>,
        #[interpolate]
        #[predict]
        pub y: Property<u16>,
        pub color: Property<PointActorColor>,
    }

    impl PointActor {
        pub fn new(x: u16, y: u16, color: PointActorColor) -> PointActor {
            return PointActor::new_complete(x, y, color);
        }
    }
}
