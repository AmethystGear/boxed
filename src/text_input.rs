use bevy::prelude::*;
use bevy_simple_text_input::{TextInputBundle, TextInputInactive, TextInputSubmitEvent};
use itertools::Itertools;

use crate::{
    rendering::InGameCamera,
    server::{Connections, Server, ServerState},
};

const BORDER_COLOR_ACTIVE: Color = Color::srgb(0.75, 0.52, 0.99);
const BORDER_COLOR_INACTIVE: Color = Color::srgb(0.25, 0.25, 0.25);
const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
const BACKGROUND_COLOR: Color = Color::srgb(0.15, 0.15, 0.15);

// i'm using a text box as input for doing all the actions,
// which isn't ideal, we should be using some kind of UI
pub fn setup_textbox(mut commands: Commands) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            },
            // Make this container node bundle to be Interactive so that clicking on it removes
            // focus from the text input.
            Interaction::None,
        ))
        .with_children(|parent| {
            parent.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Px(400.0),
                        border: UiRect::all(Val::Px(2.0)),
                        padding: UiRect::all(Val::Px(2.0)),
                        position_type: PositionType::Absolute,
                        left: Val::Px(20.0),
                        bottom: Val::Px(20.0),
                        ..default()
                    },
                    border_color: BORDER_COLOR_INACTIVE.into(),
                    background_color: BACKGROUND_COLOR.into(),
                    ..default()
                },
                TextInputBundle::default()
                    .with_text_style(TextStyle {
                        font_size: 16.,
                        color: TEXT_COLOR,
                        ..default()
                    })
                    .with_placeholder("input command here", None)
                    .with_inactive(true),
            ));
        });
}

pub fn focus(
    query: Query<(Entity, &Interaction), Changed<Interaction>>,
    mut text_input_query: Query<(Entity, &mut TextInputInactive, &mut BorderColor)>,
    mut camera: Query<&mut Transform, With<InGameCamera>>,
    input: Res<ButtonInput<KeyCode>>,
) {
    for (interaction_entity, interaction) in &query {
        if *interaction == Interaction::Pressed {
            let (entity, mut inactive, mut border_color) = text_input_query.single_mut();
            if entity == interaction_entity {
                inactive.0 = false;
                *border_color = BORDER_COLOR_ACTIVE.into();
            } else {
                inactive.0 = true;
                *border_color = BORDER_COLOR_INACTIVE.into();
            }
        }
    }

    // probably, we won't be moving the camera around manually
    // it should probably auto zoom to wherever the virus currently is.
    // Though we probably need to be able to zoom in and out manually.
    let (_, inactive, _) = text_input_query.single();
    if inactive.0 {
        let mut intent = Vec2::ZERO;
        if input.pressed(KeyCode::KeyW) || input.pressed(KeyCode::ArrowUp) {
            intent.y += 1.0;
        }
        if input.pressed(KeyCode::KeyS) || input.pressed(KeyCode::ArrowDown) {
            intent.y -= 1.0;
        }
        if input.pressed(KeyCode::KeyA) || input.pressed(KeyCode::ArrowLeft) {
            intent.x -= 1.0;
        }
        if input.pressed(KeyCode::KeyD) || input.pressed(KeyCode::ArrowRight) {
            intent.x += 1.0;
        }
        camera.single_mut().translation += Vec3::new(intent.x, intent.y, 0.0) * 2.;
    }
}

fn validate_move(
    connections: &mut Connections,
    servers: &mut Query<(Entity, &mut Server)>,
    server_name: &str,
) -> bool {
    let (infected_entity, _) = servers
        .iter()
        .filter(|(_, server)| server.state == ServerState::Infected)
        .next()
        .expect("one server should be infected");

    for (_, server) in &connections.graph[&infected_entity] {
        if let Some((_, server)) = servers
            .iter_mut()
            .filter(|(entity, _)| server == entity)
            .next()
        {
            if server.name == server_name && server.state == ServerState::Corrupted {
                return true;
            }
        }
    }
    return false;
}

// parse command and do action based on it
pub fn command(
    mut events: EventReader<TextInputSubmitEvent>,
    mut connections: ResMut<Connections>,
    mut servers: Query<(Entity, &mut Server)>,
) {
    for event in events.read() {
        match event.value.split_whitespace().collect_vec()[..] {
            ["move", server_name] => {
                if validate_move(&mut connections, &mut servers, server_name) {
                    for (_, mut server) in &mut servers {
                        if server.state == ServerState::Infected {
                            server.state = ServerState::Corrupted;
                        }
                    }
                    for (_, mut server) in &mut servers {
                        if server.name == server_name {
                            server.state = ServerState::Infected;
                        }
                    }
                }
            }
            ["msg", server, action] => match action {
                "reboot" => {}
                "cycle" => {}
                "heat" => {}
                _ => println!("bad command"),
            },
            ["msg", corrupted_server, "hack", healthy_server] => {}
            ["upd", action] => match action {
                "cycle" => {}
                "heat" => {}
                _ => println!("bad command"),
            },
            ["upd", "hack", server] => {}
            _ => println!("bad command"),
        }
    }
}
