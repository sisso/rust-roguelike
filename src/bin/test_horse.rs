use approx::AbsDiffEq;
use rltk::VirtualKeyCode::P;
use rltk::{BTerm, GameState, Point, VirtualKeyCode};

#[derive(Debug)]
struct State {
    player_pos: Point,
    player_last_pos: Point,
}

impl State {
    fn get_player_dir(&self) -> Point {
        self.player_pos - self.player_last_pos
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        // draw map
        for y in 0..50 {
            for x in 0..80 {
                ctx.set(x, y, rltk::GRAY, rltk::BLACK, rltk::to_cp437('.'));
            }
        }

        let desire_move_dir = match ctx.key {
            Some(VirtualKeyCode::Numpad7) => Some(Point::new(-1, -1)),
            Some(VirtualKeyCode::Numpad8) => Some(Point::new(0, -1)),
            Some(VirtualKeyCode::Numpad9) => Some(Point::new(1, -1)),
            Some(VirtualKeyCode::Numpad4) => Some(Point::new(-1, 0)),
            Some(VirtualKeyCode::Numpad5) => Some(Point::new(0, 0)),
            Some(VirtualKeyCode::Numpad6) => Some(Point::new(1, 0)),
            Some(VirtualKeyCode::Numpad1) => Some(Point::new(-1, 1)),
            Some(VirtualKeyCode::Numpad2) => Some(Point::new(0, 1)),
            Some(VirtualKeyCode::Numpad3) => Some(Point::new(1, 1)),
            _ => None,
        };

        if let Some(desire_move_dir) = desire_move_dir {
            let player_dir = self.get_player_dir();
            let player_dir_v2 = glam::Vec2::new(player_dir.x as f32, player_dir.y as f32);
            let desire_move_dir_v2 =
                glam::Vec2::new(desire_move_dir.x as f32, desire_move_dir.y as f32);

            let angle_diff = desire_move_dir_v2.dot(player_dir_v2);
            if angle_diff >= 1.0 {
                log::info!("angle {:?}", angle_diff);
                self.player_last_pos = self.player_pos;
                self.player_pos = self.player_pos + desire_move_dir;
            } else {
                log::info!("bad angle {:?}", angle_diff);
            }
        }

        ctx.set(
            self.player_pos.x,
            self.player_pos.y,
            rltk::WHITE,
            rltk::BLACK,
            rltk::to_cp437('@'),
        );
        ctx.set(
            self.player_last_pos.x,
            self.player_last_pos.y,
            rltk::GRAY,
            rltk::BLACK,
            rltk::to_cp437('@'),
        );

        let front = self.player_pos + self.get_player_dir();

        ctx.set(
            front.x,
            front.y,
            rltk::GRAY,
            rltk::BLACK,
            rltk::to_cp437('@'),
        );
    }
}

fn main() -> rltk::BError {
    use rltk::RltkBuilder;

    env_logger::builder()
        .filter(None, log::LevelFilter::Debug)
        .init();

    let context = RltkBuilder::simple80x50().with_title("Space RL").build()?;

    let state = State {
        player_pos: Point::new(40, 25),
        player_last_pos: Point::new(40, 26),
    };

    rltk::main_loop(context, state)
}
