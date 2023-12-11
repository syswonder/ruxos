#define NGX_CONFIGURE "\
  --prefix=/nginx \
  --with-http_sub_module \
  --with-select_module \
  --without-http_gzip_module \
  --without-pcre \
  --without-http_rewrite_module \
  --without-http_auth_basic_module \
  --without-http-cache"

#ifndef NGX_DEBUG
#ifdef CONFIG_LIBNGINX_DEBUG
#define NGX_DEBUG  1
#else
#define NGX_DEBUG  0
#endif
#endif

#ifndef NGX_COMPILER
#define NGX_COMPILER  "gcc 10.3.0 (Ubuntu 10.3.0-1ubuntu1~18.04~1) "
#endif

#ifndef NGX_HAVE_GCC_ATOMIC
#define NGX_HAVE_GCC_ATOMIC  1
#endif

#ifndef NGX_HAVE_C99_VARIADIC_MACROS
#define NGX_HAVE_C99_VARIADIC_MACROS  1
#endif

#ifndef NGX_HAVE_GCC_VARIADIC_MACROS
#define NGX_HAVE_GCC_VARIADIC_MACROS  1
#endif

#ifndef NGX_HAVE_GCC_BSWAP64
#define NGX_HAVE_GCC_BSWAP64  1
#endif

#ifndef NGX_HAVE_EPOLL
#define NGX_HAVE_EPOLL  1
#endif

#ifndef NGX_HAVE_CLEAR_EVENT
#define NGX_HAVE_CLEAR_EVENT  1
#endif

#ifndef NGX_HAVE_EPOLLRDHUP
#define NGX_HAVE_EPOLLRDHUP  0
#endif

#ifndef NGX_HAVE_EPOLLEXCLUSIVE
#define NGX_HAVE_EPOLLEXCLUSIVE  0
#endif

#ifndef NGX_HAVE_O_PATH
#define NGX_HAVE_O_PATH  0
#endif

#ifndef NGX_HAVE_SENDFILE
#define NGX_HAVE_SENDFILE  1
#endif

#ifndef NGX_HAVE_SENDFILE64
#define NGX_HAVE_SENDFILE64  1
#endif

#ifndef NGX_HAVE_PR_SET_DUMPABLE
#define NGX_HAVE_PR_SET_DUMPABLE  1
#endif

#ifndef NGX_HAVE_PR_SET_KEEPCAPS
#define NGX_HAVE_PR_SET_KEEPCAPS  1
#endif

#ifndef NGX_HAVE_CAPABILITIES
#define NGX_HAVE_CAPABILITIES  0
#endif

#ifndef NGX_HAVE_GNU_CRYPT_R
#define NGX_HAVE_GNU_CRYPT_R  1
#endif

#ifndef NGX_HAVE_NONALIGNED
#define NGX_HAVE_NONALIGNED  1
#endif

#ifndef NGX_CPU_CACHE_LINE
#define NGX_CPU_CACHE_LINE  64
#endif

#define NGX_KQUEUE_UDATA_T  (void *)

#ifndef NGX_HAVE_POSIX_FADVISE
#define NGX_HAVE_POSIX_FADVISE  0
#endif

#ifndef NGX_HAVE_O_DIRECT
#define NGX_HAVE_O_DIRECT  0
#endif

#ifndef NGX_HAVE_ALIGNED_DIRECTIO
#define NGX_HAVE_ALIGNED_DIRECTIO  0
#endif

#ifndef NGX_HAVE_STATFS
#define NGX_HAVE_STATFS  1
#endif

#ifndef NGX_HAVE_STATVFS
#define NGX_HAVE_STATVFS  1
#endif

#ifdef CONFIG_LIBNGINX_HTTP_UPSTREAM_RANDOM
#ifndef NGX_STAT_STUB
#define NGX_STAT_STUB  1
#endif
#endif

#ifndef NGX_HAVE_DLOPEN
#define NGX_HAVE_DLOPEN  1
#endif

#ifndef NGX_HAVE_SCHED_YIELD
#define NGX_HAVE_SCHED_YIELD  1
#endif

#ifndef NGX_HAVE_SCHED_SETAFFINITY
#define NGX_HAVE_SCHED_SETAFFINITY  0
#endif

#ifndef NGX_HAVE_REUSEPORT
#define NGX_HAVE_REUSEPORT  1
#endif

#ifndef NGX_HAVE_TRANSPARENT_PROXY
#define NGX_HAVE_TRANSPARENT_PROXY  1
#endif

#ifndef NGX_HAVE_IP_BIND_ADDRESS_NO_PORT
#define NGX_HAVE_IP_BIND_ADDRESS_NO_PORT  0
#endif

#ifndef NGX_HAVE_IP_PKTINFO
#define NGX_HAVE_IP_PKTINFO  0
#endif

#ifndef NGX_HAVE_IPV6_RECVPKTINFO
#define NGX_HAVE_IPV6_RECVPKTINFO  0
#endif

#ifndef NGX_HAVE_DEFERRED_ACCEPT
#define NGX_HAVE_DEFERRED_ACCEPT  1
#endif

