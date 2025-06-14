use crate::prelude::*;

wrapper!(MonitorId(u8));
wrapper!(SelectedTagset(Tagset));
wrapper!(SelectedClient(Entity));
wrapper!(Tags(Vec<Tag>));

/// A monitor represents a screen, whether physical or emulated. A monitor can hold [`MAX_TAGS`] amount
/// of tags, which can each hold clients.
#[derive(Clone, Copy, Debug, Component)]
pub struct Monitor;
