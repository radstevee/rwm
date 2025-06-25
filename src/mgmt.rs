use crate::prelude::*;

#[derive(Component, Clone, Copy, Debug)]
pub struct Unmanaged;

pub fn handle_unmanage(
    mut monitors: Query<&mut Tags, With<Monitor>>,
    mut commands: Commands,
    conn: Res<PlatformConnection>,
    clients: Query<(Entity, &ClientWindow, &Geometry, Option<&ClientFrame>), With<Unmanaged>>,
    root_window: Res<MainRootWindow>,
) {
    for (client, window, geometry, frame) in clients {
        for mut tags in &mut monitors {
            let tag = tags.get_mut(0).unwrap(); // TODO: tagging

            RWMP::unmanage(
                client,
                **window,
                *geometry,
                frame.map(|f| **f),
                **root_window,
                tag,
                &mut commands,
                &conn,
            );
        }
    }
}
