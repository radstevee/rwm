dpy:
	Xephyr -br -ac -noreset -screen 800x600 :9 &

dev:
	DISPLAY=:9 cargo run
	
dev-hot:
	DISPLAY=:9 dx serve --hot-patch

debug:
	cargo build
	DISPLAY=:9 gdb ./target/debug/rwm

run:
	DISPLAY=:9 cargo run --release
