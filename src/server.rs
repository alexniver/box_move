use std::net::{IpAddr, Ipv4Addr};

use bevy::prelude::*;
use bevy::utils::hashbrown::HashMap;
use bevy_quinnet::server::certificate::CertificateRetrievalMode;
use bevy_quinnet::server::{
    ConnectionEvent, ConnectionLostEvent, QuinnetServer, QuinnetServerPlugin,
    ServerEndpointConfiguration,
};
use bevy_quinnet::shared::channels::{ChannelId, ChannelsConfiguration};
use bevy_quinnet::shared::ClientId;

use crate::consts::SERVER_HOST;
use crate::protocol::{ClientMessage, ServerMessage};

const SPEED: f32 = 100.;

pub(crate) fn run() {
    App::new()
        .init_resource::<Players>()
        .add_plugins((MinimalPlugins, QuinnetServerPlugin::default()))
        .add_systems(Startup, start_listening)
        .add_systems(
            Update,
            (
                handle_connection,
                handle_connection_lost.after(handle_connection),
            ),
        )
        .add_systems(FixedUpdate, cube_move)
        .run();
}

#[derive(Debug)]
struct Player {
    direction: crate::protocol::Direction,
}

#[derive(Debug, Resource, Default)]
struct Players {
    map: HashMap<ClientId, Player>,
}

#[derive(Component)]
struct Rect {
    transform: Transform,
}

#[derive(Component)]
struct Client {
    client_id: ClientId,
}

#[derive(Bundle)]
struct RectBundle {
    rect: Rect,
    client: Client,
}

fn start_listening(mut server: ResMut<QuinnetServer>) {
    server
        .start_endpoint(
            ServerEndpointConfiguration::from_ip(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 6000),
            CertificateRetrievalMode::GenerateSelfSigned {
                server_hostname: SERVER_HOST.into(),
            },
            ChannelsConfiguration::default(),
        )
        .unwrap();
}

fn handle_connection(
    mut commands: Commands,
    mut connection_events: EventReader<ConnectionEvent>,
    mut players: ResMut<Players>,
    mut server: ResMut<QuinnetServer>,
) {
    for client in connection_events.read() {
        players.map.insert(
            client.id,
            Player {
                direction: crate::protocol::Direction::None,
            },
        );
        let entity = commands
            .spawn(RectBundle {
                rect: Rect {
                    transform: Transform::default(),
                },
                client: Client {
                    client_id: client.id,
                },
            })
            .id();

        server.endpoint().try_send_group_message_on(
            players.map.keys(),
            ChannelId::default(),
            ServerMessage::InitClient {
                client_id: client.id,
            },
        );

        server.endpoint().try_send_group_message_on(
            players.map.keys(),
            ChannelId::default(),
            ServerMessage::SpawnRect {
                entity,
                pos: Vec3::ZERO,
            },
        );
    }
}

fn handle_connection_lost(
    mut connection_lost_events: EventReader<ConnectionLostEvent>,
    mut players: ResMut<Players>,
) {
    for client in connection_lost_events.read() {
        players.map.remove(&client.id);
    }
}

fn handle_client_message(mut server: ResMut<QuinnetServer>, mut players: ResMut<Players>) {
    let endpoint = server.endpoint_mut();
    for client_id in endpoint.clients() {
        while let Some((_, msg)) = endpoint.try_receive_message_from::<ClientMessage>(client_id) {
            match msg {
                ClientMessage::Direction { direction } => {
                    players.map.get_mut(&client_id).unwrap().direction = direction
                }
            }
        }
    }
}

fn cube_move(
    mut players: ResMut<Players>,
    mut query: Query<(Entity, &mut Transform, &Client)>,
    time: Res<Time>,
    mut server: ResMut<QuinnetServer>,
) {
    for (entity, mut t, c) in &mut query {
        let Some(player) = players.map.get(&c.client_id) else {
            continue;
        };

        let mut value = Vec3::ZERO;
        let speed = SPEED * time.delta_seconds();
        match player.direction {
            crate::protocol::Direction::None => {}
            crate::protocol::Direction::Up => value.y += speed,
            crate::protocol::Direction::Down => value.y -= speed,
            crate::protocol::Direction::Left => value.x -= speed,
            crate::protocol::Direction::Right => value.x += speed,
        }
        t.translation += value;

        server.endpoint().try_send_group_message_on(
            players.map.keys(),
            ChannelId::default(),
            ServerMessage::RectMove {
                entity,
                pos: t.translation,
            },
        );
    }
}
