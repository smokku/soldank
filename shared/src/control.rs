use bitflags::bitflags;
use nanoserde::{DeBin, DeBinErr, SerBin};

bitflags! {
    #[derive(Default)]
    pub struct Control: u32 {
        const LEFT                 = 0b00000000000000000000000000000001;
        const RIGHT                = 0b00000000000000000000000000000010;
        const UP                   = 0b00000000000000000000000000000100;
        const DOWN                 = 0b00000000000000000000000000001000;
        const FIRE                 = 0b00000000000000000000000000010000;
        const JETS                 = 0b00000000000000000000000000100000;
        const GRENADE              = 0b00000000000000000000000001000000;
        const CHANGE               = 0b00000000000000000000000010000000;
        const THROW                = 0b00000000000000000000000100000000;
        const DROP                 = 0b00000000000000000000001000000000;
        const RELOAD               = 0b00000000000000000000010000000000;
        const PRONE                = 0b00000000000000000000100000000000;
        const FLAG_THROW           = 0b00000000000000000001000000000000;

        const WAS_RUNNING_LEFT     = 0b10000000000000000000000000000000;
        const WAS_JUMPING          = 0b01000000000000000000000000000000;
        const WAS_THROWING_WEAPON  = 0b00100000000000000000000000000000;
        const WAS_CHANGING_WEAPON  = 0b00010000000000000000000000000000;
        const WAS_THROWING_GRENADE = 0b00001000000000000000000000000000;
        const WAS_RELOADING_WEAPON = 0b00000100000000000000000000000000;
    }
}

impl SerBin for Control {
    fn ser_bin(&self, output: &mut Vec<u8>) {
        let val = self.bits();
        val.ser_bin(output);
    }
}

impl DeBin for Control {
    fn de_bin(offset: &mut usize, bytes: &[u8]) -> Result<Self, DeBinErr> {
        let val = u32::de_bin(offset, bytes)?;
        match Control::from_bits(val) {
            Some(control) => Ok(control),
            None => Err(DeBinErr {
                o: *offset,
                l: std::mem::size_of::<u32>(),
                s: bytes.len(),
            }),
        }
    }
}
