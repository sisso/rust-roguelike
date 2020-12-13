extern crate rltk;
use rltk::{Rltk, GameState, Console};

struct State {
    frame: u32,
}

impl GameState for State {
    fn tick(&mut self, ctx : &mut Rltk) {
        self.frame += 1;
        
        ctx.cls();
        ctx.print(1, 1, &format!("frame: {}", self.frame));
    }
}

fn main() {
    let context = Rltk::init_simple8x8(160, 100, "Hello RLTK World", "resources");
    let gs = State{ frame: 0 };
    rltk::main_loop(context, gs);
}
