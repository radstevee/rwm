use std::{collections::BinaryHeap, sync::Arc};

use crate::prelude::*;
use x11rb::{
    connection::Connection,
    errors::ReplyError,
    protocol::{
        ErrorKind,
        xproto::{
            ButtonIndex, ChangeWindowAttributesAux, ConnectionExt, CreateGCAux, EventMask,
            GrabMode, MapState, Screen,
        },
    },
};

wrapper!(AvailableScreens(Vec<Screen>));
wrapper!(ScreenNumber(usize));

pub fn connect(mut commands: Commands) {
    let (conn, screen_num) = catching!("failed connecting to x11", x11rb::connect(None));
    let screens = conn.setup().roots.clone();
    commands.insert_resource(AvailableScreens(screens.clone()));
    commands.insert_resource(MainRootWindow(screens[screen_num].root));
    commands.insert_resource(X11Connection(Arc::new(conn)));
    commands.insert_resource(ScreenNumber(screen_num));
    commands.insert_resource(Dragging(None));
}

pub fn become_wm(
    conn: Res<X11Connection>,
    screens: Res<AvailableScreens>,
    screen_num: Res<ScreenNumber>,
) {
    let screen = screens[**screen_num].clone();
    let change = ChangeWindowAttributesAux::default()
        .event_mask(EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY);
    let res = conn
        .change_window_attributes(screen.root, &change)
        .unwrap()
        .check();
    if let Err(ReplyError::X11Error(ref error)) = res {
        if error.error_kind == ErrorKind::Access {
            die!("another WM is already running");
        }
    }
}

pub fn load_monitors(
    screens: Res<AvailableScreens>,
    config: Res<MainConfig>,
    mut commands: Commands,
) {
    let tags_cfg = config.tags().clone();
    let tags = tags_cfg
        .enabled_tags()
        .iter()
        .map(|tag| {
            let label = tags_cfg.label(*tag).unwrap();

            Tag::new(*tag, label, Layout::new("", "", test_layout))
        })
        .collect::<Vec<Tag>>();

    let mut prev_screen = None;

    for (idx, screen) in screens.iter().enumerate() {
        let x = prev_screen.map_or(0, |s: &Screen| s.width_in_pixels);
        let y = prev_screen.map_or(0, |s: &Screen| s.height_in_pixels);

        let dimensions = Geometry::new(
            x as i32,
            y as i32,
            screen.width_in_pixels as u32,
            screen.height_in_pixels as u32,
        );

        commands.spawn((
            Monitor,
            MonitorId(idx as u8),
            dimensions,
            SelectedTagset(Tagset::default()),
            Tagset::default(),
            Tags(tags.clone()),
        ));

        prev_screen = Some(screen);
    }
}

pub fn init(
    conn: ResMut<X11Connection>,
    mut commands: Commands,
    screens: Res<AvailableScreens>,
) -> Result<()> {
    let root_gc = conn.generate_id().unwrap();

    let screen = screens[0].clone();

    let gc_aux = CreateGCAux::new()
        .graphics_exposures(0)
        .background(screen.black_pixel)
        .foreground(screen.white_pixel);

    conn.create_gc(root_gc, screen.root, &gc_aux).unwrap();

    let wm_protocols = conn
        .intern_atom(false, b"WM_PROTOCOLS")
        .unwrap()
        .reply()
        .context("failed receiving WM_PROTOCOLS reply")?
        .atom;
    commands.insert_resource(NetWMProtocols(wm_protocols));

    let wm_delete_window = conn
        .intern_atom(false, b"WM_DELETE_WINDOW")
        .unwrap()
        .reply()
        .context("failed receiving WM_DELETE_WINDOW reply")?
        .atom;
    commands.insert_resource(NetWMDeleteWindow(wm_delete_window));

    let state = X11State {
        conn: X11Connection(conn.clone()),
        ignored_sequences: BinaryHeap::default()
    };

    commands.insert_resource(state);

    Ok(())
}

pub fn scan_existing_windows(
    mut state: ResMut<X11State>,
    mut monitors: Query<&mut Tags, With<Monitor>>,
    mut commands: Commands,
    screens: Res<AvailableScreens>,
) -> Result<()> {
    for screen in screens.clone() {
        let tree = state
            .conn
            .query_tree(screen.root)
            .context("failed querying tree")?
            .reply()
            .context("failed receiving tree reply")?;

        let mut windows = Vec::with_capacity(tree.children.len());

        for win in tree.children {
            let attr = match state.conn.get_window_attributes(win)?.reply() {
                Ok(attr) => attr,
                Err(_) => continue,
            };
            let geom = match state.conn.get_geometry(win)?.reply() {
                Ok(geom) => geom,
                Err(_) => continue,
            };

            windows.push((win, attr, geom));
        }

        for (win, attr, geometry) in windows {
            if !attr.override_redirect && attr.map_state != MapState::UNMAPPED {
                let geometry = Geometry::new(
                    geometry.x as i32,
                    geometry.y as i32,
                    geometry.width as u32,
                    geometry.height as u32,
                );

                for mut tags in &mut monitors {
                    let mut tag = (tags).get_mut(0).unwrap();
                    state.manage(win, geometry, screen.root, &mut tag, &mut commands)?;
                }
            }
        }
    }

    Ok(())
}

pub fn grab(state: Res<X11State>, root_window: Res<MainRootWindow>) {
    state
        .conn
        .grab_button(
            false,
            **root_window,
            EventMask::BUTTON_PRESS | EventMask::BUTTON_RELEASE | EventMask::BUTTON1_MOTION,
            GrabMode::ASYNC,
            GrabMode::ASYNC,
            **root_window,
            0u32,
            ButtonIndex::ANY,
            mod_mask(),
        )
        .unwrap();

    state
        .conn
        .grab_key(
            false,
            **root_window,
            mod_mask(),
            0, /* any key */
            GrabMode::ASYNC,
            GrabMode::ASYNC,
        )
        .unwrap();
}
