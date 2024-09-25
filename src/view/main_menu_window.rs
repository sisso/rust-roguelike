use crate::state::State;
use crate::view::window::Window;
use crate::{cfg, loader, view};
use rltk::{Rltk, VirtualKeyCode, RGB};

pub fn run(state: &mut State, ctx: &mut Rltk) {
    let menu_rect = view::get_window_rect().shrink(2);
    view::draw_rect(ctx, &menu_rect, rltk::WHITE, rltk::BLACK);

    let internal_rect = menu_rect.shrink(2);
    let mut y = internal_rect.get_y() + 1;
    ctx.print_color(
        internal_rect.get_x(),
        y,
        rltk::WHITE,
        rltk::BLACK,
        "1) main game",
    );
    y += 1;
    ctx.print_color(
        internal_rect.get_x(),
        y,
        rltk::WHITE,
        rltk::BLACK,
        "2) landed",
    );
    y += 1;
    ctx.print_color(
        internal_rect.get_x(),
        y,
        rltk::WHITE,
        rltk::BLACK,
        "3) test view",
    );
    y += 1;

    if let Some(key) = ctx.key {
        match key {
            VirtualKeyCode::Key1 => {
                state.clear();
                loader::start_game(state);
                state.window = Window::World;
            }
            VirtualKeyCode::Key2 => {}
            VirtualKeyCode::Key3 => {}
            _ => {}
        }
    }
}
