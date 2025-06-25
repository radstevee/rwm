use crate::prelude::*;

#[cfg(feature = "x11")]
pub static RWMP: X11 = X11;

#[cfg(feature = "x11")]
pub type RWMP = X11;

#[cfg(not(feature = "x11"))]
wayland_unimplemented!();

pub type PlatformConnection = <RWMP as RWMPlatform>::Connection;

/// A platform that rwm can run on. Currently, only X is supported.
/// A platform instance should not hold any data.
///
/// This is low-level API and should probably not be used directly.
pub trait RWMPlatform: Plugin + Clone + Copy {
    type Connection: Resource;

    /// Manages a window and returns the populated client.
    fn manage(
        window: Window,
        geometry: Geometry,
        root_window: Window,
        tag: &mut Tag,
        commands: &mut Commands,
        conn: &Self::Connection,
    ) -> Result<(Entity, ClientFrame)>;

    /// Unmanages a window.
    fn unmanage(
        client: Entity,
        window: Window,
        geometry: Geometry,
        frame: Option<Window>,
        root_window: Window,
        tag: &mut Tag,
        commands: &mut Commands,
        conn: &Self::Connection,
    );

    /// Updates the position of the given [`client`] and adds the border.
    fn update_bordered_client_geometry(
        config: &MainConfig,
        geometry: Geometry,
        window: Window,
        frame: Window,
        conn: &Self::Connection,
    );

    /// Deletes the frame window of a client.
    fn delete_frame(
        geometry: Geometry,
        window: Window,
        frame: Window,
        root_window: Window,
        conn: &Self::Connection,
    );

    /// Recreates the frame window of a client.
    fn create_frame(
        config: &MainConfig,
        geometry: Geometry,
        window: Window,
        root_window: Window,
        conn: &Self::Connection,
    ) -> Result<ClientFrame>;

    /// Updates the position of the given [`client`] and adds the border.
    fn update_client_geometry(geometry: Geometry, window: Window, conn: &Self::Connection);

    /// Ungrabs the mouse.
    fn ungrab_mouse(state: &Self::Connection);

    /// Focuses the given [`window`].
    fn focus(window: Window, conn: &Self::Connection);
}
