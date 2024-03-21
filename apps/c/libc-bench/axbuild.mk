libc-bench-dir := $(APP)/libc-bench-master
package-name := libc-bench-master

bench-obj := $(package-name)/libctest.o

app-objs := $(bench-obj)

force_rebuild:

$(libc-bench-dir):
	@echo "Download libc-bench code"
	wget https://git.musl-libc.org/cgit/libc-bench/snapshot/$(package-name).tar.gz -P $(APP)
	tar -zxvf $(APP)/$(package-name).tar.gz -C $(APP) && rm -f $(APP)/$(package-name).tar.gz
	patch -p1 -N -d $(libc-bench-dir) --no-backup-if-mismatch -r - < $(APP)/bench.patch
	mv $(APP)/$(package-name)/main.c $(APP)/$(package-name)/test.c

$(APP)/$(bench-obj): build_libc_bench

build_libc_bench: $(libc-bench-dir) force_rebuild
	cd $(libc-bench-dir) && $(MAKE) CC=$(CC) CFLAGS="$(CFLAGS)" -j

clean_c::
	$(MAKE) -C $(libc-bench-dir) clean

.PHONY: build_libc_bench clean_c force_rebuild
