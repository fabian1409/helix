use crate::compositor::{Component, Context};
use helix_view::graphics::{Margin, Rect};
use helix_view::snipe::Snipe;
use tui::buffer::Buffer as Surface;
use tui::text::{Span, Spans, Text};
use tui::widgets::{Block, BorderType, Paragraph, Widget};

impl Component for Snipe {
    fn render(&mut self, viewport: Rect, surface: &mut Surface, cx: &mut Context) {
        let text_style = cx.editor.theme.get("ui.text.info");
        let popup_style = cx.editor.theme.get("ui.popup.info");
        let key_style = cx.editor.theme.get("keyword");

        // Calculate the area of the terminal to modify. Because we want to
        // render at the bottom right, we use the viewport's width and height
        // which evaluate to the most bottom right coordinate.
        let width = u16::max(self.width + 2 + 2, (viewport.width as f64 * 0.4f64) as u16); // +2 for border, +2 for margin
        let height = u16::max(self.height + 2, (viewport.height as f64 * 0.4f64) as u16); // +2 for border
        let area = viewport.intersection(Rect::new(
            (viewport.width / 2).saturating_sub(width / 2),
            (viewport.height / 2).saturating_sub((height + 2) / 2), // +2 for statusline
            width,
            height,
        ));
        surface.clear_with(area, popup_style);

        let border_type = if cx.editor.config().rounded_corners {
            BorderType::Rounded
        } else {
            BorderType::Plain
        };
        let block = Block::bordered()
            .title(self.title.as_str())
            .border_style(popup_style)
            .border_type(border_type);

        let margin = Margin::horizontal(1);
        let inner = block.inner(area).inner(margin);
        block.render(area, surface);

        let lines = self
            .items
            .iter()
            .map(|(k, v)| {
                Spans::from(vec![
                    Span::from("("),
                    Span::styled(k, key_style),
                    Span::from(")"),
                    Span::from("  "),
                    Span::styled(v, text_style),
                ])
            })
            .collect::<Vec<_>>();

        Paragraph::new(&Text::from(lines))
            .style(text_style)
            .render(inner, surface);
    }
}
