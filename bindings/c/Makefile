SHELL = /usr/bin/env bash
.SHELLFLAGS = -o pipefail -c

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

include:
	mkdir -p $@

include/accesskit.h: src/*.rs cbindgen.toml include
	rustup run nightly cbindgen | clang-format '--assume-filename=*.c' > $@

../../target/$(PROFILE)/libaccesskit.a: src/*.rs Cargo.toml
	cargo build $(CARGOFLAGS)

install: ../../target/$(PROFILE)/libaccesskit.a
	mkdir -p $(DESTDIR)/lib
	install target/$(PROFILE)/libaccesskit.a $(DESTDIR)/lib/libaccesskit.a
	mkdir -p $(DESTDIR)/include
	install include/accesskit.h $(DESTDIR)/include/

clean:
	rm -rf include
	rm -rf ../../target
