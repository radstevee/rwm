use std::{process::exit, sync::OnceLock};

use crate::prelude::*;
use x11rb::{
    connect,
    connection::Connection,
    errors::ReplyError,
    protocol::{
        ErrorKind,
        xproto::{
            Atom, ChangeWindowAttributesAux, ConnectionExt, CreateGCAux, EventMask, Gcontext,
            Screen,
        },
    },
    rust_connection::RustConnection,
};

pub mod atom;
pub mod platform;

pub struct X11State {
    conn: RustConnection,
    monitors: Vec<Monitor>,
    root_gc: Gcontext,

    wm_protocols: Atom,
    wm_delete_window: Atom,
}

fn become_wm(conn: &RustConnection, screen: &Screen) -> Result<()> {
    let change = ChangeWindowAttributesAux::default()
        .event_mask(EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY);
    let res = conn
        .change_window_attributes(screen.root, &change)
        .context("failed changing root window attributes")?
        .check();
    if let Err(ReplyError::X11Error(ref error)) = res {
        if error.error_kind == ErrorKind::Access {
            error!("another WM is already running");
            exit(1)
        } else {
            Ok(())
        }
    } else {
        Ok(res?)
    }
}

impl X11State {
    fn init_monitors(screens: Vec<Screen>) -> Vec<Monitor> {
        let tags = (0..=MAX_TAGS)
            .map(|idx| Tag::new(idx as u8, "tag", Layout::new("", "", test_layout)))
            .collect::<Vec<Tag>>()
            .slice::<MAX_TAGS>();

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

    fn init_conn(
        conn: RustConnection,
        screens: Vec<Screen>,
        primary_screen: usize,
    ) -> Result<Self> {
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
            monitors: X11State::init_monitors(screens),
            root_gc: root_gc,
            wm_protocols,
            wm_delete_window,
        })
    }

    pub fn create() -> Result<X11State> {
        let (conn, screen_num) = connect(None).context("failed connecting")?;

        let roots = conn.setup().roots.clone();

        let primary_screen = &roots[screen_num];

        become_wm(&conn, &primary_screen)?;

        Self::init_conn(conn, roots, screen_num)
    }
}

static X11_STATE: OnceLock<X11State> = OnceLock::new();
