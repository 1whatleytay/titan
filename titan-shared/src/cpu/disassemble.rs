pub trait LabelProvider {
  fn label_for(&mut self, address: u32) -> String;
}

#[derive(Default)]
pub struct HexLabelProvider {}

impl LabelProvider for HexLabelProvider {
  fn label_for(&mut self, address: u32) -> String {
      format!("0x{address:08x}")
  }
}

pub struct Disassembler<Provider: LabelProvider> {
  pub pc: u32,
  pub labels: Provider,
}

pub trait Dispatchable<T, Provider: LabelProvider> {
  fn new(pc: u32, labels: &mut Provider) -> Self;
  fn dispatch(&mut self, instruction: u32) -> Option<T>;
  fn update_pc(&mut self);
}
