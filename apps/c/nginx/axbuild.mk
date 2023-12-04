#nginx-version := 1.24.0
#nginx-dir := $(APP)/nginx-$(nginx-version)
#nginx-objs := nginx-$(nginx-version)/objs/nginx_app.o
nginx-dir := $(APP)/nginx-app
nginx-objs := nginx-app/objs/nginx_app.o

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

disk.img:
	ls
	echo "nginx makefile create_nginx_img"
	./$(APP)/create_nginx_img.sh

$(nginx-dir):
	git clone https://github.com/lhw2002426/nginx-app.git $(APP)/nginx-app
	@echo "Download nginx source code"

$(APP)/$(nginx-objs): build_nginx

build_nginx: $(nginx-dir) disk.img
	cd $(nginx-dir) && $(MAKE) $(nginx-build-args)

clean_c::
	$(MAKE) -C $(nginx-dir) distclean

.PHONY: build_nginx clean_c
