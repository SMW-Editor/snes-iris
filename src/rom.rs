#[derive(Clone)]
pub struct Rom {
    cart: Vec<u8>,
    mapper: Mapper
}

#[derive(Clone)]
pub enum Mapper {
    LoRom,
    HiRom,
    Sa1
}

impl Rom {
    pub fn new(cart: Vec<u8>, mapper: Mapper) -> Self {
        Self { cart, mapper }
    }
    pub fn load(&self, addr: u32) -> u8 {
        let off = Self::map_rom(addr);
        self.cart[off]
    }
    pub fn load_u16(&self, addr: u32) -> u16 {
        let off = Self::map_rom(addr);
        u16::from_le_bytes([self.cart[off], self.cart[off+1]])
    }
    pub fn load_u24(&self, addr: u32) -> u32 {
        let off = Self::map_rom(addr);
        u32::from_le_bytes([self.cart[off], self.cart[off+1], self.cart[off+2], 0])
    }
    pub fn load_u32(&self, addr: u32) -> u32 {
        let off = Self::map_rom(addr);
        u32::from_le_bytes([self.cart[off], self.cart[off+1], self.cart[off+2], self.cart[off+3]])
    }
    pub fn slice(&self, addr: u32) -> &[u8] {
        let off = Self::map_rom(addr);
        let end = (off & 0xFF8000) + 0x8000;
        &self.cart[off..end]
    }
    pub fn map_rom(addr: u32) -> usize {
        // TODO: mapper support
        let mut bank = (addr >> 16) & 0x3F;
        if bank & 0x30 == 0x30 { bank &= !0x10; }
        let addr = addr & 0x7FFF;
        (bank << 15 | addr) as _
    }
}
