use crate::prelude::*;

use super::events::X11Event;

#[derive(Clone, Copy)]
pub struct X11;

impl Platform for X11 {
    type State = X11State;

    fn init(world: &mut World) {
        let monitors = {
            let mut state = world.resource_mut::<Self::State>();
            println!("{:#?}", state.monitors.clone());
            std::mem::take(&mut state.monitors)
        };

        info!("Found {} monitors", monitors.len());
        for monitor in &monitors {
            world.spawn(monitor.clone());
        }

        let mut state = world.resource_mut::<Self::State>();
        state.monitors = monitors;
    }
}

impl Plugin for X11 {
    fn build(&self, app: &mut App) {
        app.add_event::<X11Event>()
            .add_systems(First, events::poll)
            .add_systems(Update, events::handle_events);
    }

    fn name(&self) -> &str {
        "X11"
    }
}
