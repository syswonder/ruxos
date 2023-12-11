
CC =	aarch64-linux-musl-gcc
#CFLAGS =  -pipe  -O -W -Wall -Wpointer-arith -Wno-unused-parameter -g
ifeq ($(MUSL), y)
	CFLAGS = -DAX_CONFIG_SMP -DAX_CONFIG_ALLOC -DAX_CONFIG_FP_SIMD -DAX_CONFIG_MULTITASK -DAX_CONFIG_FS -DAX_CONFIG_NET -DAX_CONFIG_PIPE -DAX_CONFIG_EPOLL -DAX_CONFIG_POLL -DAX_CONFIG_FD -DAX_LOG_INFO -DAX_CONFIG_SELECT -nostdinc -fno-builtin -ffreestanding -Wall -I../../../../ulib/axmusl/musl-1.2.3/include -O3 -Wno-format
else
	CFLAGS = -DAX_CONFIG_SMP -DAX_CONFIG_ALLOC -DAX_CONFIG_FP_SIMD -DAX_CONFIG_MULTITASK -DAX_CONFIG_FS -DAX_CONFIG_NET -DAX_CONFIG_PIPE -DAX_CONFIG_EPOLL -DAX_CONFIG_POLL -DAX_CONFIG_FD -DAX_LOG_INFO -DAX_CONFIG_SELECT -nostdinc -fno-builtin -ffreestanding -Wall -I../../../../ulib/axlibc/include -O3 -Wno-format
endif
#CFLAGS = -DAX_CONFIG_SMP -DAX_CONFIG_ALLOC -DAX_CONFIG_FP_SIMD -DAX_CONFIG_MULTITASK -DAX_CONFIG_FS -DAX_CONFIG_NET -DAX_CONFIG_PIPE -DAX_CONFIG_EPOLL -DAX_CONFIG_POLL -DAX_CONFIG_FD -DAX_LOG_INFO -DAX_CONFIG_SELECT -nostdinc -fno-builtin -ffreestanding -Wall -O3 -Wno-format
CPP =	cc -E
PLATFORM_NAME ?= aarch64-qemu-virt
LD_SCRIPT := ../../../../modules/axhal/linker_$(PLATFORM_NAME).lds
LINK =	rust-lld -flavor gnu -nostdlib -static -no-pie --gc-sections -T$(LD_SCRIPT) --error-limit=0
nginx-version = 1.24.0
SRC = ../nginx-$(nginx-version)

ifeq ($(MUSL), y)
  LIBC = ../../../../ulib/axmusl/install/lib/libc.a
else
  LIBC = \
        ../../../../ulib/axlibc/build_aarch64/libc.a \
        ../../../../target/aarch64-unknown-none-softfloat/release/libaxlibc.a
endif

OBJCOPY ?= rust-objcopy --binary-architecture=aarch64
FEAT = axfeat/log-level-info axfeat/paging axfeat/irq axlibc/smp axlibc/alloc axlibc/fp_simd axlibc/multitask axlibc/fs axlibc/net axlibc/pipe axlibc/epoll axlibc/poll axlibc/select axlibc/fd

define run_cmd
  @printf '$(WHITE_C)$(1)$(END_C) $(GRAY_C)$(2)$(END_C)\n'
  @$(1) $(2)
endef

OBJ_DIRS = ./src/core ./src/http ./src/event ./src/os/unix ./src/event/modules ./src/http/modules

ALL_INCS = -I $(SRC)/src/core \
	-I $(SRC)/src/event \
	-I $(SRC)/src/event/modules \
	-I $(SRC)/src/os/unix \
	-I . \
	-I $(SRC)/src/http \
	-I $(SRC)/src/http/modules


CORE_DEPS = $(SRC)/src/core/nginx.h \
	$(SRC)/src/core/ngx_config.h \
	$(SRC)/src/core/ngx_core.h \
	$(SRC)/src/core/ngx_log.h \
	$(SRC)/src/core/ngx_palloc.h \
	$(SRC)/src/core/ngx_array.h \
	$(SRC)/src/core/ngx_list.h \
	$(SRC)/src/core/ngx_hash.h \
	$(SRC)/src/core/ngx_buf.h \
	$(SRC)/src/core/ngx_queue.h \
	$(SRC)/src/core/ngx_string.h \
	$(SRC)/src/core/ngx_parse.h \
	$(SRC)/src/core/ngx_parse_time.h \
	$(SRC)/src/core/ngx_inet.h \
	$(SRC)/src/core/ngx_file.h \
	$(SRC)/src/core/ngx_crc.h \
	$(SRC)/src/core/ngx_crc32.h \
	$(SRC)/src/core/ngx_murmurhash.h \
	$(SRC)/src/core/ngx_md5.h \
	$(SRC)/src/core/ngx_sha1.h \
	$(SRC)/src/core/ngx_rbtree.h \
	$(SRC)/src/core/ngx_radix_tree.h \
	$(SRC)/src/core/ngx_rwlock.h \
	$(SRC)/src/core/ngx_slab.h \
	$(SRC)/src/core/ngx_times.h \
	$(SRC)/src/core/ngx_shmtx.h \
	$(SRC)/src/core/ngx_connection.h \
	$(SRC)/src/core/ngx_cycle.h \
	$(SRC)/src/core/ngx_conf_file.h \
	$(SRC)/src/core/ngx_module.h \
	$(SRC)/src/core/ngx_resolver.h \
	$(SRC)/src/core/ngx_open_file_cache.h \
	$(SRC)/src/core/ngx_crypt.h \
	$(SRC)/src/core/ngx_proxy_protocol.h \
	$(SRC)/src/core/ngx_syslog.h \
	$(SRC)/src/event/ngx_event.h \
	$(SRC)/src/event/ngx_event_timer.h \
	$(SRC)/src/event/ngx_event_posted.h \
	$(SRC)/src/event/ngx_event_connect.h \
	$(SRC)/src/event/ngx_event_pipe.h \
	$(SRC)/src/event/ngx_event_udp.h \
	$(SRC)/src/os/unix/ngx_time.h \
	$(SRC)/src/os/unix/ngx_errno.h \
	$(SRC)/src/os/unix/ngx_alloc.h \
	$(SRC)/src/os/unix/ngx_files.h \
	$(SRC)/src/os/unix/ngx_channel.h \
	$(SRC)/src/os/unix/ngx_shmem.h \
	$(SRC)/src/os/unix/ngx_process.h \
	$(SRC)/src/os/unix/ngx_setaffinity.h \
	$(SRC)/src/os/unix/ngx_setproctitle.h \
	$(SRC)/src/os/unix/ngx_atomic.h \
	$(SRC)/src/os/unix/ngx_gcc_atomic_x86.h \
	$(SRC)/src/os/unix/ngx_thread.h \
	$(SRC)/src/os/unix/ngx_socket.h \
	$(SRC)/src/os/unix/ngx_os.h \
	$(SRC)/src/os/unix/ngx_user.h \
	$(SRC)/src/os/unix/ngx_dlopen.h \
	$(SRC)/src/os/unix/ngx_process_cycle.h \
	$(SRC)/src/os/unix/ngx_linux_config.h \
	$(SRC)/src/os/unix/ngx_linux.h \
	ngx_auto_config.h


