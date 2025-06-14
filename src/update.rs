use crate::prelude::*;

pub fn update_client_geometry(
    config: Res<MainConfig>,
    query: Query<(&Geometry, &ClientWindow, &ClientFrame), With<Client>>,
    mut state: ResMut<<CurrentPlatform as Platform>::State>,
) {
    if let Err(e) = CurrentPlatform::update_client_geometry(config, query, &mut state) {
        error!(
            "failed updating client geometry: {e:?} - this may result in the client never updating again"
        );
    };
}
