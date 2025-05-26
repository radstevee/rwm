#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum NetAtom {
    Supported,
    WMName,
    WMState,
    WMCheck,
    WMFullscreen,
    ActiveWindow,
    WMWindowType,
    WMWindowTypeDialog,
    WMWindowTypeDock,
    ClientList,
    ClientInfo,
    WMWindowOpacity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Atom {
    WMProtocols,
    WMDelete,
    WMState,
    WMTakeFocus,
}

impl Atom {
    pub const fn id(&self) -> &'static str {
        match self {
            Self::WMProtocols => "WM_PROTOCOLS",
            Self::WMDelete => "WM_DELETE_WINDOW",
            _ => todo!(),
        }
    }
}
