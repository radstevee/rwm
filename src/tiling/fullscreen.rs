use crate::prelude::*;

#[derive(Component, Clone, Copy, Debug)]
pub struct Fullscreened;

#[derive(Component, Clone, Copy, Debug)]
pub struct TransitioningFullscreenStates;

pub fn handle_fullscreen_add(
    mut query: Query<
        (
            Entity,
            &mut Geometry,
            &mut OriginalGeometry,
            &ClientWindow,
            &ClientFrame,
        ),
        Added<Fullscreened>,
    >,
    mut state: ResMut<PlatformState>,
    mut commands: Commands,
    monitors: Query<&Geometry, Without<ClientWindow>>,
    root_window: Res<MainRootWindow>,
) {
    for (client, mut geometry, mut original_geometry, window, frame) in &mut query {
        for monitor in &monitors {
            *original_geometry = OriginalGeometry(*geometry);
            *geometry = Geometry::new(0, 0, monitor.width(), monitor.height());

            trace!("adding fullscreen - deleting frame {}", **frame);
            CurrentPlatform::delete_frame(*geometry, **window, **frame, **root_window, &mut state);
            CurrentPlatform::update_client_geometry(*geometry, **window, &mut state);
            CurrentPlatform::focus(**window, &mut state);

            commands
                .entity(client)
                .insert(TransitioningFullscreenStates);
        }
    }
}

pub fn add_fullscreen_remove_handler(world: &mut World) {
    world.add_observer(handle_fullscreen_remove);
}

fn handle_fullscreen_remove(
    removal: On<Remove, Fullscreened>,
    mut query: Query<(Entity, &mut Geometry, &OriginalGeometry, &ClientWindow)>,
    mut state: ResMut<PlatformState>,
    mut dragging: ResMut<Dragging>,
    mut monitors: Query<&mut Tags, With<Monitor>>,
    mut commands: Commands,
    root_window: Res<MainRootWindow>,
    config: Res<MainConfig>,
) {
    for (client, mut geometry, original_geometry, window) in &mut query {
        if client != removal.target().unwrap() {
            continue;
        }

        *geometry = **original_geometry;
        *dragging = Dragging(None);
        CurrentPlatform::ungrab_mouse(&mut state);

        commands.entity(client).despawn();

        for mut tags in &mut monitors {
            let tag = tags.get_mut(0).unwrap(); // TODO: tagging
            let (client, frame) = match CurrentPlatform::manage(
                **window,
                *geometry,
                **root_window,
                tag,
                &mut commands,
                &mut state,
            ) {
                Err(e) => {
                    error!("failed remanaging: {e}");
                    continue;
                }
                Ok(v) => v,
            };

            CurrentPlatform::update_bordered_client_geometry(
                &config, *geometry, **window, *frame, &mut state,
            );
 
            commands.entity(client).insert(TransitioningFullscreenStates);
        }

        CurrentPlatform::focus(**window, &mut state);
    }
}

pub fn handle_fullscreen(
    mut events: EventReader<KeybindTriggered>,
    mut commands: Commands,
    mut query: Query<(Entity, Has<Fullscreened>), With<Client>>,
) {
    for event in events.read() {
        dbg!(event.clone());
        if event.action() != &KeybindAction::ToggleFullscreen {
            continue;
        }

        for (client, fullscreened) in &mut query {
            if !event.client().is_some_and(|c| c == client) {
                continue;
            }

            if fullscreened {
                commands.entity(client).remove::<Fullscreened>();
            } else {
                commands.entity(client).insert(Fullscreened);
            }
        }
    }
}