CORE_INCS = -I $(SRC)/src/core \
	-I $(SRC)/src/event \
	-I $(SRC)/src/event/modules \
	-I $(SRC)/src/os/unix \
	-I .


HTTP_DEPS = $(SRC)/src/http/ngx_http.h \
	$(SRC)/src/http/ngx_http_request.h \
	$(SRC)/src/http/ngx_http_config.h \
	$(SRC)/src/http/ngx_http_core_module.h \
	$(SRC)/src/http/ngx_http_cache.h \
	$(SRC)/src/http/ngx_http_variables.h \
	$(SRC)/src/http/ngx_http_script.h \
	$(SRC)/src/http/ngx_http_upstream.h \
	$(SRC)/src/http/ngx_http_upstream_round_robin.h \
	$(SRC)/src/http/modules/ngx_http_ssi_filter_module.h


HTTP_INCS = -I $(SRC)/src/http \
	-I $(SRC)/src/http/modules

OUT_BIN = ./nginx_aarch64-qemu-q35.bin
OUT_ELF = ./nginx_aarch64-qemu-q35.elf

build:	binary modules

binary:	$(OUT_BIN)

_cargo_build:
	$(call cargo_build,-p axlibc,$(FEAT))

$(OUT_BIN): $(OUT_ELF)
	@echo "use clib:"
	@echo "($LIBC)"
	$(call run_cmd,$(OBJCOPY),$(OUT_ELF) --strip-all -O binary $@)


./nginx_aarch64-qemu-q35.elf:	./nginx_app.o

	$(LINK) -o ./nginx_aarch64-qemu-q35.elf \
	$(LIBC) \
	./nginx_app.o
	
