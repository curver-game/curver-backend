use std::{collections::HashMap, sync::Arc};

use parking_lot::RwLock;
use tokio::sync::mpsc::Receiver;
use uuid::Uuid;

use crate::{
    constants::{MAP_HEIGHT, MAP_WIDTH, MS_PER_TICK},
    curver_ws_actor::CurverAddress,
    debug_ui::DebugUi,
    game::{player::Player, Clients, Game, GameState, Players},
    message::{CurverMessageToReceive, CurverMessageToSend, ForwardedMessage, UuidSerde},
};

pub struct Room {
    receiver: Receiver<ForwardedMessage>,

    clients: Arc<RwLock<Clients>>,
    players: Arc<RwLock<Players>>,

    game_state: GameState,
}

impl Room {
    pub fn new(receiver: Receiver<ForwardedMessage>) -> Self {
        let clients = Arc::new(RwLock::new(HashMap::new()));
        let players = Arc::new(RwLock::new(HashMap::new()));

        Self {
            receiver,
            clients: clients.clone(),
            players: players.clone(),
            game_state: GameState::Waiting,
        }
    }

    pub async fn message_handler(mut self) {
        loop {
            if let Some(forwarded_message) = self.receiver.recv().await {
                match forwarded_message.message {
                    CurverMessageToReceive::JoinRoom { .. } => {
                        self.join_room_and_notify_all(
                            forwarded_message.user_id,
                            forwarded_message.address,
                        );
                    }

                    CurverMessageToReceive::LeaveRoom => {
                        self.leave_room_and_notify_all(forwarded_message.user_id);

                        if self.check_if_clients_empty() {
                            break;
                        }
                    }

                    CurverMessageToReceive::IsReady { is_ready } => {
                        self.toggle_ready_for_user_and_notify_all(
                            forwarded_message.user_id,
                            is_ready,
                        );

                        self.spawn_game_if_ready_and_notify_all();
                    }

                    CurverMessageToReceive::Rotate {
                        angle_unit_vector_x,
                        angle_unit_vector_y,
                    } => {
                        self.rotate_player(
                            forwarded_message.user_id,
                            angle_unit_vector_x,
                            angle_unit_vector_y,
                        );
                    }

                    CurverMessageToReceive::CreateRoom => {
                        panic!("CreateRoom message should not be sent to a room");
                    }
                }
            }
        }
    }

    // --- Game Logic ---
    fn spawn_game_if_ready_and_notify_all(&mut self) {
        if !self.check_if_ready_to_start() {
            return;
        }

        self.spawn_game();
        self.game_state = GameState::Started;

        self.send_message_to_all(CurverMessageToSend::GameState {
            current_state: GameState::Started,
        })
    }

    fn spawn_game(&self) {
        let mut game = Game::new(self.clients.clone(), self.players.clone());

        tokio::spawn(async move {
            let mut debug_ui = DebugUi::new();

            let winner = loop {
                if let Some(winner) = game.tick() {
                    break winner;
                }

                debug_ui.draw_game(&game);

                tokio::time::sleep(tokio::time::Duration::from_millis(MS_PER_TICK as u64)).await;
            };

            debug_ui.display_winner(&winner);
        });
    }

    // --- Message Handling ---
    fn join_room_and_notify_all(&mut self, user_id: Uuid, address: CurverAddress) {
        self.add_client(user_id, address);

        let random_angle = rand::random::<f32>() * 2.0 * std::f32::consts::PI;
        // TODO: Fix this
        let player = Player {
            id: UuidSerde(user_id),
            x: rand::random::<f32>() * MAP_WIDTH,
            y: rand::random::<f32>() * MAP_HEIGHT,
            angle_unit_vector_x: random_angle.cos(),
            angle_unit_vector_y: random_angle.sin(),
            is_ready: false,
        };

        self.players.write().insert(user_id, player);

        self.send_update_to_all();
    }

    fn rotate_player(&mut self, user_id: Uuid, angle_unit_vector_x: f32, angle_unit_vector_y: f32) {
        let mut players_lock = self.players.write();

        if let Some(player) = players_lock.get_mut(&user_id) {
            player.angle_unit_vector_x = angle_unit_vector_x;
            player.angle_unit_vector_y = angle_unit_vector_y;
        }
    }

    fn leave_room_and_notify_all(&mut self, user_id: Uuid) {
        self.remove_client(user_id);

        self.send_message_to_all(CurverMessageToSend::UserEliminated {
            user_id: UuidSerde(user_id),
        })
    }

    // --- Message Sending ---
    fn send_update_to_all(&self) {
        let update = CurverMessageToSend::Update {
            players: self.players.read().values().cloned().collect(),
        };

        self.send_message_to_all(update);
    }

    fn send_message_to_all(&self, message: CurverMessageToSend) {
        for address in self.clients.read().values() {
            address.do_send(message.clone());
        }
    }

    fn send_message_to_user(&self, user_id: Uuid, message: CurverMessageToSend) {
        if let Some(address) = self.clients.read().get(&user_id) {
            address.do_send(message);
        }
    }

    // --- Client Handling ---
    fn add_client(&mut self, user_id: Uuid, address: CurverAddress) {
        let mut clients_lock = self.clients.write();
        clients_lock.insert(user_id, address);
    }

    fn toggle_ready_for_user_and_notify_all(&mut self, user_id: Uuid, is_ready: bool) {
        let mut players_lock = self.players.write();

        if let Some(player) = players_lock.get_mut(&user_id) {
            player.is_ready = is_ready;
        }

        self.send_update_to_all();
    }

    fn check_if_ready_to_start(&self) -> bool {
        let players_lock = self.players.read();

        if players_lock.len() < 2 {
            return false;
        }

        for player in players_lock.values() {
            if !player.is_ready {
                return false;
            }
        }

        true
    }

    fn remove_client(&mut self, user_id: Uuid) {
        let mut clients_lock = self.clients.write();
        clients_lock.remove(&user_id);
    }

    fn check_if_clients_empty(&self) -> bool {
        let clients_lock = self.clients.read();
        clients_lock.is_empty()
    }
}
