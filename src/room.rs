use std::{collections::HashMap, sync::Arc};

use parking_lot::RwLock;
use tokio::sync::mpsc::Receiver;
use uuid::Uuid;

use crate::{
    constants::{GAME_START_COUNTDOWN_SECONDS, MAP_HEIGHT, MAP_WIDTH, MS_PER_TICK},
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

                        self.spawn_game_if_ready_and_notify_all().await;
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
    async fn spawn_game_if_ready_and_notify_all(&mut self) {
        if !self.check_if_ready_to_start() {
            return;
        }

        self.position_all_players();
        self.game_state = GameState::Countdown;
        self.send_update_to_all();

        tokio::time::sleep(tokio::time::Duration::from_secs(
            GAME_START_COUNTDOWN_SECONDS,
        ))
        .await;

        self.spawn_game();
        self.game_state = GameState::Started;

        self.send_update_to_all();
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
        self.spawn_player(user_id);

        self.send_update_to_all();
    }

    fn rotate_player(&mut self, user_id: Uuid, angle_unit_vector_x: f32, angle_unit_vector_y: f32) {
        if let Some(player) = self.players.write().get_mut(&user_id) {
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

    // --- Player Handling ---
    fn spawn_player(&mut self, player_id: Uuid) {
        let player = Player {
            id: UuidSerde(player_id),
            x: 0.0,
            y: 0.0,
            angle_unit_vector_x: 0.0,
            angle_unit_vector_y: 0.0,
            is_ready: false,
        };

        self.players.write().insert(player_id, player);
    }

    fn position_all_players(&mut self) {
        // Create an imaginary circle on the center of the map
        // The circle's radius will be 40% of the map's width or height, whichever is smaller
        // Place all players on the circle
        // The angle between each player will be 360 / number of players
        // The first player will be placed at 0 degrees and it means they will be placed on the right side of the circle
        // The second player will be placed at 360 / number of players degrees and it means they will be placed on the left side of the circle
        // The players will always face the center of the circle

        let player_count = self.players.read().len() as f32;

        let circle_radius = MAP_WIDTH.min(MAP_HEIGHT) * 0.4;

        let angle_between_players = 360.0 / player_count;

        let mut current_angle: f32 = 0.0;

        let circle_center_x = MAP_WIDTH / 2.0;
        let circle_center_y = MAP_HEIGHT / 2.0;

        for player in self.players.write().values_mut() {
            player.x = circle_center_x + circle_radius * current_angle.to_radians().cos();
            player.y = circle_center_y + circle_radius * current_angle.to_radians().sin();

            player.angle_unit_vector_x = (circle_center_x - player.x) / circle_radius;
            player.angle_unit_vector_y = (circle_center_y - player.y) / circle_radius;

            current_angle += angle_between_players;
        }
    }

    // --- Message Sending ---
    fn send_update_to_all(&self) {
        let update = CurverMessageToSend::Update {
            players: self.players.read().values().cloned().collect(),
            game_state: self.game_state,
        };

        self.send_message_to_all(update);
    }

    fn send_message_to_all(&self, message: CurverMessageToSend) {
        for address in self.clients.read().values() {
            address.do_send(message.clone());
        }
    }

    // --- Client Handling ---
    fn add_client(&mut self, user_id: Uuid, address: CurverAddress) {
        self.clients.write().insert(user_id, address);
    }

    fn toggle_ready_for_user_and_notify_all(&mut self, user_id: Uuid, is_ready: bool) {
        if let Some(player) = self.players.write().get_mut(&user_id) {
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

        drop(players_lock);

        true
    }

    fn remove_client(&mut self, user_id: Uuid) {
        self.clients.write().remove(&user_id);
    }

    fn check_if_clients_empty(&self) -> bool {
        self.clients.read().is_empty()
    }
}
