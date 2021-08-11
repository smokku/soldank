use bitflags::bitflags;

bitflags! {
    #[derive(Default)]
    pub struct Cheats: u32 {
        const MOUSE_AIM = 0b00000000000000000000000000000001;
    }
}
