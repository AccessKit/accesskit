CFLAGS := -Werror -Wall -Wextra -Wpedantic -g -I src/

PROFILE := release
DESTDIR=/usr/local

ifeq ($(CC), clang)
	CFLAGS += -fsanitize=address -fsanitize=undefined
	LDFLAGS += -fsanitize=address
endif

ifeq ($(PROFILE), release)
	CFLAGS += -O3
	CARGOFLAGS += --release
endif

../../target/$(PROFILE):
	mkdir -p $@

../../target/$(PROFILE)/include/accesskit.h: src/*.rs cbindgen.toml ../../target/$(PROFILE)
	mkdir -p ../../target/$(PROFILE)/include
	cbindgen --lang C | clang-format --assume-filename=*.c > $@

../../target/$(PROFILE)/libaccesskit.a: src/*.rs Cargo.toml
	RUSTFLAGS="-C metadata=accesskit" cargo build $(CARGOFLAGS)

install: ../../target/$(PROFILE)/libaccesskit.a
	mkdir -p $(DESTDIR)/lib
	install target/$(PROFILE)/libaccesskit.a $(DESTDIR)/lib/libaccesskit.a
	mkdir -p $(DESTDIR)/include
	install ../../target/$(PROFILE)/include/accesskit.h $(DESTDIR)/include/

clean:
	rm -rf ../../target
