use std::{collections::HashMap, sync::Arc};

use parking_lot::RwLock;
use tokio::sync::mpsc::Receiver;
use uuid::Uuid;

use crate::{
    curver_ws_actor::CurverAddress,
    debug_ui::DebugUi,
    game::{Game, MS_PER_TICK},
    message::{CurverMessageToReceive, ForwardedMessage},
};

pub struct Room {
    pub clients: Arc<RwLock<HashMap<Uuid, CurverAddress>>>,
    pub id: Uuid,
    pub receiver: Receiver<ForwardedMessage>,
}

impl Room {
    pub fn new(id: Uuid, receiver: Receiver<ForwardedMessage>) -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            id,
            receiver,
        }
    }

    pub async fn message_handler(mut self) {
        let mut game = Game::new();

        tokio::spawn(async move {
            let mut debug_ui = DebugUi::new();

            let winner = loop {
                if let Some(winner) = game.tick() {
                    break winner;
                }

                debug_ui.draw_game(&game);

                tokio::time::sleep(tokio::time::Duration::from_millis(MS_PER_TICK as u64)).await;
            };

            debug_ui.display_winner(winner);
        });

        loop {
            if let Some(forwarded_message) = self.receiver.recv().await {
                match forwarded_message.message {
                    CurverMessageToReceive::JoinRoom { .. } => {
                        let mut clients_lock = self.clients.write();
                        clients_lock.insert(forwarded_message.user_id, forwarded_message.address);
                    }

                    CurverMessageToReceive::LeaveRoom => {
                        let mut clients_lock = self.clients.write();
                        clients_lock.remove(&forwarded_message.user_id);

                        if clients_lock.is_empty() {
                            break;
                        }
                    }

                    CurverMessageToReceive::Rotate { .. } => {}

                    _ => {}
                }
            }
        }
    }
}
