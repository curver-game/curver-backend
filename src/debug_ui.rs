use std::{collections::HashMap, io::Stdout};

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
    game::{Game, GameOutcome},
};

pub struct DebugUi {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

const MAP_PERCENTAGE_WIDTH: f64 = 0.6;

impl DebugUi {
    pub fn new() -> Self {
        let stdout = std::io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend).unwrap();

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
                                y1: (MAP_HEIGHT - first_line.1).into(),
                                x2: second_line.0.into(),
                                y2: (MAP_HEIGHT - second_line.1).into(),
                                color: Color::LightBlue,
                            };

                            ctx.draw(&line);
                        }
                    }
                });

            let rect = Rect::new(
                0,
                0,
                (size.width as f64 * MAP_PERCENTAGE_WIDTH) as u16,
                size.height,
            );
            f.render_widget(canvas, rect);
        });
    }

    pub fn display_outcome(&mut self, outcome: GameOutcome) {
        self.terminal.draw(|f| {
            let size = f.size();

            // Display winner Uuid in the middle of the screen in a fancy widget
            let title = match outcome {
                GameOutcome::Winner { .. } => format!("Winner)"),
                GameOutcome::Tie => "Draw".to_string(),
            };

            let body = match outcome {
                GameOutcome::Winner { user_id } => format!("Player: {}", user_id.get_uuid()),
                GameOutcome::Tie => "No winner".to_string(),
            };

            let block = Block::default().title(title).borders(Borders::ALL);

            let text_widget = ratatui::widgets::Paragraph::new(body).block(block);
            let rect = Rect::new(
                (size.width as f64 * MAP_PERCENTAGE_WIDTH) as u16 / 2 - 5,
                size.height / 2 - 1,
                10,
                3,
            );
            f.render_widget(text_widget, rect);
        });
    }

    pub fn draw_rooms(&mut self, room_map: HashMap<Uuid, Uuid>) {
        self.terminal.draw(|f| {
            let size = f.size();

            let rooms: HashMap<Uuid, Vec<Uuid>> =
                room_map
                    .iter()
                    .fold(HashMap::new(), |mut acc, (user_id, room_id)| {
                        acc.entry(*room_id).or_insert_with(Vec::new).push(*user_id);
                        acc
                    });

            let canvas = Canvas::default()
                .block(Block::default().title("Rooms").borders(Borders::ALL))
                .x_bounds([0.0, size.width as f64 * MAP_PERCENTAGE_WIDTH])
                .y_bounds([0.0, size.height as f64])
                .paint(|ctx| {
                    let mut y = size.height as f64 - 1.0;
                    for (room_id, user_ids) in rooms.iter() {
                        // Draw room Ids as mini headers and list user ids with small padding.
                        // Also display the total number of users in the room.
                        let x = 0.0;
                        let room_id_str = format!("Room: {}", room_id);
                        ctx.print(x, y, room_id_str);
                        y -= 1.0;

                        for user_id in user_ids.iter() {
                            let user_id_str = format!("User: {}", user_id);
                            ctx.print(x + 2.0, y, user_id_str);
                            y -= 1.0;
                        }

                        let user_count_str = format!("User count: {}", user_ids.len());
                        ctx.print(x + 2.0, y, user_count_str);

                        y -= 1.0;

                        // Draw a line to separate rooms
                        let line = Line {
                            x1: 0.0,
                            y1: y as f64,
                            x2: size.width as f64 * MAP_PERCENTAGE_WIDTH * 0.5,
                            y2: y as f64,
                            color: Color::LightBlue,
                        };

                        y -= 1.0;

                        ctx.draw(&line);
                    }

                    y -= 1.0;

                    let total_user_count_str = format!("Total users: {}", room_map.len());
                    ctx.print(0.0, y, total_user_count_str);

                    y -= 1.0;

                    let room_count_str = format!("Total rooms: {}", rooms.len());
                    ctx.print(0.0, y, room_count_str);
                });

            let rect = Rect::new(
                (size.width as f64 * MAP_PERCENTAGE_WIDTH) as u16,
                0,
                (size.width as f64 * (1.0 - MAP_PERCENTAGE_WIDTH)) as u16,
                size.height,
            );
            f.render_widget(canvas, rect);
        });
    }

    pub fn clear(&mut self) {
        self.terminal.clear();
    }
}
