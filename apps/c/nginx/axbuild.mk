nginx-version := 1.24.0
nginx-dir := $(APP)/nginx-$(nginx-version)
nginx-objs := nginx-$(nginx-version)/objs/nginx_app.o
#nginx-dir := $(APP)/nginx-app
#nginx-objs := nginx-app/objs/nginx_app.o

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

ifeq ($(V9P),y)
  DISK_ARG = 9p
else
  DISK_ARG = no_9p
endif


disk.img:
	ls
	echo "nginx makefile create_nginx_img"
	./$(APP)/create_nginx_img.sh $(DISK_ARG)

$(nginx-dir):
	@echo "Download nginx source code"
	wget https://nginx.org/download/nginx-$(nginx-version).tar.gz -P $(APP)
	tar -zxvf $(APP)/nginx-$(nginx-version).tar.gz -C $(APP) && rm -f $(APP)/nginx-$(nginx-version).tar.gz
	cd $(nginx-dir) && git init && git add .
	patch -p1 -N -d $(nginx-dir) --no-backup-if-mismatch -r - < $(APP)/nginx.patch

$(APP)/$(nginx-objs): build_nginx

build_nginx: $(nginx-dir) disk.img
	cd $(nginx-dir) && $(MAKE) $(nginx-build-args)

clean_c::
	$(MAKE) -C $(nginx-dir) distclean

.PHONY: build_nginx clean_c
