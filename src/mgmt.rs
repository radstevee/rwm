use crate::prelude::*;

#[derive(Component, Clone, Copy, Debug)]
pub struct Disowned;

pub(super) fn handle_unmanage(
    mut commands: Commands,
    query: Query<(Entity, )>
) {
    
}
