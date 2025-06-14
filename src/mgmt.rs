use crate::prelude::*;

#[derive(Component, Clone, Copy, Debug)]
pub struct Disowned;

pub fn handle_unmanage(
    mut monitors: Query<&mut Tags, With<Monitor>>,
    mut state: ResMut<PlatformState>,
    clients: Query<(Entity, &ClientWindow, &Geometry, &ClientFrame), With<Disowned>>,
    root_window: Res<MainRootWindow>,
) {
    for (client, window, geometry, frame) in clients {
        for mut tags in &mut monitors {
            let tag = tags.get_mut(0).unwrap(); // TODO: tagging

            if let Err(e) = CurrentPlatform::unmanage(
                client,
                **window,
                *geometry,
                **frame,
                **root_window,
                tag,
                &mut state,
            ) {
                error!(
                    "failed unmanaging window {}: {e:?} - this may lead to undefined behaviour",
                    **window
                );
            }
        }
    }
}
