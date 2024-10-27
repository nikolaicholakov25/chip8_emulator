build:
				cd desktop && \
				cargo build && \
				cd ..

build-release:
				cd chip8_core && \
				cargo build --release && \
				cd .. && \
				cd desktop && \
				cargo build --release && \
				cd ..
