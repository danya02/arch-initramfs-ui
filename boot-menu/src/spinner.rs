use std::time::Duration;

use cursive::{
    reexports::enumset::EnumSet, theme::Style, utils::markup::StyledString, view::Resizable, views,
    View,
};

pub fn spinner_view() -> impl View {
    views::Canvas::new(std::time::Instant::now())
        .with_draw(move |c, printer| {
            let elapsed = c.elapsed();
            let tick_duration = Duration::from_millis(100);
            let elapsed_ticks = elapsed.div_duration_f32(tick_duration);
            let elapsed_ticks = elapsed_ticks.round() as usize;
            let positions = [
                (0, 0),
                (2, 0),
                (4, 0),
                (4, 1),
                (4, 2),
                (2, 2),
                (0, 2),
                (0, 1),
            ];
            let tick_state = elapsed_ticks % positions.len();

            let bg_style = Style {
                effects: EnumSet::EMPTY,
                color: cursive::theme::ColorStyle {
                    front: cursive::theme::ColorType::Color(cursive::theme::Color::Light(
                        cursive::theme::BaseColor::Red,
                    )),
                    back: cursive::theme::ColorType::Color(cursive::theme::Color::Light(
                        cursive::theme::BaseColor::Red,
                    )),
                },
            };
            let bg_text = StyledString::styled("  ", bg_style);
            let fg_style = Style {
                effects: EnumSet::EMPTY,
                color: cursive::theme::ColorStyle {
                    front: cursive::theme::ColorType::Color(cursive::theme::Color::Dark(
                        cursive::theme::BaseColor::Black,
                    )),
                    back: cursive::theme::ColorType::Color(cursive::theme::Color::Dark(
                        cursive::theme::BaseColor::Black,
                    )),
                },
            };
            let fg_text = StyledString::styled("  ", fg_style);

            for (i, pos) in positions.iter().enumerate() {
                if tick_state == i {
                    printer.print_styled(*pos, &fg_text);
                } else {
                    printer.print_styled(*pos, &bg_text);
                }
            }
        })
        .fixed_size((6, 3))
}
