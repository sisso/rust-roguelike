#[derive(Clone, Debug)]
pub enum Msg {
    PlayerAttack,
    PlayerReceiveAttack,
    Died,
    ShipLand,
    ShipLaunch,
    PlayerMove,
    PlayerFailMove,
}

impl Msg {
    pub fn to_string(&self) -> String {
        match self {
            Msg::PlayerAttack { .. } => format!("player attack a mob"),
            Msg::Died { .. } => format!("mob dies"),
            Msg::ShipLand => format!("ship landed into planet"),
            Msg::ShipLaunch => format!("ship launch into space"),
            Msg::PlayerMove => "move".to_string(),
            Msg::PlayerFailMove => "fail to move".to_string(),
            Msg::PlayerReceiveAttack => "receive damage".to_string(),
        }
    }

    pub fn bg(&self) -> (u8, u8, u8) {
        rltk::BLACK
    }

    pub fn fg(&self) -> (u8, u8, u8) {
        rltk::WHITE
    }
}

#[derive(Clone, Debug, Default)]
pub struct GameLog {
    queue: Vec<Msg>,
}

impl GameLog {
    pub fn list(&self) -> &Vec<Msg> {
        &self.queue
    }
}

impl GameLog {
    pub fn push(&mut self, msg: Msg) {
        self.queue.push(msg);
    }

    pub fn take(&mut self) -> Vec<Msg> {
        std::mem::replace(&mut self.queue, Vec::new())
    }
}
