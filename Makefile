display:
	Xephyr -br -ac -noreset -screen 800x600 :9 &

dev:
	DISPLAY=:9 LOG_LEVEL=trace cargo run
	
dev-hot:
	DISPLAY=:9 LOG_LEVEL=trace dx run --hot-patch --args="--help"

debug:
	cargo build
	export LOG_LEVEL=trace
	DISPLAY=:9 gdb ./target/debug/rwm

run:
	DISPLAY=:9 cargo run --release
