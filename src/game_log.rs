#[derive(Clone, Debug)]
pub enum Msg {
    PlayerAttack {
        hit_roll: i32,
        hit_require: i32,
        damage: i32,
    },
    PlayerMissAttack {
        hit_roll: i32,
        hit_require: i32,
    },
    PlayerReceiveAttack {
        hit_roll: i32,
        hit_require: i32,
        damage: i32,
    },
    PlayerReceiveMissAttack {
        hit_roll: i32,
        hit_require: i32,
    },
    Died,
    ShipLand,
    ShipLaunch,
    PlayerMove,
    PlayerFailMove,
}

impl Msg {
    pub fn to_string(&self) -> String {
        match self {
            Msg::PlayerAttack {
                hit_roll,
                hit_require,
                damage,
            } => format!(
                "attack a mob, roll {} from {}, cause damage of {}",
                hit_roll, hit_require, damage
            ),
            Msg::Died { .. } => format!("mob dies"),
            Msg::ShipLand => format!("ship landed into planet"),
            Msg::ShipLaunch => format!("ship launch into space"),
            Msg::PlayerMove => "move".to_string(),
            Msg::PlayerFailMove => "fail to move".to_string(),
            Msg::PlayerReceiveAttack {
                hit_roll,
                hit_require,
                damage,
            } => format!(
                "hit, roll {} from {}, cause damage of {}",
                hit_require, hit_roll, damage
            ),
            Msg::PlayerMissAttack {
                hit_roll,
                hit_require,
            } => format!("miss, roll {} from {}", hit_roll, hit_require),
            Msg::PlayerReceiveMissAttack {
                hit_roll,
                hit_require,
            } => format!("defend, roll {} from {}", hit_require, hit_roll),
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
