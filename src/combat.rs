use rand::rngs::StdRng;
use rand::Rng;

#[derive(Clone, Debug, Copy)]
pub struct Dice {
    pub dices: u32,
    pub bonus: i32,
}

pub type DiceRoll = i32;

impl Dice {
    pub fn roll(&self, rng: &mut StdRng) -> DiceRoll {
        (0..self.dices)
            .into_iter()
            .map(|_| rng.gen_range(1..=6))
            .sum::<i32>()
            + self.bonus
    }
}

impl Dice {
    pub fn new(dices: u32, bonus: i32) -> Dice {
        Self { dices, bonus }
    }
}

#[derive(Clone, Debug)]
pub struct CombatStats {
    pub attack: Dice,
    pub defense: Dice,
    pub damage: Dice,
}

#[derive(Clone, Debug)]
pub enum CombatResult {
    AttackHit {
        hit_roll: i32,
        hit_require: i32,
        damage_roll: i32,
    },
    Defend {
        hit_roll: i32,
        hit_require: i32,
    },
}

pub fn execute_attack(
    rng: &mut StdRng,
    attacker: &CombatStats,
    defender: Option<&CombatStats>,
) -> CombatResult {
    let attack_roll = attacker.attack.roll(rng);
    let defense_roll = defender.map(|d| d.defense.roll(rng)).unwrap_or(1);
    if attack_roll > defense_roll {
        let damage = attacker.damage.roll(rng);

        CombatResult::AttackHit {
            hit_roll: attack_roll,
            hit_require: defense_roll,
            damage_roll: damage,
        }
    } else {
        CombatResult::Defend {
            hit_roll: attack_roll,
            hit_require: defense_roll,
        }
    }
}
