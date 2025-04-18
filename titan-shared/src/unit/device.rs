use crate::assembler::binary::{Binary, RawRegion, RegionFlags};

impl Binary {
  pub fn mount_data(&mut self, address: u32, data: Vec<u8>) {
      self.regions.push(RawRegion {
          flags: RegionFlags::all(),
          address,
          data,
      })
  }

  pub fn mount_constant(&mut self, address: u32, count: usize, constant: u8) {
      self.mount_data(address, vec![constant; count])
  }

  pub fn mount(&mut self, address: u32, count: usize) {
      self.mount_constant(address, count, 0)
  }

  pub fn mount_display(&mut self) {
      self.mount(0x10008000, 0x8000)
  }

  pub fn mount_keyboard(&mut self) {
      self.mount(0xFFFF0000, 0x100)
  }

  pub fn with_mount_data(mut self, address: u32, data: Vec<u8>) -> Self {
      self.mount_data(address, data);

      self
  }

  pub fn with_mount_constant(mut self, address: u32, count: usize, constant: u8) -> Self {
      self.mount_constant(address, count, constant);

      self
  }

  pub fn with_mount(mut self, address: u32, count: usize) -> Self {
      self.mount(address, count);

      self
  }

  pub fn with_mount_display(mut self) -> Self {
      self.mount_display();

      self
  }

  pub fn with_mount_keyboard(mut self) -> Self {
      self.mount_keyboard();

      self
  }
}
