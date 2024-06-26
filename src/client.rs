use std::net::IpAddr;

use bevy::{math::vec3, prelude::*, sprite::MaterialMesh2dBundle, utils::HashMap};
use bevy_quinnet::{
    client::{
        certificate::CertificateVerificationMode, connection::ClientEndpointConfiguration,
        QuinnetClient, QuinnetClientPlugin,
    },
    shared::{
        channels::{ChannelType, ChannelsConfiguration},
        ClientId,
    },
};

use crate::{
    consts::{LOCAL_BIND_IP, SERVER_HOST, SERVER_PORT},
    protocol::{ClientMessage, ServerMessage},
};

pub(crate) fn run(host: IpAddr) {
    App::new()
        .init_resource::<EntityMap>()
        .insert_resource(ConnectionInfo {
            client_id: None,
            host,
        })
        .add_plugins(DefaultPlugins)
        .add_plugins(QuinnetClientPlugin::default())
        .add_systems(Startup, (setup, start_connection))
        .add_systems(Update, handle_server_message)
        .add_systems(Update, handle_move)
        // .add_systems(FixedUpdate, box_move)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        transform: Transform::default().with_translation(vec3(20., 20., 0.)),
        ..default()
    });
}

fn start_connection(mut client: ResMut<QuinnetClient>, conn_info: Res<ConnectionInfo>) {
    client
        .open_connection(
            ClientEndpointConfiguration::from_ips(conn_info.host, SERVER_PORT, LOCAL_BIND_IP, 0),
            CertificateVerificationMode::SkipVerification,
            ChannelsConfiguration::from_types(vec![ChannelType::OrderedReliable]).unwrap(),
        )
        .unwrap();
}

fn handle_server_message(
    mut commands: Commands,
    mut meshs: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut client: ResMut<QuinnetClient>,
    mut connection_info: ResMut<ConnectionInfo>,
    mut entity_map: ResMut<EntityMap>,
    mut query_rect: Query<&mut Transform>,
) {
    while let Some((_, message)) = client
        .connection_mut()
        .try_receive_message::<ServerMessage>()
    {
        match message {
            ServerMessage::InitClient { client_id } => {
                connection_info.client_id = Some(client_id);
            }
            ServerMessage::SpawnRect { entity, pos } => {
                let client_entity = commands
                    .spawn((MaterialMesh2dBundle {
                        mesh: bevy::sprite::Mesh2dHandle(meshs.add(Rectangle::new(50., 50.))),
                        material: materials.add(Color::BLUE),
                        transform: Transform::default().with_translation(pos),
                        ..default()
                    },))
                    .id();
                entity_map.map.insert(entity, client_entity);
            }
            ServerMessage::RectMove { entity, pos } => {
                let entity = *entity_map.map.get(&entity).unwrap();
                let mut t = query_rect.get_mut(entity).unwrap();
                t.translation = pos;
            }
            ServerMessage::DespawnRect { entity } => {
                let entity = *entity_map.map.get(&entity).unwrap();
                commands.entity(entity).despawn();
            }
        }
    }
}

fn handle_move(
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    conn_info: Res<ConnectionInfo>,
    client: Res<QuinnetClient>,
) {
    let mut direction = crate::protocol::Direction::None;
    if input.pressed(KeyCode::KeyW) {
        direction = crate::protocol::Direction::Up;
    }

    if input.pressed(KeyCode::KeyS) {
        direction = crate::protocol::Direction::Down;
    }

    if input.pressed(KeyCode::KeyA) {
        direction = crate::protocol::Direction::Left;
    }

    if input.pressed(KeyCode::KeyD) {
        direction = crate::protocol::Direction::Right;
    }
    client
        .connection()
        .try_send_message(ClientMessage::Direction { direction })
}

#[derive(Debug, Resource)]
struct ConnectionInfo {
    host: IpAddr,
    client_id: Option<ClientId>,
}

// map server-entity <-> client-entity
#[derive(Resource, Default)]
struct EntityMap {
    map: HashMap<Entity, Entity>,
}
