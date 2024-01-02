wamr-version := 6dbfeb25dd164c0ffcec21806e1c1cd0dff27c58
wamr-dir := $(APP)/wasm-micro-runtime-$(wamr-version)

CMAKE = cmake

ARCH ?= x86_64
ARCH_UPPER ?= $(shell echo $(ARCH) | tr '[a-z]' '[A-Z]')

app-objs := wamr.o
wamr_product_dir = $(wamr-dir)/product-mini/platforms/ruxos
wamr_product_build = $(wamr_product_dir)/build

$(wamr-dir):
	@echo "Download wamr source code"
	wget https://github.com/intel/wasm-micro-runtime/archive/$(wamr-version).tar.gz -P $(APP)
	tar -zxf $(APP)/$(wamr-version).tar.gz -C $(APP) && rm -f $(APP)/$(wamr-version).tar.gz
	cd $(wamr-dir) && git init && git add .
	patch -p1 -N -d $(wamr-dir) --no-backup-if-mismatch -r - < $(APP)/wamr.patch

$(APP)/$(app-objs): build_wamr

build_wamr: $(wamr-dir) $(APP)/axbuild.mk
	mkdir -p $(wamr_product_dir) && cp -r $(wamr_product_dir)/../linux/* $(wamr_product_dir) && cp $(APP)/CMakeLists.txt $(wamr_product_dir)
	cd $(wamr_product_dir) && mkdir -p build && cd build && $(CMAKE) .. -DWAMR_BUILD_TARGET=$(ARCH_UPPER) -DWAMR_DISABLE_HW_BOUND_CHECK=1 && $(MAKE)
	cp $(wamr_product_build)/libiwasm.a $(app-objs)

clean_c::
	rm -rf $(wamr_product_build)
	rm -f $(APP)/$(app-objs)

.PHONY: build_wamr clean_c
