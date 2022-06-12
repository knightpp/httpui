use tui::{
    buffer, layout,
    style::Style,
    widgets::{Block, Paragraph, StatefulWidget, Widget},
};

const PIPE_FRAMES: &[&str] = &["|", "/", "-", "\\"];
const CLOCK_FRAMES: &[&str] = &[
    "🕐", "🕜", "🕑", "🕝", "🕒", "🕞", "🕓", "🕟", "🕔", "🕠", "🕕", "🕡", "🕖", "🕢", "🕗", "🕣",
    "🕘", "🕤", "🕙", "🕥", "🕚", "🕦", "🕛", "🕧",
];

#[derive(Default)]
pub struct Spinner<'a> {
    frames: &'a [&'a str],
    style: Option<Style>,
    block: Option<Block<'a>>,
}

impl<'a> Spinner<'a> {
    // pub fn new() -> Spinner<'a> {
    //     Default::default()
    // }

    pub fn pipes() -> Spinner<'a> {
        Spinner {
            frames: PIPE_FRAMES,
            ..Default::default()
        }
    }

    pub fn clock() -> Spinner<'a> {
        Spinner {
            frames: CLOCK_FRAMES,
            ..Default::default()
        }
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

#[derive(Default)]
pub struct SpinnerState {
    current: usize,
}

impl<'a> StatefulWidget for Spinner<'a> {
    type State = SpinnerState;

    fn render(self, area: layout::Rect, buf: &mut buffer::Buffer, state: &mut Self::State) {
        let frame = self.frames[state.current % self.frames.len()];
        state.current = (state.current + 1) % self.frames.len();
        let mut w = Paragraph::new(frame);

        if let Some(style) = self.style {
            buf.set_style(area, style);
        }
        if let Some(block) = self.block {
            w = w.block(block);
        }

        w.render(area, buf);
    }
}