#ifndef NGX_HAVE_KEEPALIVE_TUNABLE
#define NGX_HAVE_KEEPALIVE_TUNABLE  1
#endif

#ifndef NGX_HAVE_TCP_FASTOPEN
#define NGX_HAVE_TCP_FASTOPEN  0
#endif

#ifndef NGX_HAVE_TCP_INFO
#define NGX_HAVE_TCP_INFO  0
#endif

#ifndef NGX_HAVE_ACCEPT4
#define NGX_HAVE_ACCEPT4  0
#endif

#ifndef NGX_HAVE_EVENTFD
#define NGX_HAVE_EVENTFD  0
#endif

#ifndef NGX_HAVE_SYS_EVENTFD_H
#define NGX_HAVE_SYS_EVENTFD_H  1
#endif

#ifndef NGX_HAVE_UNIX_DOMAIN
#define NGX_HAVE_UNIX_DOMAIN  0
#endif

#ifndef NGX_PTR_SIZE
#define NGX_PTR_SIZE  8
#endif

#ifndef NGX_SIG_ATOMIC_T_SIZE
#define NGX_SIG_ATOMIC_T_SIZE  4
#endif

#ifndef NGX_HAVE_LITTLE_ENDIAN
#define NGX_HAVE_LITTLE_ENDIAN  1
#endif

#ifndef NGX_MAX_SIZE_T_VALUE
#define NGX_MAX_SIZE_T_VALUE  9223372036854775807LL
#endif

#ifndef NGX_SIZE_T_LEN
#define NGX_SIZE_T_LEN  (sizeof("-9223372036854775808") - 1)
#endif

#ifndef NGX_MAX_OFF_T_VALUE
#define NGX_MAX_OFF_T_VALUE  9223372036854775807LL
#endif

#ifndef NGX_OFF_T_LEN
#define NGX_OFF_T_LEN  (sizeof("-9223372036854775808") - 1)
#endif

#ifndef NGX_TIME_T_SIZE
#define NGX_TIME_T_SIZE  8
#endif

#ifndef NGX_TIME_T_LEN
#define NGX_TIME_T_LEN  (sizeof("-9223372036854775808") - 1)
#endif

#ifndef NGX_MAX_TIME_T_VALUE
#define NGX_MAX_TIME_T_VALUE  9223372036854775807LL
#endif

#ifndef NGX_HAVE_INET6
#define NGX_HAVE_INET6  0
#endif

#ifndef NGX_HAVE_PREAD
#define NGX_HAVE_PREAD  0
#endif

/*#ifndef NGX_HAVE_PWRITE
#define NGX_HAVE_PWRITE  1
#endif*/

#ifndef NGX_HAVE_PWRITEV
#define NGX_HAVE_PWRITEV  1
#endif

#ifndef NGX_SYS_NERR
#define NGX_SYS_NERR  12 /* was 135, Unikraft does not have all the error codes */
#endif

#ifndef NGX_HAVE_LOCALTIME_R
#define NGX_HAVE_LOCALTIME_R  1
#endif

#ifndef NGX_HAVE_CLOCK_MONOTONIC
#define NGX_HAVE_CLOCK_MONOTONIC  0
#endif

#ifndef NGX_HAVE_POSIX_MEMALIGN
#define NGX_HAVE_POSIX_MEMALIGN  1
#endif

#ifndef NGX_HAVE_MEMALIGN
#define NGX_HAVE_MEMALIGN  1
#endif

#ifndef NGX_HAVE_MAP_ANON
#define NGX_HAVE_MAP_ANON  1
#endif

#ifndef NGX_HAVE_MAP_DEVZERO
#define NGX_HAVE_MAP_DEVZERO  1
#endif

#ifndef NGX_HAVE_SYSVSHM
#define NGX_HAVE_SYSVSHM  1
#endif

#ifndef NGX_HAVE_POSIX_SEM
#define NGX_HAVE_POSIX_SEM  1
#endif

#ifndef NGX_HAVE_MSGHDR_MSG_CONTROL
#define NGX_HAVE_MSGHDR_MSG_CONTROL  1
#endif

#ifndef NGX_HAVE_FIONBIO
#define NGX_HAVE_FIONBIO  1 /* ioctl(FIONBIO) */
#endif

#ifndef NGX_HAVE_GMTOFF
#define NGX_HAVE_GMTOFF  1
#endif

#ifndef NGX_HAVE_D_TYPE
#define NGX_HAVE_D_TYPE  1
#endif

#ifndef NGX_HAVE_SC_NPROCESSORS_ONLN
#define NGX_HAVE_SC_NPROCESSORS_ONLN  1
#endif

#ifndef NGX_HAVE_LEVEL1_DCACHE_LINESIZE
#define NGX_HAVE_LEVEL1_DCACHE_LINESIZE  0
#endif

#ifndef NGX_HAVE_OPENAT
#define NGX_HAVE_OPENAT  1
#endif

#ifndef NGX_HAVE_GETADDRINFO
#define NGX_HAVE_GETADDRINFO  1
#endif

