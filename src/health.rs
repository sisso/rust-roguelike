use crate::game_log::{GameLog, Msg};
use hecs::{CommandBuffer, World};

#[derive(Clone, Default, Debug)]
pub struct Health {
    pub hp: i32,
    pub pending_damage: Vec<i32>,
}

pub fn run_health_system(world: &mut World, logs: &mut GameLog) {
    let mut buffer = CommandBuffer::new();
    for (e, health) in &mut world.query::<&mut Health>() {
        let total = health.pending_damage.iter().sum::<i32>();
        health.pending_damage.clear();
        health.hp -= total;

        if health.hp <= 0 {
            buffer.despawn(e);
            logs.push(Msg::Died {});
        }
    }
    buffer.run_on(world);
}
