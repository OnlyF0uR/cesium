# Makefile

# Compiler and flags
EMCC = emcc
C_FILES = contract.c convert.c
OUTPUT = ../../target/nomisma-c.wasm
OPTIMIZATION = -O3
EXPORTED_FUNCTIONS = -s "EXPORTED_FUNCTIONS=['_initialize', '_create']"
WASM_FLAG = -s WASM=1
NO_ENTRY_FLAG = --no-entry

.PHONY: all clean

# Default target
all: $(OUTPUT)

# Target to build the .wasm file
$(OUTPUT): $(C_FILES)
	$(EMCC) $(C_FILES) -o $(OUTPUT) $(OPTIMIZATION) $(EXPORTED_FUNCTIONS) $(WASM_FLAG) $(NO_ENTRY_FLAG)

# Clean target
clean:
	rm -f $(OUTPUT)
