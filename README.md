# rwm

heavily WIP project to rewrite dwm in rust and make it much better!

## goals

- [ ] be fast
- [ ] be responsive
- [ ] have good config and documentation
- [ ] hot reloading configuration
- [ ] use minimal resources
- [ ] support x and wlr
- [ ] have a good CLI

## development

### x11

For x11, you can spawn a Xephyr instance with the
`dpy` make target.

The `dev`, `dev-hot` and `debug` tasks will all
spawn rwm on the Xephyr display.

### hot reloading

For hot reloading, you need the latest git version
of the dioxus CLI installed. You can install it like this:

```bash
cargo install dioxus-cli --locked --git https://github.com/dioxuslabs/dioxus
```

Then, you can run the `dev-hot` make target.

#### hot reload integration

To integrate something with hot reloading, you must wrap
your calls that you want to be hot-reloadable with `dioxus_devtools::subsecond::call`.
