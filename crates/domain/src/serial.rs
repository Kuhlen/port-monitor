#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortInfo {
    pub name: String,
    pub port_type: String,
}

// typed fields: the old String wire format and its validate() are gone
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SerialConfig {
    pub port: String,
    pub baud_rate: u32,
    pub data_bits: DataBits,
    pub parity: Parity,
    pub stop_bits: StopBits,
    pub flow: FlowControl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DataBits {
    Five,
    Six,
    Seven,
    #[default]
    Eight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Parity {
    #[default]
    None,
    Even,
    Odd,
}

// 1.5 dropped: serialport never accepted it (spec D6)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StopBits {
    #[default]
    One,
    Two,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FlowControl {
    #[default]
    None,
    Hardware,
    Software,
}

// ALL order = segment order in frame_popup.slint
impl DataBits {
    pub const ALL: [Self; 4] = [Self::Five, Self::Six, Self::Seven, Self::Eight];
}
impl Parity {
    pub const ALL: [Self; 3] = [Self::None, Self::Even, Self::Odd];
}
impl StopBits {
    pub const ALL: [Self; 2] = [Self::One, Self::Two];
}
impl FlowControl {
    pub const ALL: [Self; 3] = [Self::None, Self::Hardware, Self::Software];
}
