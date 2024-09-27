use crate::game_log::{GameLog, Msg};
use hecs::{CommandBuffer, World};

pub type Hp = i32;

#[derive(Clone, Default, Debug)]
pub struct Health {
    pub hp: Hp,
    pub max_hp: Hp,
    pub pending_damage: Vec<Hp>,
}

pub fn run_health_system(world: &mut World, logs: &mut GameLog) {
    let mut buffer = CommandBuffer::new();
    for (e, health) in &mut world.query::<&mut Health>() {
        let total = health.pending_damage.iter().sum::<Hp>();
        health.pending_damage.clear();
        health.hp -= total;

        if health.hp <= 0 {
            buffer.despawn(e);
            logs.push(Msg::Died {});
        }
    }
    buffer.run_on(world);
}
