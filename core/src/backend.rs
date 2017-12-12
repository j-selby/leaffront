#[derive(Debug)]
pub struct Notification {
    pub name : String,
    pub contents : String
}

pub trait Backend {
    fn get_notification(&mut self) -> Option<Notification>;
}