./nginx_app.o:	./src/core/nginx.o \
	./src/core/ngx_log.o \
	./src/core/ngx_palloc.o \
	./src/core/ngx_array.o \
	./src/core/ngx_list.o \
	./src/core/ngx_hash.o \
	./src/core/ngx_buf.o \
	./src/core/ngx_queue.o \
	./src/core/ngx_output_chain.o \
	./src/core/ngx_string.o \
	./src/core/ngx_parse.o \
	./src/core/ngx_parse_time.o \
	./src/core/ngx_inet.o \
	./src/core/ngx_file.o \
	./src/core/ngx_crc32.o \
	./src/core/ngx_murmurhash.o \
	./src/core/ngx_md5.o \
	./src/core/ngx_sha1.o \
	./src/core/ngx_rbtree.o \
	./src/core/ngx_radix_tree.o \
	./src/core/ngx_slab.o \
	./src/core/ngx_times.o \
	./src/core/ngx_shmtx.o \
	./src/core/ngx_connection.o \
	./src/core/ngx_cycle.o \
	./src/core/ngx_spinlock.o \
	./src/core/ngx_rwlock.o \
	./src/core/ngx_cpuinfo.o \
	./src/core/ngx_conf_file.o \
	./src/core/ngx_module.o \
	./src/core/ngx_resolver.o \
	./src/core/ngx_open_file_cache.o \
	./src/core/ngx_crypt.o \
	./src/core/ngx_proxy_protocol.o \
	./src/core/ngx_syslog.o \
	./src/event/ngx_event.o \
	./src/event/ngx_event_timer.o \
	./src/event/ngx_event_posted.o \
	./src/event/ngx_event_accept.o \
	./src/event/ngx_event_udp.o \
	./src/event/ngx_event_connect.o \
	./src/event/ngx_event_pipe.o \
	./src/os/unix/ngx_time.o \
	./src/os/unix/ngx_errno.o \
	./src/os/unix/ngx_alloc.o \
	./src/os/unix/ngx_files.o \
	./src/os/unix/ngx_socket.o \
	./src/os/unix/ngx_recv.o \
	./src/os/unix/ngx_readv_chain.o \
	./src/os/unix/ngx_udp_recv.o \
	./src/os/unix/ngx_send.o \
	./src/os/unix/ngx_writev_chain.o \
	./src/os/unix/ngx_udp_send.o \
	./src/os/unix/ngx_udp_sendmsg_chain.o \
	./src/os/unix/ngx_channel.o \
	./src/os/unix/ngx_shmem.o \
	./src/os/unix/ngx_process.o \
	./src/os/unix/ngx_daemon.o \
	./src/os/unix/ngx_setaffinity.o \
	./src/os/unix/ngx_setproctitle.o \
	./src/os/unix/ngx_posix_init.o \
	./src/os/unix/ngx_user.o \
	./src/os/unix/ngx_dlopen.o \
	./src/os/unix/ngx_process_cycle.o \
	./src/os/unix/ngx_linux_init.o \
	./src/event/modules/ngx_epoll_module.o \
	./src/os/unix/ngx_linux_sendfile_chain.o \
	./src/event/modules/ngx_select_module.o \
	./src/http/ngx_http.o \
	./src/http/ngx_http_core_module.o \
	./src/http/ngx_http_special_response.o \
	./src/http/ngx_http_request.o \
	./src/http/ngx_http_parse.o \
	./src/http/modules/ngx_http_log_module.o \
	./src/http/ngx_http_request_body.o \
	./src/http/ngx_http_variables.o \
	./src/http/ngx_http_script.o \
	./src/http/ngx_http_upstream.o \
	./src/http/ngx_http_upstream_round_robin.o \
	./src/http/ngx_http_write_filter_module.o \
	./src/http/ngx_http_header_filter_module.o \
	./src/http/modules/ngx_http_chunked_filter_module.o \
	./src/http/modules/ngx_http_range_filter_module.o \
	./src/http/ngx_http_postpone_filter_module.o \
	./src/http/modules/ngx_http_ssi_filter_module.o \
	./src/http/modules/ngx_http_charset_filter_module.o \
	./src/http/modules/ngx_http_sub_filter_module.o \
	./src/http/modules/ngx_http_userid_filter_module.o \
	./src/http/modules/ngx_http_headers_filter_module.o \
	./src/http/ngx_http_copy_filter_module.o \
	./src/http/modules/ngx_http_not_modified_filter_module.o \
	./src/http/modules/ngx_http_static_module.o \
	./src/http/modules/ngx_http_autoindex_module.o \
	./src/http/modules/ngx_http_index_module.o \
	./src/http/modules/ngx_http_mirror_module.o \
	./src/http/modules/ngx_http_try_files_module.o \
	./src/http/modules/ngx_http_access_module.o \
	./src/http/modules/ngx_http_limit_conn_module.o \
	./src/http/modules/ngx_http_limit_req_module.o \
	./src/http/modules/ngx_http_geo_module.o \
	./src/http/modules/ngx_http_map_module.o \
	./src/http/modules/ngx_http_split_clients_module.o \
	./src/http/modules/ngx_http_referer_module.o \
	./src/http/modules/ngx_http_proxy_module.o \
	./src/http/modules/ngx_http_fastcgi_module.o \
	./src/http/modules/ngx_http_uwsgi_module.o \
	./src/http/modules/ngx_http_scgi_module.o \
	./src/http/modules/ngx_http_memcached_module.o \
	./src/http/modules/ngx_http_empty_gif_module.o \
	./src/http/modules/ngx_http_browser_module.o \
	./src/http/modules/ngx_http_upstream_hash_module.o \
	./src/http/modules/ngx_http_upstream_ip_hash_module.o \
	./src/http/modules/ngx_http_upstream_least_conn_module.o \
	./src/http/modules/ngx_http_upstream_random_module.o \
	./src/http/modules/ngx_http_upstream_keepalive_module.o \
	./src/http/modules/ngx_http_upstream_zone_module.o \
	./ngx_modules.o

	$(CC) -r -o ./nginx_app.o \
	./src/core/nginx.o \
	./src/core/ngx_log.o \
	./src/core/ngx_palloc.o \
	./src/core/ngx_array.o \
	./src/core/ngx_list.o \
	./src/core/ngx_hash.o \
	./src/core/ngx_buf.o \
	./src/core/ngx_queue.o \
	./src/core/ngx_output_chain.o \
	./src/core/ngx_string.o \
	./src/core/ngx_parse.o \
	./src/core/ngx_parse_time.o \
	./src/core/ngx_inet.o \
	./src/core/ngx_file.o \
	./src/core/ngx_crc32.o \
	./src/core/ngx_murmurhash.o \
	./src/core/ngx_md5.o \
	./src/core/ngx_sha1.o \
	./src/core/ngx_rbtree.o \
	./src/core/ngx_radix_tree.o \
	./src/core/ngx_slab.o \
	./src/core/ngx_times.o \
	./src/core/ngx_shmtx.o \
	./src/core/ngx_connection.o \
	./src/core/ngx_cycle.o \
	./src/core/ngx_spinlock.o \
	./src/core/ngx_rwlock.o \
	./src/core/ngx_cpuinfo.o \
	./src/core/ngx_conf_file.o \
	./src/core/ngx_module.o \
	./src/core/ngx_resolver.o \
	./src/core/ngx_open_file_cache.o \
	./src/core/ngx_crypt.o \
	./src/core/ngx_proxy_protocol.o \
	./src/core/ngx_syslog.o \
	./src/event/ngx_event.o \
	./src/event/ngx_event_timer.o \
	./src/event/ngx_event_posted.o \
	./src/event/ngx_event_accept.o \
	./src/event/ngx_event_udp.o \
	./src/event/ngx_event_connect.o \
	./src/event/ngx_event_pipe.o \
	./src/os/unix/ngx_time.o \
	./src/os/unix/ngx_errno.o \
	./src/os/unix/ngx_alloc.o \
	./src/os/unix/ngx_files.o \
	./src/os/unix/ngx_socket.o \
	./src/os/unix/ngx_recv.o \
	./src/os/unix/ngx_readv_chain.o \
	./src/os/unix/ngx_udp_recv.o \
	./src/os/unix/ngx_send.o \
	./src/os/unix/ngx_writev_chain.o \
	./src/os/unix/ngx_udp_send.o \
	./src/os/unix/ngx_udp_sendmsg_chain.o \
	./src/os/unix/ngx_channel.o \
	./src/os/unix/ngx_shmem.o \
	./src/os/unix/ngx_process.o \
	./src/os/unix/ngx_daemon.o \
	./src/os/unix/ngx_setaffinity.o \
	./src/os/unix/ngx_setproctitle.o \
	./src/os/unix/ngx_posix_init.o \
	./src/os/unix/ngx_user.o \
	./src/os/unix/ngx_dlopen.o \
	./src/os/unix/ngx_process_cycle.o \
	./src/os/unix/ngx_linux_init.o \
	./src/event/modules/ngx_epoll_module.o \
	./src/os/unix/ngx_linux_sendfile_chain.o \
	./src/event/modules/ngx_select_module.o \
	./src/http/ngx_http.o \
	./src/http/ngx_http_core_module.o \
	./src/http/ngx_http_special_response.o \
	./src/http/ngx_http_request.o \
	./src/http/ngx_http_parse.o \
	./src/http/modules/ngx_http_log_module.o \
	./src/http/ngx_http_request_body.o \
	./src/http/ngx_http_variables.o \
	./src/http/ngx_http_script.o \
	./src/http/ngx_http_upstream.o \
	./src/http/ngx_http_upstream_round_robin.o \
	./src/http/ngx_http_write_filter_module.o \
	./src/http/ngx_http_header_filter_module.o \
	./src/http/modules/ngx_http_chunked_filter_module.o \
	./src/http/modules/ngx_http_range_filter_module.o \
	./src/http/ngx_http_postpone_filter_module.o \
	./src/http/modules/ngx_http_ssi_filter_module.o \
	./src/http/modules/ngx_http_charset_filter_module.o \
	./src/http/modules/ngx_http_sub_filter_module.o \
	./src/http/modules/ngx_http_userid_filter_module.o \
	./src/http/modules/ngx_http_headers_filter_module.o \
	./src/http/ngx_http_copy_filter_module.o \
	./src/http/modules/ngx_http_not_modified_filter_module.o \
	./src/http/modules/ngx_http_static_module.o \
	./src/http/modules/ngx_http_autoindex_module.o \
	./src/http/modules/ngx_http_index_module.o \
	./src/http/modules/ngx_http_mirror_module.o \
	./src/http/modules/ngx_http_try_files_module.o \
	./src/http/modules/ngx_http_access_module.o \
	./src/http/modules/ngx_http_limit_conn_module.o \
	./src/http/modules/ngx_http_limit_req_module.o \
	./src/http/modules/ngx_http_geo_module.o \
	./src/http/modules/ngx_http_map_module.o \
	./src/http/modules/ngx_http_split_clients_module.o \
	./src/http/modules/ngx_http_referer_module.o \
	./src/http/modules/ngx_http_proxy_module.o \
	./src/http/modules/ngx_http_fastcgi_module.o \
	./src/http/modules/ngx_http_uwsgi_module.o \
	./src/http/modules/ngx_http_scgi_module.o \
	./src/http/modules/ngx_http_memcached_module.o \
	./src/http/modules/ngx_http_empty_gif_module.o \
	./src/http/modules/ngx_http_browser_module.o \
	./src/http/modules/ngx_http_upstream_hash_module.o \
	./src/http/modules/ngx_http_upstream_ip_hash_module.o \
	./src/http/modules/ngx_http_upstream_least_conn_module.o \
	./src/http/modules/ngx_http_upstream_random_module.o \
	./src/http/modules/ngx_http_upstream_keepalive_module.o \
	./src/http/modules/ngx_http_upstream_zone_module.o \
	./ngx_modules.o

modules:

./ngx_modules.o:	$(CORE_DEPS) \
	./ngx_modules.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./ngx_modules.o \
		./ngx_modules.c


./src/core/nginx.o:	$(CORE_DEPS) \
	$(SRC)/src/core/nginx.c
	@mkdir -p $(dir $@)
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/core/nginx.o \
		$(SRC)/src/core/nginx.c


./src/core/ngx_log.o:	$(CORE_DEPS) \
	$(SRC)/src/core/ngx_log.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/core/ngx_log.o \
		$(SRC)/src/core/ngx_log.c


./src/core/ngx_palloc.o:	$(CORE_DEPS) \
	$(SRC)/src/core/ngx_palloc.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/core/ngx_palloc.o \
		$(SRC)/src/core/ngx_palloc.c


