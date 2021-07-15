BUILD_TYPE ?= debug
JAR = build/accesskit-mac.jar
JAVA_SRC = java/dev/accesskit/mac/AccessKitMacManager.java
DYLIB = ../../../target/$(BUILD_TYPE)/libaccesskit_mac_jni.dylib
RUST_SRC = src/lib.rs
DIST_TGZ = build/accesskit-mac-jni-$(BUILD_TYPE).tar.gz
DIST_DIR = build/dist

all: $(DIST_TGZ)

$(JAR): $(JAVA_SRC)
	mkdir -p build/classes
	javac -d build/classes $<
	jar cvf $@ -C build/classes .

# TODO: translate BUILD_TYPE to cargo argument
$(DYLIB): $(RUST_SRC)
	cargo build

$(DIST_TGZ): $(JAVA_SRC) $(JAR) $(DYLIB)
	rm -rf $(DIST_DIR)
	mkdir -p $(DIST_DIR)
	cp -a java $(DIST_DIR)/javasrc
	cp -a $(JAR) $(DIST_DIR)/
	cp -a $(DYLIB) $(DIST_DIR)/
	tar zcvf $(DIST_TGZ) -C $(DIST_DIR) .

.PHONY: all
