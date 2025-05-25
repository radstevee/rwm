use rwm::prelude::*;
use tracing_subscriber::fmt::format;

fn main() -> Result<()> {
    let fmt = format()
        .with_file(true)
        .with_line_number(true)
        .with_timer(TimeFormatter);

    tracing_subscriber::fmt().event_format(fmt).init();
    info!("Initialising platform {}", PLATFORM.name());
    
    PLATFORM.init()?;
    
    Ok(())
}