./src/core/ngx_array.o:	$(CORE_DEPS) \
	$(SRC)/src/core/ngx_array.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/core/ngx_array.o \
		$(SRC)/src/core/ngx_array.c


./src/core/ngx_list.o:	$(CORE_DEPS) \
	$(SRC)/src/core/ngx_list.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/core/ngx_list.o \
		$(SRC)/src/core/ngx_list.c


./src/core/ngx_hash.o:	$(CORE_DEPS) \
	$(SRC)/src/core/ngx_hash.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/core/ngx_hash.o \
		$(SRC)/src/core/ngx_hash.c


./src/core/ngx_buf.o:	$(CORE_DEPS) \
	$(SRC)/src/core/ngx_buf.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/core/ngx_buf.o \
		$(SRC)/src/core/ngx_buf.c


./src/core/ngx_queue.o:	$(CORE_DEPS) \
	$(SRC)/src/core/ngx_queue.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/core/ngx_queue.o \
		$(SRC)/src/core/ngx_queue.c


./src/core/ngx_output_chain.o:	$(CORE_DEPS) \
	$(SRC)/src/core/ngx_output_chain.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/core/ngx_output_chain.o \
		$(SRC)/src/core/ngx_output_chain.c


./src/core/ngx_string.o:	$(CORE_DEPS) \
	$(SRC)/src/core/ngx_string.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/core/ngx_string.o \
		$(SRC)/src/core/ngx_string.c


./src/core/ngx_parse.o:	$(CORE_DEPS) \
	$(SRC)/src/core/ngx_parse.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/core/ngx_parse.o \
		$(SRC)/src/core/ngx_parse.c


./src/core/ngx_parse_time.o:	$(CORE_DEPS) \
	$(SRC)/src/core/ngx_parse_time.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/core/ngx_parse_time.o \
		$(SRC)/src/core/ngx_parse_time.c


./src/core/ngx_inet.o:	$(CORE_DEPS) \
	$(SRC)/src/core/ngx_inet.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/core/ngx_inet.o \
		$(SRC)/src/core/ngx_inet.c


./src/core/ngx_file.o:	$(CORE_DEPS) \
	$(SRC)/src/core/ngx_file.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/core/ngx_file.o \
		$(SRC)/src/core/ngx_file.c


./src/core/ngx_crc32.o:	$(CORE_DEPS) \
	$(SRC)/src/core/ngx_crc32.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/core/ngx_crc32.o \
		$(SRC)/src/core/ngx_crc32.c


./src/core/ngx_murmurhash.o:	$(CORE_DEPS) \
	$(SRC)/src/core/ngx_murmurhash.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/core/ngx_murmurhash.o \
		$(SRC)/src/core/ngx_murmurhash.c


./src/core/ngx_md5.o:	$(CORE_DEPS) \
	$(SRC)/src/core/ngx_md5.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/core/ngx_md5.o \
		$(SRC)/src/core/ngx_md5.c


./src/core/ngx_sha1.o:	$(CORE_DEPS) \
	$(SRC)/src/core/ngx_sha1.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/core/ngx_sha1.o \
		$(SRC)/src/core/ngx_sha1.c


./src/core/ngx_rbtree.o:	$(CORE_DEPS) \
	$(SRC)/src/core/ngx_rbtree.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/core/ngx_rbtree.o \
		$(SRC)/src/core/ngx_rbtree.c


./src/core/ngx_radix_tree.o:	$(CORE_DEPS) \
	$(SRC)/src/core/ngx_radix_tree.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/core/ngx_radix_tree.o \
		$(SRC)/src/core/ngx_radix_tree.c


./src/core/ngx_slab.o:	$(CORE_DEPS) \
	$(SRC)/src/core/ngx_slab.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/core/ngx_slab.o \
		$(SRC)/src/core/ngx_slab.c


./src/core/ngx_times.o:	$(CORE_DEPS) \
	$(SRC)/src/core/ngx_times.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/core/ngx_times.o \
		$(SRC)/src/core/ngx_times.c


./src/core/ngx_shmtx.o:	$(CORE_DEPS) \
	$(SRC)/src/core/ngx_shmtx.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/core/ngx_shmtx.o \
		$(SRC)/src/core/ngx_shmtx.c


./src/core/ngx_connection.o:	$(CORE_DEPS) \
	$(SRC)/src/core/ngx_connection.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/core/ngx_connection.o \
		$(SRC)/src/core/ngx_connection.c


./src/core/ngx_cycle.o:	$(CORE_DEPS) \
	$(SRC)/src/core/ngx_cycle.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/core/ngx_cycle.o \
		$(SRC)/src/core/ngx_cycle.c


./src/core/ngx_spinlock.o:	$(CORE_DEPS) \
	$(SRC)/src/core/ngx_spinlock.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/core/ngx_spinlock.o \
		$(SRC)/src/core/ngx_spinlock.c


./src/core/ngx_rwlock.o:	$(CORE_DEPS) \
	$(SRC)/src/core/ngx_rwlock.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/core/ngx_rwlock.o \
		$(SRC)/src/core/ngx_rwlock.c


./src/core/ngx_cpuinfo.o:	$(CORE_DEPS) \
	$(SRC)/src/core/ngx_cpuinfo.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/core/ngx_cpuinfo.o \
		$(SRC)/src/core/ngx_cpuinfo.c


./src/core/ngx_conf_file.o:	$(CORE_DEPS) \
	$(SRC)/src/core/ngx_conf_file.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/core/ngx_conf_file.o \
		$(SRC)/src/core/ngx_conf_file.c


./src/core/ngx_module.o:	$(CORE_DEPS) \
	$(SRC)/src/core/ngx_module.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/core/ngx_module.o \
		$(SRC)/src/core/ngx_module.c


./src/core/ngx_resolver.o:	$(CORE_DEPS) \
	$(SRC)/src/core/ngx_resolver.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/core/ngx_resolver.o \
		$(SRC)/src/core/ngx_resolver.c


./src/core/ngx_open_file_cache.o:	$(CORE_DEPS) \
	$(SRC)/src/core/ngx_open_file_cache.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/core/ngx_open_file_cache.o \
		$(SRC)/src/core/ngx_open_file_cache.c


./src/core/ngx_crypt.o:	$(CORE_DEPS) \
	$(SRC)/src/core/ngx_crypt.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/core/ngx_crypt.o \
		$(SRC)/src/core/ngx_crypt.c


./src/core/ngx_proxy_protocol.o:	$(CORE_DEPS) \
	$(SRC)/src/core/ngx_proxy_protocol.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/core/ngx_proxy_protocol.o \
		$(SRC)/src/core/ngx_proxy_protocol.c


./src/core/ngx_syslog.o:	$(CORE_DEPS) \
	$(SRC)/src/core/ngx_syslog.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/core/ngx_syslog.o \
		$(SRC)/src/core/ngx_syslog.c


./src/event/ngx_event.o:	$(CORE_DEPS) \
	$(SRC)/src/event/ngx_event.c
	@mkdir -p $(dir $@)
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/event/ngx_event.o \
		$(SRC)/src/event/ngx_event.c


./src/event/ngx_event_timer.o:	$(CORE_DEPS) \
	$(SRC)/src/event/ngx_event_timer.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/event/ngx_event_timer.o \
		$(SRC)/src/event/ngx_event_timer.c


