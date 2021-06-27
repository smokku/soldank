use bitflags::bitflags;
use nanoserde::{DeBin, DeBinErr, SerBin};

bitflags! {
    #[derive(Default)]
    pub struct Control: u16 {
        const LEFT                 = 0b0000000000000001;
        const RIGHT                = 0b0000000000000010;
        const UP                   = 0b0000000000000100;
        const DOWN                 = 0b0000000000001000;
        const FIRE                 = 0b0000000000010000;
        const JETS                 = 0b0000000000100000;
        const GRENADE              = 0b0000000001000000;
        const CHANGE               = 0b0000000010000000;
        const THROW                = 0b0000000100000000;
        const DROP                 = 0b0000001000000000;
        const RELOAD               = 0b0000010000000000;
        const PRONE                = 0b0000100000000000;
        const FLAG_THROW           = 0b0001000000000000;
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
        let val = u16::de_bin(offset, bytes)?;
        match Control::from_bits(val) {
            Some(control) => Ok(control),
            None => Err(DeBinErr {
                o: *offset,
                l: std::mem::size_of::<u16>(),
                s: bytes.len(),
            }),
        }
    }
}
