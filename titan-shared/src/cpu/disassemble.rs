pub trait LabelProvider {
    fn label_for(&mut self, address: u32) -> String;
}

impl<T: LabelProvider> LabelProvider for &mut T {
    fn label_for(&mut self, address: u32) -> String {
        (**self).label_for(address)
    }
}

#[derive(Default)]
pub struct HexLabelProvider;

impl LabelProvider for HexLabelProvider {
    fn label_for(&mut self, address: u32) -> String {
        format!("0x{address:08x}")
    }
}