./src/event/ngx_event_posted.o:	$(CORE_DEPS) \
	$(SRC)/src/event/ngx_event_posted.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/event/ngx_event_posted.o \
		$(SRC)/src/event/ngx_event_posted.c


./src/event/ngx_event_accept.o:	$(CORE_DEPS) \
	$(SRC)/src/event/ngx_event_accept.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/event/ngx_event_accept.o \
		$(SRC)/src/event/ngx_event_accept.c


./src/event/ngx_event_udp.o:	$(CORE_DEPS) \
	$(SRC)/src/event/ngx_event_udp.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/event/ngx_event_udp.o \
		$(SRC)/src/event/ngx_event_udp.c


./src/event/ngx_event_connect.o:	$(CORE_DEPS) \
	$(SRC)/src/event/ngx_event_connect.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/event/ngx_event_connect.o \
		$(SRC)/src/event/ngx_event_connect.c


./src/event/ngx_event_pipe.o:	$(CORE_DEPS) \
	$(SRC)/src/event/ngx_event_pipe.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/event/ngx_event_pipe.o \
		$(SRC)/src/event/ngx_event_pipe.c


./src/os/unix/ngx_time.o:	$(CORE_DEPS) \
	$(SRC)/src/os/unix/ngx_time.c
	@mkdir -p $(dir $@)
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/os/unix/ngx_time.o \
		$(SRC)/src/os/unix/ngx_time.c


./src/os/unix/ngx_errno.o:	$(CORE_DEPS) \
	$(SRC)/src/os/unix/ngx_errno.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/os/unix/ngx_errno.o \
		$(SRC)/src/os/unix/ngx_errno.c


./src/os/unix/ngx_alloc.o:	$(CORE_DEPS) \
	$(SRC)/src/os/unix/ngx_alloc.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/os/unix/ngx_alloc.o \
		$(SRC)/src/os/unix/ngx_alloc.c


./src/os/unix/ngx_files.o:	$(CORE_DEPS) \
	$(SRC)/src/os/unix/ngx_files.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/os/unix/ngx_files.o \
		$(SRC)/src/os/unix/ngx_files.c


./src/os/unix/ngx_socket.o:	$(CORE_DEPS) \
	$(SRC)/src/os/unix/ngx_socket.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/os/unix/ngx_socket.o \
		$(SRC)/src/os/unix/ngx_socket.c


./src/os/unix/ngx_recv.o:	$(CORE_DEPS) \
	$(SRC)/src/os/unix/ngx_recv.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/os/unix/ngx_recv.o \
		$(SRC)/src/os/unix/ngx_recv.c


./src/os/unix/ngx_readv_chain.o:	$(CORE_DEPS) \
	$(SRC)/src/os/unix/ngx_readv_chain.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/os/unix/ngx_readv_chain.o \
		$(SRC)/src/os/unix/ngx_readv_chain.c


./src/os/unix/ngx_udp_recv.o:	$(CORE_DEPS) \
	$(SRC)/src/os/unix/ngx_udp_recv.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/os/unix/ngx_udp_recv.o \
		$(SRC)/src/os/unix/ngx_udp_recv.c


./src/os/unix/ngx_send.o:	$(CORE_DEPS) \
	$(SRC)/src/os/unix/ngx_send.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/os/unix/ngx_send.o \
		$(SRC)/src/os/unix/ngx_send.c


./src/os/unix/ngx_writev_chain.o:	$(CORE_DEPS) \
	$(SRC)/src/os/unix/ngx_writev_chain.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/os/unix/ngx_writev_chain.o \
		$(SRC)/src/os/unix/ngx_writev_chain.c


./src/os/unix/ngx_udp_send.o:	$(CORE_DEPS) \
	$(SRC)/src/os/unix/ngx_udp_send.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/os/unix/ngx_udp_send.o \
		$(SRC)/src/os/unix/ngx_udp_send.c


./src/os/unix/ngx_udp_sendmsg_chain.o:	$(CORE_DEPS) \
	$(SRC)/src/os/unix/ngx_udp_sendmsg_chain.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/os/unix/ngx_udp_sendmsg_chain.o \
		$(SRC)/src/os/unix/ngx_udp_sendmsg_chain.c


./src/os/unix/ngx_channel.o:	$(CORE_DEPS) \
	$(SRC)/src/os/unix/ngx_channel.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/os/unix/ngx_channel.o \
		$(SRC)/src/os/unix/ngx_channel.c


./src/os/unix/ngx_shmem.o:	$(CORE_DEPS) \
	$(SRC)/src/os/unix/ngx_shmem.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/os/unix/ngx_shmem.o \
		$(SRC)/src/os/unix/ngx_shmem.c


./src/os/unix/ngx_process.o:	$(CORE_DEPS) \
	$(SRC)/src/os/unix/ngx_process.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/os/unix/ngx_process.o \
		$(SRC)/src/os/unix/ngx_process.c


./src/os/unix/ngx_daemon.o:	$(CORE_DEPS) \
	$(SRC)/src/os/unix/ngx_daemon.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/os/unix/ngx_daemon.o \
		$(SRC)/src/os/unix/ngx_daemon.c


./src/os/unix/ngx_setaffinity.o:	$(CORE_DEPS) \
	$(SRC)/src/os/unix/ngx_setaffinity.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/os/unix/ngx_setaffinity.o \
		$(SRC)/src/os/unix/ngx_setaffinity.c


./src/os/unix/ngx_setproctitle.o:	$(CORE_DEPS) \
	$(SRC)/src/os/unix/ngx_setproctitle.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/os/unix/ngx_setproctitle.o \
		$(SRC)/src/os/unix/ngx_setproctitle.c


./src/os/unix/ngx_posix_init.o:	$(CORE_DEPS) \
	$(SRC)/src/os/unix/ngx_posix_init.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/os/unix/ngx_posix_init.o \
		$(SRC)/src/os/unix/ngx_posix_init.c


./src/os/unix/ngx_user.o:	$(CORE_DEPS) \
	$(SRC)/src/os/unix/ngx_user.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/os/unix/ngx_user.o \
		$(SRC)/src/os/unix/ngx_user.c


./src/os/unix/ngx_dlopen.o:	$(CORE_DEPS) \
	$(SRC)/src/os/unix/ngx_dlopen.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/os/unix/ngx_dlopen.o \
		$(SRC)/src/os/unix/ngx_dlopen.c


./src/os/unix/ngx_process_cycle.o:	$(CORE_DEPS) \
	$(SRC)/src/os/unix/ngx_process_cycle.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/os/unix/ngx_process_cycle.o \
		$(SRC)/src/os/unix/ngx_process_cycle.c


./src/os/unix/ngx_linux_init.o:	$(CORE_DEPS) \
	$(SRC)/src/os/unix/ngx_linux_init.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/os/unix/ngx_linux_init.o \
		$(SRC)/src/os/unix/ngx_linux_init.c


./src/event/modules/ngx_epoll_module.o:	$(CORE_DEPS) \
	$(SRC)/src/event/modules/ngx_epoll_module.c
	@mkdir -p $(dir $@)
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/event/modules/ngx_epoll_module.o \
		$(SRC)/src/event/modules/ngx_epoll_module.c


