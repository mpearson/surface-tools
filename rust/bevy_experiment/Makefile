# Define paths as variables
TARGET_DIR := ../../target
WASM_DIR := $(TARGET_DIR)/wasm32-unknown-unknown/release
BINDGEN_DIR := $(WASM_DIR)/wasm-bindgen

# 	wasm-bindgen ../../target/wasm32-unknown-unknown/release/bevy_experiment.wasm --out-dir ../../target/wasm32-unknown-unknown/release/wasm-bindgen

# # Define the run target
# run:
# 	@echo "Running the project..."
# 	python -m http.server --directory ../../target/wasm32-unknown-unknown/release/wasm-bindgen/bevy_expe


# Define the default target
all: run

# Define the build target
build:
	@echo "Building the project..."
	cargo build --release --target wasm32-unknown-unknown
	wasm-bindgen $(WASM_DIR)/bevy_experiment.wasm --out-dir $(BINDGEN_DIR) --target web
	cp index.html $(BINDGEN_DIR)

# Define the run target
run:
	cargo run --target wasm32-unknown-unknown

# run:
# 	@echo "Running the project..."
# 	python -m http.server --directory $(BINDGEN_DIR)

# Define the clean target
clean:
	@echo "Cleaning the project..."
	cargo clean

