build:
				cd chip8_core && \
				cargo build --release && \
				cd .. && \
				cd desktop && \
				cargo build --release && \
				cd ..