./src/os/unix/ngx_linux_sendfile_chain.o:	$(CORE_DEPS) \
	$(SRC)/src/os/unix/ngx_linux_sendfile_chain.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/os/unix/ngx_linux_sendfile_chain.o \
		$(SRC)/src/os/unix/ngx_linux_sendfile_chain.c


./src/event/modules/ngx_select_module.o:	$(CORE_DEPS) \
	$(SRC)/src/event/modules/ngx_select_module.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) \
		-o ./src/event/modules/ngx_select_module.o \
		$(SRC)/src/event/modules/ngx_select_module.c


./src/http/ngx_http.o:	$(CORE_DEPS) $(HTTP_DEPS) \
	$(SRC)/src/http/ngx_http.c
	@mkdir -p $(dir $@)
	$(CC) -c $(CFLAGS) $(CORE_INCS) $(HTTP_INCS) \
		-o ./src/http/ngx_http.o \
		$(SRC)/src/http/ngx_http.c


./src/http/ngx_http_core_module.o:	$(CORE_DEPS) $(HTTP_DEPS) \
	$(SRC)/src/http/ngx_http_core_module.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) $(HTTP_INCS) \
		-o ./src/http/ngx_http_core_module.o \
		$(SRC)/src/http/ngx_http_core_module.c


./src/http/ngx_http_special_response.o:	$(CORE_DEPS) $(HTTP_DEPS) \
	$(SRC)/src/http/ngx_http_special_response.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) $(HTTP_INCS) \
		-o ./src/http/ngx_http_special_response.o \
		$(SRC)/src/http/ngx_http_special_response.c


./src/http/ngx_http_request.o:	$(CORE_DEPS) $(HTTP_DEPS) \
	$(SRC)/src/http/ngx_http_request.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) $(HTTP_INCS) \
		-o ./src/http/ngx_http_request.o \
		$(SRC)/src/http/ngx_http_request.c


./src/http/ngx_http_parse.o:	$(CORE_DEPS) $(HTTP_DEPS) \
	$(SRC)/src/http/ngx_http_parse.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) $(HTTP_INCS) \
		-o ./src/http/ngx_http_parse.o \
		$(SRC)/src/http/ngx_http_parse.c


./src/http/modules/ngx_http_log_module.o:	$(CORE_DEPS) $(HTTP_DEPS) \
	$(SRC)/src/http/modules/ngx_http_log_module.c
	@mkdir -p $(dir $@)
	$(CC) -c $(CFLAGS) $(CORE_INCS) $(HTTP_INCS) \
		-o ./src/http/modules/ngx_http_log_module.o \
		$(SRC)/src/http/modules/ngx_http_log_module.c


./src/http/ngx_http_request_body.o:	$(CORE_DEPS) $(HTTP_DEPS) \
	$(SRC)/src/http/ngx_http_request_body.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) $(HTTP_INCS) \
		-o ./src/http/ngx_http_request_body.o \
		$(SRC)/src/http/ngx_http_request_body.c


./src/http/ngx_http_variables.o:	$(CORE_DEPS) $(HTTP_DEPS) \
	$(SRC)/src/http/ngx_http_variables.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) $(HTTP_INCS) \
		-o ./src/http/ngx_http_variables.o \
		$(SRC)/src/http/ngx_http_variables.c


./src/http/ngx_http_script.o:	$(CORE_DEPS) $(HTTP_DEPS) \
	$(SRC)/src/http/ngx_http_script.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) $(HTTP_INCS) \
		-o ./src/http/ngx_http_script.o \
		$(SRC)/src/http/ngx_http_script.c


./src/http/ngx_http_upstream.o:	$(CORE_DEPS) $(HTTP_DEPS) \
	$(SRC)/src/http/ngx_http_upstream.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) $(HTTP_INCS) \
		-o ./src/http/ngx_http_upstream.o \
		$(SRC)/src/http/ngx_http_upstream.c


./src/http/ngx_http_upstream_round_robin.o:	$(CORE_DEPS) $(HTTP_DEPS) \
	$(SRC)/src/http/ngx_http_upstream_round_robin.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) $(HTTP_INCS) \
		-o ./src/http/ngx_http_upstream_round_robin.o \
		$(SRC)/src/http/ngx_http_upstream_round_robin.c


./src/http/ngx_http_write_filter_module.o:	$(CORE_DEPS) $(HTTP_DEPS) \
	$(SRC)/src/http/ngx_http_write_filter_module.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) $(HTTP_INCS) \
		-o ./src/http/ngx_http_write_filter_module.o \
		$(SRC)/src/http/ngx_http_write_filter_module.c


./src/http/ngx_http_header_filter_module.o:	$(CORE_DEPS) $(HTTP_DEPS) \
	$(SRC)/src/http/ngx_http_header_filter_module.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) $(HTTP_INCS) \
		-o ./src/http/ngx_http_header_filter_module.o \
		$(SRC)/src/http/ngx_http_header_filter_module.c


./src/http/modules/ngx_http_chunked_filter_module.o:	$(CORE_DEPS) $(HTTP_DEPS) \
	$(SRC)/src/http/modules/ngx_http_chunked_filter_module.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) $(HTTP_INCS) \
		-o ./src/http/modules/ngx_http_chunked_filter_module.o \
		$(SRC)/src/http/modules/ngx_http_chunked_filter_module.c


./src/http/modules/ngx_http_range_filter_module.o:	$(CORE_DEPS) $(HTTP_DEPS) \
	$(SRC)/src/http/modules/ngx_http_range_filter_module.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) $(HTTP_INCS) \
		-o ./src/http/modules/ngx_http_range_filter_module.o \
		$(SRC)/src/http/modules/ngx_http_range_filter_module.c


./src/http/ngx_http_postpone_filter_module.o:	$(CORE_DEPS) $(HTTP_DEPS) \
	$(SRC)/src/http/ngx_http_postpone_filter_module.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) $(HTTP_INCS) \
		-o ./src/http/ngx_http_postpone_filter_module.o \
		$(SRC)/src/http/ngx_http_postpone_filter_module.c


./src/http/modules/ngx_http_ssi_filter_module.o:	$(CORE_DEPS) $(HTTP_DEPS) \
	$(SRC)/src/http/modules/ngx_http_ssi_filter_module.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) $(HTTP_INCS) \
		-o ./src/http/modules/ngx_http_ssi_filter_module.o \
		$(SRC)/src/http/modules/ngx_http_ssi_filter_module.c


./src/http/modules/ngx_http_charset_filter_module.o:	$(CORE_DEPS) $(HTTP_DEPS) \
	$(SRC)/src/http/modules/ngx_http_charset_filter_module.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) $(HTTP_INCS) \
		-o ./src/http/modules/ngx_http_charset_filter_module.o \
		$(SRC)/src/http/modules/ngx_http_charset_filter_module.c


./src/http/modules/ngx_http_sub_filter_module.o:	$(CORE_DEPS) $(HTTP_DEPS) \
	$(SRC)/src/http/modules/ngx_http_sub_filter_module.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) $(HTTP_INCS) \
		-o ./src/http/modules/ngx_http_sub_filter_module.o \
		$(SRC)/src/http/modules/ngx_http_sub_filter_module.c


