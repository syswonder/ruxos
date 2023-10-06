nginx-version := 1.24.0
nginx-dir := $(APP)/nginx-$(nginx-version)
nginx-objs := nginx-$(nginx-version)/objs/nginx_app.o

app-objs := $(nginx-objs)

CFLAGS += -Wno-format

nginx-build-args := \
  CC=$(CC) \
  CFLAGS="$(CFLAGS)" \
  USE_JEMALLOC=no \
  -j

ifneq ($(V),)
  nginx-build-args += V=$(V)
endif

$(nginx-dir):
	@echo "Download nginx source code"

$(APP)/$(nginx-objs): build_nginx

build_nginx: $(nginx-dir)
	cd $(nginx-dir) && $(MAKE) $(nginx-build-args)

clean_c::
	$(MAKE) -C $(nginx-dir) distclean

.PHONY: build_nginx clean_c
