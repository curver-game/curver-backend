use std::io::Stdout;

use ratatui::{
    prelude::{CrosstermBackend, Rect},
    style::Color,
    widgets::{
        canvas::{Canvas, Line},
        Block, Borders,
    },
    Terminal,
};
use uuid::Uuid;

use crate::{
    constants::{MAP_HEIGHT, MAP_WIDTH},
    game::Game,
};

pub struct DebugUi {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl DebugUi {
    pub fn new() -> Self {
        let stdout = std::io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.clear();

        Self { terminal }
    }

    pub fn draw_game(&mut self, game: &Game) {
        self.terminal.draw(|f| {
            let size = f.size();

            let canvas = Canvas::default()
                .block(Block::default().title("Game").borders(Borders::ALL))
                .x_bounds([0.0, MAP_WIDTH as f64])
                .y_bounds([0.0, MAP_HEIGHT as f64])
                .paint(|ctx| {
                    for path in game.paths.values() {
                        if path.nodes.len() < 2 {
                            continue;
                        }

                        for i in 0..path.nodes.len() - 1 {
                            let first_line = &path.nodes[i];
                            let second_line = &path.nodes[i + 1];
                            let line = Line {
                                x1: first_line.0.into(),
                                y1: first_line.1.into(),
                                x2: second_line.0.into(),
                                y2: second_line.1.into(),
                                color: Color::LightBlue,
                            };

                            ctx.draw(&line);
                        }
                    }
                });

            let rect = Rect::new((size.width - size.height) / 2, 0, size.height, size.height);
            f.render_widget(canvas, rect);
        });
    }

    pub fn display_winner(&mut self, winner_id: &Uuid) {
        self.terminal.clear();

        self.terminal.draw(|f| {
            let size = f.size();

            // Display winner Uuid in the middle of the screen in a fancy widget
            let block = Block::default().title("Winner").borders(Borders::ALL);
            let text = format!("{}", winner_id);
            let text_widget = ratatui::widgets::Paragraph::new(text).block(block);
            let rect = Rect::new(
                (size.width - size.height) / 2,
                size.height / 2 - 1,
                size.height,
                3,
            );
            f.render_widget(text_widget, rect);
        });
    }
}