./src/http/modules/ngx_http_userid_filter_module.o:	$(CORE_DEPS) $(HTTP_DEPS) \
	$(SRC)/src/http/modules/ngx_http_userid_filter_module.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) $(HTTP_INCS) \
		-o ./src/http/modules/ngx_http_userid_filter_module.o \
		$(SRC)/src/http/modules/ngx_http_userid_filter_module.c


./src/http/modules/ngx_http_headers_filter_module.o:	$(CORE_DEPS) $(HTTP_DEPS) \
	$(SRC)/src/http/modules/ngx_http_headers_filter_module.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) $(HTTP_INCS) \
		-o ./src/http/modules/ngx_http_headers_filter_module.o \
		$(SRC)/src/http/modules/ngx_http_headers_filter_module.c


./src/http/ngx_http_copy_filter_module.o:	$(CORE_DEPS) $(HTTP_DEPS) \
	$(SRC)/src/http/ngx_http_copy_filter_module.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) $(HTTP_INCS) \
		-o ./src/http/ngx_http_copy_filter_module.o \
		$(SRC)/src/http/ngx_http_copy_filter_module.c


./src/http/modules/ngx_http_not_modified_filter_module.o:	$(CORE_DEPS) $(HTTP_DEPS) \
	$(SRC)/src/http/modules/ngx_http_not_modified_filter_module.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) $(HTTP_INCS) \
		-o ./src/http/modules/ngx_http_not_modified_filter_module.o \
		$(SRC)/src/http/modules/ngx_http_not_modified_filter_module.c


./src/http/modules/ngx_http_static_module.o:	$(CORE_DEPS) $(HTTP_DEPS) \
	$(SRC)/src/http/modules/ngx_http_static_module.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) $(HTTP_INCS) \
		-o ./src/http/modules/ngx_http_static_module.o \
		$(SRC)/src/http/modules/ngx_http_static_module.c


./src/http/modules/ngx_http_autoindex_module.o:	$(CORE_DEPS) $(HTTP_DEPS) \
	$(SRC)/src/http/modules/ngx_http_autoindex_module.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) $(HTTP_INCS) \
		-o ./src/http/modules/ngx_http_autoindex_module.o \
		$(SRC)/src/http/modules/ngx_http_autoindex_module.c


./src/http/modules/ngx_http_index_module.o:	$(CORE_DEPS) $(HTTP_DEPS) \
	$(SRC)/src/http/modules/ngx_http_index_module.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) $(HTTP_INCS) \
		-o ./src/http/modules/ngx_http_index_module.o \
		$(SRC)/src/http/modules/ngx_http_index_module.c


./src/http/modules/ngx_http_mirror_module.o:	$(CORE_DEPS) $(HTTP_DEPS) \
	$(SRC)/src/http/modules/ngx_http_mirror_module.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) $(HTTP_INCS) \
		-o ./src/http/modules/ngx_http_mirror_module.o \
		$(SRC)/src/http/modules/ngx_http_mirror_module.c


./src/http/modules/ngx_http_try_files_module.o:	$(CORE_DEPS) $(HTTP_DEPS) \
	$(SRC)/src/http/modules/ngx_http_try_files_module.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) $(HTTP_INCS) \
		-o ./src/http/modules/ngx_http_try_files_module.o \
		$(SRC)/src/http/modules/ngx_http_try_files_module.c


./src/http/modules/ngx_http_access_module.o:	$(CORE_DEPS) $(HTTP_DEPS) \
	$(SRC)/src/http/modules/ngx_http_access_module.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) $(HTTP_INCS) \
		-o ./src/http/modules/ngx_http_access_module.o \
		$(SRC)/src/http/modules/ngx_http_access_module.c


./src/http/modules/ngx_http_limit_conn_module.o:	$(CORE_DEPS) $(HTTP_DEPS) \
	$(SRC)/src/http/modules/ngx_http_limit_conn_module.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) $(HTTP_INCS) \
		-o ./src/http/modules/ngx_http_limit_conn_module.o \
		$(SRC)/src/http/modules/ngx_http_limit_conn_module.c


./src/http/modules/ngx_http_limit_req_module.o:	$(CORE_DEPS) $(HTTP_DEPS) \
	$(SRC)/src/http/modules/ngx_http_limit_req_module.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) $(HTTP_INCS) \
		-o ./src/http/modules/ngx_http_limit_req_module.o \
		$(SRC)/src/http/modules/ngx_http_limit_req_module.c


./src/http/modules/ngx_http_geo_module.o:	$(CORE_DEPS) $(HTTP_DEPS) \
	$(SRC)/src/http/modules/ngx_http_geo_module.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) $(HTTP_INCS) \
		-o ./src/http/modules/ngx_http_geo_module.o \
		$(SRC)/src/http/modules/ngx_http_geo_module.c


./src/http/modules/ngx_http_map_module.o:	$(CORE_DEPS) $(HTTP_DEPS) \
	$(SRC)/src/http/modules/ngx_http_map_module.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) $(HTTP_INCS) \
		-o ./src/http/modules/ngx_http_map_module.o \
		$(SRC)/src/http/modules/ngx_http_map_module.c


./src/http/modules/ngx_http_split_clients_module.o:	$(CORE_DEPS) $(HTTP_DEPS) \
	$(SRC)/src/http/modules/ngx_http_split_clients_module.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) $(HTTP_INCS) \
		-o ./src/http/modules/ngx_http_split_clients_module.o \
		$(SRC)/src/http/modules/ngx_http_split_clients_module.c


./src/http/modules/ngx_http_referer_module.o:	$(CORE_DEPS) $(HTTP_DEPS) \
	$(SRC)/src/http/modules/ngx_http_referer_module.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) $(HTTP_INCS) \
		-o ./src/http/modules/ngx_http_referer_module.o \
		$(SRC)/src/http/modules/ngx_http_referer_module.c


./src/http/modules/ngx_http_proxy_module.o:	$(CORE_DEPS) $(HTTP_DEPS) \
	$(SRC)/src/http/modules/ngx_http_proxy_module.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) $(HTTP_INCS) \
		-o ./src/http/modules/ngx_http_proxy_module.o \
		$(SRC)/src/http/modules/ngx_http_proxy_module.c


./src/http/modules/ngx_http_fastcgi_module.o:	$(CORE_DEPS) $(HTTP_DEPS) \
	$(SRC)/src/http/modules/ngx_http_fastcgi_module.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) $(HTTP_INCS) \
		-o ./src/http/modules/ngx_http_fastcgi_module.o \
		$(SRC)/src/http/modules/ngx_http_fastcgi_module.c


./src/http/modules/ngx_http_uwsgi_module.o:	$(CORE_DEPS) $(HTTP_DEPS) \
	$(SRC)/src/http/modules/ngx_http_uwsgi_module.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) $(HTTP_INCS) \
		-o ./src/http/modules/ngx_http_uwsgi_module.o \
		$(SRC)/src/http/modules/ngx_http_uwsgi_module.c


./src/http/modules/ngx_http_scgi_module.o:	$(CORE_DEPS) $(HTTP_DEPS) \
	$(SRC)/src/http/modules/ngx_http_scgi_module.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) $(HTTP_INCS) \
		-o ./src/http/modules/ngx_http_scgi_module.o \
		$(SRC)/src/http/modules/ngx_http_scgi_module.c


