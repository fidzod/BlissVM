#[derive(Default)]
pub struct Storage(Vec<u8>);

const SECTOR_SIZE: usize = 512;

impl Storage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_data(data: Vec<u8>) -> Self {
        Self(data)
    }

    pub fn read_sector(&self, sector: u32) -> [u8; SECTOR_SIZE] {
        let mut out = [0u8; SECTOR_SIZE];
        let base = (sector as usize) * SECTOR_SIZE;
        let end = (base + SECTOR_SIZE).min(self.0.len());

        if base < self.0.len() {
            out[..end - base].copy_from_slice(&self.0[base..end]);
        }

        out
    }
}
