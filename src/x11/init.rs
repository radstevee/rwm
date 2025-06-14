use crate::prelude::*;
use x11rb::{
    connection::Connection, errors::ReplyError, protocol::{
        xproto::{
            ChangeWindowAttributesAux, ConnectionExt, CreateGCAux, EventMask, MapState, Screen,
        }, ErrorKind
    }
};

wrapper!(AvailableScreens(Vec<Screen>));
wrapper!(ScreenNumber(usize));

pub fn connect(mut commands: Commands) {
    let (conn, screen_num) = catching!("failed connecting to x11", x11rb::connect(None));
    commands.insert_resource(X11Connection(conn));
    commands.insert_resource(ScreenNumber(screen_num as usize));
}

pub fn become_wm(conn: Res<X11Connection>, screens: Res<AvailableScreens>, screen_num: Res<ScreenNumber>) {
    let screen = screens[**screen_num];
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

pub fn load_monitors(screens: Res<AvailableScreens>, config: Res<MainConfig>, mut commands: Commands) {
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

        commands.spawn((Monitor, MonitorId(idx as u8), dimensions, SelectedTagset(Tagset::default()), Tagset::default(), Tags(tags)));

        prev_screen = Some(screen);
    }
}

pub fn init(conn: Res<X11Connection>, screens: Res<AvailableScreens>, mut commands: Commands) -> Result<()> {
    let root_gc = conn.generate_id().context("failed generating root gc id")?;

    let screen = screens[0].clone();

    let gc_aux = CreateGCAux::new()
        .graphics_exposures(0)
        .background(screen.black_pixel)
        .foreground(screen.white_pixel);

    conn.create_gc(root_gc, screen.root, &gc_aux)
        .context("failed creating root gc")?;

    let wm_protocols = conn
        .intern_atom(false, b"WM_PROTOCOLS")
        .context("failed fetching WM_PROTOCOLS")?
        .reply()
        .context("failed receiving WM_PROTOCOLS reply")?
        .atom;
    commands.insert_resource(NetWMProtocols(wm_protocols));

    let wm_delete_window = conn
        .intern_atom(false, b"WM_DELETE_WINDOW")
        .context("failed fetching WM_DELETE_WINDOW")?
        .reply()
        .context("failed receiving WM_DELETE_WINDOW reply")?
        .atom;
    commands.insert_resource(NetWMDeleteWindow(wm_delete_window));

    let mut state = X11State {
        conn: **conn,
        dragging: None,
    };
    
    commands.insert_resource(state);

    Ok(())
}

pub fn scan_existing_windows(mut state: ResMut<X11State>, screens: Res<AvailableScreens>, mut commands: Commands) {
    for screen in screens {
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

        for (win, attr, geom) in windows {
            if !attr.override_redirect && attr.map_state != MapState::UNMAPPED {
                let geom = Geometry::new(
                    geom.x as i32,
                    geom.y as i32,
                    geom.width as u32,
                    geom.height as u32,
                );
                state.manage(win, geom, &mut commands)?;
            }
        }
    }
}