./src/http/modules/ngx_http_memcached_module.o:	$(CORE_DEPS) $(HTTP_DEPS) \
	$(SRC)/src/http/modules/ngx_http_memcached_module.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) $(HTTP_INCS) \
		-o ./src/http/modules/ngx_http_memcached_module.o \
		$(SRC)/src/http/modules/ngx_http_memcached_module.c


./src/http/modules/ngx_http_empty_gif_module.o:	$(CORE_DEPS) $(HTTP_DEPS) \
	$(SRC)/src/http/modules/ngx_http_empty_gif_module.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) $(HTTP_INCS) \
		-o ./src/http/modules/ngx_http_empty_gif_module.o \
		$(SRC)/src/http/modules/ngx_http_empty_gif_module.c


./src/http/modules/ngx_http_browser_module.o:	$(CORE_DEPS) $(HTTP_DEPS) \
	$(SRC)/src/http/modules/ngx_http_browser_module.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) $(HTTP_INCS) \
		-o ./src/http/modules/ngx_http_browser_module.o \
		$(SRC)/src/http/modules/ngx_http_browser_module.c


./src/http/modules/ngx_http_upstream_hash_module.o:	$(CORE_DEPS) $(HTTP_DEPS) \
	$(SRC)/src/http/modules/ngx_http_upstream_hash_module.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) $(HTTP_INCS) \
		-o ./src/http/modules/ngx_http_upstream_hash_module.o \
		$(SRC)/src/http/modules/ngx_http_upstream_hash_module.c


./src/http/modules/ngx_http_upstream_ip_hash_module.o:	$(CORE_DEPS) $(HTTP_DEPS) \
	$(SRC)/src/http/modules/ngx_http_upstream_ip_hash_module.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) $(HTTP_INCS) \
		-o ./src/http/modules/ngx_http_upstream_ip_hash_module.o \
		$(SRC)/src/http/modules/ngx_http_upstream_ip_hash_module.c


./src/http/modules/ngx_http_upstream_least_conn_module.o:	$(CORE_DEPS) $(HTTP_DEPS) \
	$(SRC)/src/http/modules/ngx_http_upstream_least_conn_module.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) $(HTTP_INCS) \
		-o ./src/http/modules/ngx_http_upstream_least_conn_module.o \
		$(SRC)/src/http/modules/ngx_http_upstream_least_conn_module.c


./src/http/modules/ngx_http_upstream_random_module.o:	$(CORE_DEPS) $(HTTP_DEPS) \
	$(SRC)/src/http/modules/ngx_http_upstream_random_module.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) $(HTTP_INCS) \
		-o ./src/http/modules/ngx_http_upstream_random_module.o \
		$(SRC)/src/http/modules/ngx_http_upstream_random_module.c


./src/http/modules/ngx_http_upstream_keepalive_module.o:	$(CORE_DEPS) $(HTTP_DEPS) \
	$(SRC)/src/http/modules/ngx_http_upstream_keepalive_module.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) $(HTTP_INCS) \
		-o ./src/http/modules/ngx_http_upstream_keepalive_module.o \
		$(SRC)/src/http/modules/ngx_http_upstream_keepalive_module.c


./src/http/modules/ngx_http_upstream_zone_module.o:	$(CORE_DEPS) $(HTTP_DEPS) \
	$(SRC)/src/http/modules/ngx_http_upstream_zone_module.c
	$(CC) -c $(CFLAGS) $(CORE_INCS) $(HTTP_INCS) \
		-o ./src/http/modules/ngx_http_upstream_zone_module.o \
		$(SRC)/src/http/modules/ngx_http_upstream_zone_module.c


manpage:	./nginx.8

./nginx.8:	man/nginx.8 ./ngx_auto_config.h
	sed -e "s|%%PREFIX%%|/nginx|" \
		-e "s|%%PID_PATH%%|/nginx/logs/nginx.pid|" \
		-e "s|%%CONF_PATH%%|/nginx/conf/nginx.conf|" \
		-e "s|%%ERROR_LOG_PATH%%|/nginx/logs/error.log|" \
		< man/nginx.8 > $@

install:	build 
	test -d '$(DESTDIR)/nginx' || mkdir -p '$(DESTDIR)/nginx'

	test -d '$(DESTDIR)/nginx/sbin' \
		|| mkdir -p '$(DESTDIR)/nginx/sbin'
	test ! -f '$(DESTDIR)/nginx/sbin/nginx' \
		|| mv '$(DESTDIR)/nginx/sbin/nginx' \
			'$(DESTDIR)/nginx/sbin/nginx.old'
	cp ./nginx '$(DESTDIR)/nginx/sbin/nginx'

	test -d '$(DESTDIR)/nginx/conf' \
		|| mkdir -p '$(DESTDIR)/nginx/conf'

	cp conf/koi-win '$(DESTDIR)/nginx/conf'
	cp conf/koi-utf '$(DESTDIR)/nginx/conf'
	cp conf/win-utf '$(DESTDIR)/nginx/conf'

	test -f '$(DESTDIR)/nginx/conf/mime.types' \
		|| cp conf/mime.types '$(DESTDIR)/nginx/conf'
	cp conf/mime.types '$(DESTDIR)/nginx/conf/mime.types.default'

	test -f '$(DESTDIR)/nginx/conf/fastcgi_params' \
		|| cp conf/fastcgi_params '$(DESTDIR)/nginx/conf'
	cp conf/fastcgi_params \
		'$(DESTDIR)/nginx/conf/fastcgi_params.default'

	test -f '$(DESTDIR)/nginx/conf/fastcgi.conf' \
		|| cp conf/fastcgi.conf '$(DESTDIR)/nginx/conf'
	cp conf/fastcgi.conf '$(DESTDIR)/nginx/conf/fastcgi.conf.default'

	test -f '$(DESTDIR)/nginx/conf/uwsgi_params' \
		|| cp conf/uwsgi_params '$(DESTDIR)/nginx/conf'
	cp conf/uwsgi_params \
		'$(DESTDIR)/nginx/conf/uwsgi_params.default'

	test -f '$(DESTDIR)/nginx/conf/scgi_params' \
		|| cp conf/scgi_params '$(DESTDIR)/nginx/conf'
	cp conf/scgi_params \
		'$(DESTDIR)/nginx/conf/scgi_params.default'

	test -f '$(DESTDIR)/nginx/conf/nginx.conf' \
		|| cp conf/nginx.conf '$(DESTDIR)/nginx/conf/nginx.conf'
	cp conf/nginx.conf '$(DESTDIR)/nginx/conf/nginx.conf.default'

	test -d '$(DESTDIR)/nginx/logs' \
		|| mkdir -p '$(DESTDIR)/nginx/logs'

	test -d '$(DESTDIR)/nginx/logs' \
		|| mkdir -p '$(DESTDIR)/nginx/logs'

	test -d '$(DESTDIR)/nginx/html' \
		|| cp -R html '$(DESTDIR)/nginx'

	test -d '$(DESTDIR)/nginx/logs' \
		|| mkdir -p '$(DESTDIR)/nginx/logs'
