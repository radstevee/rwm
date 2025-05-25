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
    fn init_conn(conn: RustConnection, screen: &Screen) -> Result<Self> {
        let root_gc = conn.generate_id().context("failed generating root gc id")?;
        let font = conn.generate_id().context("failed generating font id")?;
        conn.open_font(font, b"9x15")
            .context("failed opening font")?;

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
            monitors: vec![], /* TODO */
            root_gc: root_gc,
            wm_protocols,
            wm_delete_window,
        })
    }

    pub fn create() -> Result<X11State> {
        let (conn, screen_num) = connect(None).context("failed connecting")?;

        let screen = &conn.setup().roots[screen_num].clone();

        become_wm(&conn, &screen)?;
        
        Self::init_conn(conn, &screen)
    }
}

static X11_STATE: OnceLock<X11State> = OnceLock::new();
