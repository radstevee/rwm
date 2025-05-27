use x11rb::{connection::Connection, errors::ReplyError, protocol::{xproto::{ChangeWindowAttributesAux, ConnectionExt, CreateGCAux, EventMask, Screen}, ErrorKind}, rust_connection::RustConnection};
use crate::prelude::*;

pub fn become_wm(conn: &RustConnection, screen: &Screen) -> Result<()> {
    let change = ChangeWindowAttributesAux::default()
        .event_mask(EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY);
    let res = conn
        .change_window_attributes(screen.root, &change)
        .context("failed changing root window attributes")?
        .check();
    if let Err(ReplyError::X11Error(ref error)) = res {
        if error.error_kind == ErrorKind::Access {
            die!("another WM is already running");
        } else {
            Ok(())
        }
    } else {
        Ok(res?)
    }
}

pub fn load_monitors(screens: Vec<Screen>) -> Vec<Monitor> {
    let tags_cfg = config().tags().clone();
    let tags = tags_cfg
        .enabled_tags()
        .into_iter()
        .map(|tag| {
            let label = tags_cfg.label(*tag).unwrap();

            Tag::new(*tag, label, Layout::new("", "", test_layout))
        })
        .collect::<Vec<Tag>>();

    let mut prev_screen = None;
    let mut monitors = vec![];

    for (idx, screen) in screens.iter().enumerate() {
        let x = prev_screen.map_or(0, |s: &Screen| s.width_in_pixels);
        let y = prev_screen.map_or(0, |s: &Screen| s.height_in_pixels);

        let dimensions = Geometry::new(
            x as u32,
            y as u32,
            screen.width_in_pixels as u32,
            screen.height_in_pixels as u32,
        );

        monitors.push(Monitor::new(idx as u8, tags.clone(), dimensions));

        prev_screen = Some(&screen);
    }

    monitors
}

pub fn init_state(
    conn: RustConnection,
    screens: Vec<Screen>,
    primary_screen: usize,
) -> Result<X11State> {
    let root_gc = conn.generate_id().context("failed generating root gc id")?;
    let font = conn.generate_id().context("failed generating font id")?;
    conn.open_font(font, b"9x15")
        .context("failed opening font")?;

    let screen = screens[primary_screen].clone();

    let gc_aux = CreateGCAux::new()
        .graphics_exposures(0)
        .background(screen.black_pixel)
        .foreground(screen.white_pixel)
        .font(font);

    conn.create_gc(root_gc, screen.root, &gc_aux)
        .context("failed creating root gc")?;
    conn.close_font(font).context("failed closing font")?;

    let wm_protocols = conn
        .intern_atom(false, b"WM_PROTOCOLS")
        .context("failed fetching WM_PROTOCOLS")?
        .reply()
        .context("failed receiving WM_PROTOCOLS reply")?
        .atom;
    let wm_delete_window = conn
        .intern_atom(false, b"WM_DELETE_WINDOW")
        .context("failed fetching WM_DELETE_WINDOW")?
        .reply()
        .context("failed receiving WM_DELETE_WINDOW reply")?
        .atom;

    Ok(X11State {
        conn,
        monitors: load_monitors(screens),
        root_gc,
        wm_protocols,
        wm_delete_window,
    })
}
