xephyr:
	Xephyr -br -ac -noreset -screen 800x600 :9 &

dev:
	DISPLAY=:9 LOG_LEVEL=trace cargo run
