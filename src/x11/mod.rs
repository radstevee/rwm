use std::sync::OnceLock;

use crate::prelude::*;
use init::{become_wm, init_state};
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
pub mod init;

pub struct X11State {
    conn: RustConnection,
    monitors: Vec<Monitor>,
    root_gc: Gcontext,

    wm_protocols: Atom,
    wm_delete_window: Atom,
}

impl X11State {
    pub fn state() -> &'static X11State {
        X11_STATE.get().unwrap_or_else(|| die!("X11 state not initialised yet"))
    }
    

    pub fn create() -> Result<X11State> {
        let (conn, screen_num) = connect(None).context("failed connecting")?;

        let roots = conn.setup().roots.clone();

        let primary_screen = &roots[screen_num];

        become_wm(&conn, &primary_screen)?;

        init_state(conn, roots, screen_num)
    }
}

static X11_STATE: OnceLock<X11State> = OnceLock::new();
