use crate::loader::NewGameParams;
use crate::state::State;
use crate::view::window::Window;
use crate::{cfg, loader, view};
use rltk::{Rltk, VirtualKeyCode, RGB};

pub fn run(state: &mut State, ctx: &mut Rltk) {
    let menu_rect = view::get_window_rect().shrink(2);
    view::draw_rect(ctx, menu_rect, rltk::WHITE, rltk::BLACK);

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
        "2) orbiting",
    );
    y += 1;
    ctx.print_color(
        internal_rect.get_x(),
        y,
        rltk::WHITE,
        rltk::BLACK,
        "3) landed",
    );
    y += 1;
    ctx.print_color(
        internal_rect.get_x(),
        y,
        rltk::WHITE,
        rltk::BLACK,
        "4) test view",
    );
    y += 1;

    if let Some(key) = ctx.key {
        match key {
            VirtualKeyCode::Key1 => {
                loader::start_game(state, &NewGameParams::Normal);
                state.window_manage.set_window(Window::World);
            }
            VirtualKeyCode::Key2 => {
                loader::start_game(state, &NewGameParams::Orbiting);
                state.window_manage.set_window(Window::World);
            }
            VirtualKeyCode::Key3 => {
                loader::start_game(state, &NewGameParams::Landed);
                state.window_manage.set_window(Window::World);
            }
            _ => {}
        }
    }
}
