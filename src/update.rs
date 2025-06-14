use crate::prelude::*;

pub fn update_client_geometry(
    config: Res<MainConfig>,
    query: Query<(&Geometry, &ClientWindow, &ClientFrame, &Client), Changed<Geometry>>,
    mut state: ResMut<PlatformState>,
) {
    for (geometry, window, frame, _) in query {
        info!("updating geometry");
        if let Err(e) = CurrentPlatform::update_client_geometry(
            &config, *geometry, **window, **frame, &mut state,
        ) {
            error!(
                "failed updating client geometry: {e:?} - this may result in the client never updating again"
            );
        };
    }
}
