# rwm

heavily WIP project to make my own window manager :)

## goals

- [ ] be fast
- [ ] be responsive
- [ ] have good config and documentation
- [ ] hot reloading configuration
- [ ] use minimal resources
- [ ] support x and wayland
- [ ] have a good CLI
- [ ] integrated whiteboard support
- [ ] integrated clipboard management

## development

### x11

For x11, you can spawn a Xephyr instance with the `dpy` make target.

The `dev`, `dev-hot` and `debug` tasks will all spawn rwm on the Xephyr display.

The `xdev` make target will spawn a temporary Xephyr instance and run rwm inside of it.

### hot reloading

For hot reloading, you need the latest git version
of the dioxus CLI installed. You can install it like this:

```bash
cargo install dioxus-cli --locked --git https://github.com/dioxuslabs/dioxus
```

Then, you can run the `dev-hot` make target.

#### hot reload integration

To integrate something with hot reloading that is not a system, you must wrap your calls that you want to be hot-reloadable with `bevy::app::hotpatch::call`.

### hacking guidelines

- **Add proper error handling**. X11 API calls are exempt from this, they can unwrapped, they're infallible unless the connection died
- Don't double-mut-deref mutable components/resources when you're mutating them, instead, create new ones and only do one mut-deref to change the actual value
