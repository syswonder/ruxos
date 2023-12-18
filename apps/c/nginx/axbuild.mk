nginx-version = 1.24.0
nginx-src := $(APP)/nginx-$(nginx-version)
nginx-objdir := $(APP)/objs
nginx-objs := objs/nginx_app.o

app-objs := $(nginx-objs)

CFLAGS += -Wno-format

nginx-build-args := \
  CC=$(CC) \
  CFLAGS="$(CFLAGS)" \
  USE_JEMALLOC=no \
  -j
$(info lhw debug CC $(CC) CFLAGS $(CFLAGS) )
$(info lhw debug LD $(LD) LDFLAGS $(LDFLAGS) )
$(info lhw debug copy obj: $(OBJCOPY), $(OUT_ELF))

ifneq ($(V),)
  nginx-build-args += V=$(V)
endif

ifeq ($(V9P),y)
  DISK_ARG = 9p
else
  DISK_ARG = no_9p
endif


disk.img:
	echo "nginx makefile create_nginx_img"
	./$(APP)/create_nginx_img.sh $(DISK_ARG)

$(nginx-objdir):
	git clone https://github.com/lhw2002426/nginx-app.git -b nginx-objs $(APP)/objs

$(nginx-src):
	@echo "Download nginx source code"
	wget https://nginx.org/download/nginx-$(nginx-version).tar.gz -P $(APP)
	tar -zxvf $(APP)/nginx-$(nginx-version).tar.gz -C $(APP) && rm -f $(APP)/nginx-$(nginx-version).tar.gz
	cd $(nginx-src) && git init && git add .
#	patch -p1 -N -d $(nginx-objdir) --no-backup-if-mismatch -r - < $(APP)/nginx.patch

$(APP)/$(nginx-objs): build_nginx

clean_c::
	find . -type f \( -name "*.o" -o -name "*.elf" -o -name "*.bin" \) -exec rm -f {} +

build_nginx: $(nginx-src) disk.img $(nginx-objdir)
	cd $(nginx-objdir) && $(MAKE) $(nginx-build-args)


.PHONY: build_nginx
