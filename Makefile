dpy:
	Xephyr -br -ac -noreset -screen 800x600 :9 &

kill-dpy:
	killall Xephyr

dev:
	DISPLAY=:9 cargo run

xdev:
	{ \
	  prevdisplay=$$DISPLAY; \
	  Xephyr -br -ac -noreset -screen 800x600 :9 & \
	  pid=$$!; \
	  export DISPLAY=:9; \
	  cargo run; \
	  kill $$pid; \
	  export DISPLAY=$$prevdisplay; \
	}
	
dev-hot:
	DISPLAY=:9 dx serve --hot-patch

debug:
	cargo build
	DISPLAY=:9 gdb ./target/debug/rwm

run:
	DISPLAY=:9 cargo run --release
