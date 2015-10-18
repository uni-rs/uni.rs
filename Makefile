ARCH = i686
CROSS_COMPILE = i686-elf-

AR = $(CROSS_COMPILE)ar
RUSTC ?= rustc
AS = $(CROSS_COMPILE)gcc
LD = $(CROSS_COMPILE)gcc

LDFLAGS = -nostdlib -T $(SRC)/arch/$(ARCH)/linker.ld

ROOT_DIR = $(PWD)
SRC = $(ROOT_DIR)/src

BIN_PATH ?= $(ROOT_DIR)/examples/$(BIN_NAME)
BIN_OBJ = $(BIN_PATH)/$(BIN_NAME).rs.o
BIN_RS ?= $(ROOT_DIR)/examples/$(BIN_NAME)/src/mod.rs

LIBUNI_RS_MAIN = $(SRC)/lib.rs
LIBUNI_RS_SRC = $(LIBUNI_RS_MAIN)
LIBUNI_ASM_SRC = $(SRC)/arch/$(ARCH)/boot.S
LIBUNI_ASM_OBJ = $(LIBUNI_ASM_SRC:.S=.o)

bin: $(BIN_NAME)

$(BIN_NAME): libuni.a
	$(RUSTC) $(BIN_RS) --crate-type lib --emit obj -o $(BIN_OBJ)
	$(LD) $(LDFLAGS) $(BIN_OBJ) $^ -o $@

libuni.a: libuni.rs.o $(LIBUNI_ASM_OBJ)
	$(AR) rcs $@ $^

libuni.rs.o: $(LIBUNI_RS_SRC)
	$(RUSTC) $(LIBUNI_RS_MAIN) --crate-type lib --emit obj -o $@

%.o: %.S
	$(AS) -c $< -o $@

clean:
	rm -rf libuni.a libuni.rs.o $(LIBUNI_ASM_OBJ)
