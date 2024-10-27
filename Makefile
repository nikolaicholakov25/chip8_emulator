build:
		cargo build --manifest-path ./desktop/Cargo.toml

build-release:
		cargo build --manifest-path ./desktop/Cargo.toml && \
		cargo build --manifest-path ./chip8_core/Cargo.toml

run-game:
		cargo run --manifest-path ./desktop/Cargo.toml ./games/${game} $(speed)
