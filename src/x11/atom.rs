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