#ifndef NGX_HAVE_SELECT
#define NGX_HAVE_SELECT  1
#endif

#ifdef CONFIG_LIBNGINX_HTTP_V2
#ifndef NGX_HTTP_V2
#define NGX_HTTP_V2  1
#endif
#else
#ifndef NGX_HTTP_V2
#define NGX_HTTP_V2  0
#endif
#endif

#ifndef NGX_SSL
#ifdef CONFIG_LIBSSL
#define NGX_SSL  1
#else
#define NGX_SSL  0
#endif
#endif

#ifndef NGX_OPENSSL
#if defined(CONFIG_LIBSSL)
#define NGX_OPENSSL  1
#else
#define NGX_OPENSSL  0
#endif
#endif

#ifndef NGX_HTTP_SSL
#ifdef CONFIG_LIBNGINX_HTTP_SSL
#define NGX_HTTP_SSL  1
#else
#define NGX_HTTP_SSL  0
#endif
#endif

#ifndef NGX_HTTP_CACHE
#define NGX_HTTP_CACHE  0 /* disabled module */
#endif

#ifndef NGX_HTTP_HEADERS
#define NGX_HTTP_HEADERS  1
#endif

#ifndef NGX_HTTP_GZIP
#ifdef CONFIG_LIBNGINX_HTTP_GZIP
#define NGX_HTTP_GZIP  1
#else
#define NGX_HTTP_GZIP  0
#endif
#endif

#define CONFIG_LIBNGINX_HTTP_SSI
#ifdef CONFIG_LIBNGINX_HTTP_SSI
#ifndef NGX_HTTP_SSI
#define NGX_HTTP_SSI  1
#endif
#endif

#ifdef CONFIG_LIBCRYPTO
#ifndef NGX_CRYPT
#define NGX_CRYPT  1
#endif
#else
#ifndef NGX_CRYPT
#define NGX_CRYPT  0
#endif
#endif

#ifndef NGX_HTTP_X_FORWARDED_FOR
#define NGX_HTTP_X_FORWARDED_FOR  1
#endif

#ifndef NGX_HTTP_X_FORWARDED_FOR
#define NGX_HTTP_X_FORWARDED_FOR  1
#endif

#ifndef NGX_HTTP_UPSTREAM_ZONE
#define NGX_HTTP_UPSTREAM_ZONE  1
#endif

#ifdef CONFIG_LIBPCRE
#ifndef NGX_PCRE
#define NGX_PCRE  1
#endif

#ifndef NGX_HAVE_PCRE_JIT
#define NGX_HAVE_PCRE_JIT  0
#endif
#endif

#ifdef CONFIG_LIBZLIB
#ifndef NGX_ZLIB
#define NGX_ZLIB  1
#endif
#else
#ifndef NGX_ZLIB
#define NGX_ZLIB  0
#endif
#endif

#ifndef NGX_PREFIX
#define NGX_PREFIX  "/nginx/"
#endif

#ifndef NGX_CONF_PREFIX
#define NGX_CONF_PREFIX  "conf/"
#endif

#ifndef NGX_SBIN_PATH
#define NGX_SBIN_PATH  "sbin/nginx"
#endif

#ifndef NGX_CONF_PATH
#define NGX_CONF_PATH  "conf/nginx.conf"
#endif

#ifndef NGX_PID_PATH
#define NGX_PID_PATH  "logs/nginx.pid"
#endif

#ifndef NGX_LOCK_PATH
#define NGX_LOCK_PATH  "logs/nginx.lock"
#endif

#ifndef NGX_ERROR_LOG_PATH
#define NGX_ERROR_LOG_PATH  "logs/error.log"
#endif

#ifndef NGX_HTTP_LOG_PATH
#define NGX_HTTP_LOG_PATH  "logs/access.log"
#endif

#ifndef NGX_HTTP_CLIENT_TEMP_PATH
#define NGX_HTTP_CLIENT_TEMP_PATH  "client_body_temp"
#endif

#ifndef NGX_HTTP_PROXY_TEMP_PATH
#define NGX_HTTP_PROXY_TEMP_PATH  "proxy_temp"
#endif

#ifndef NGX_HTTP_FASTCGI_TEMP_PATH
#define NGX_HTTP_FASTCGI_TEMP_PATH  "fastcgi_temp"
#endif

#ifndef NGX_HTTP_UWSGI_TEMP_PATH
#define NGX_HTTP_UWSGI_TEMP_PATH  "uwsgi_temp"
#endif

#ifndef NGX_HTTP_SCGI_TEMP_PATH
#define NGX_HTTP_SCGI_TEMP_PATH  "scgi_temp"
#endif

#ifndef NGX_SUPPRESS_WARN
#define NGX_SUPPRESS_WARN  1
#endif

#ifndef NGX_SMP
#define NGX_SMP  0
#endif

#ifndef NGX_USER
#define NGX_USER  "root"
#endif

#ifndef NGX_GROUP
#define NGX_GROUP  "root"
#endif

