#[derive(Clone)]
pub struct PortConnection {
    pub audio_out: String,
    pub audio_in: String
}
impl PortConnection {
    pub fn new(paudio_in: &str, paudio_out: &str) -> PortConnection {
        PortConnection{
            audio_in: String::from(paudio_in),
            audio_out: String::from(paudio_out)
        }
    }
}