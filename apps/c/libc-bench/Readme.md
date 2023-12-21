# Description

- This is [libc-bench](https://git.musl-libc.org/cgit/libc-bench/) test. 

- Run with following command
```
make A=apps/c/libc-bench/ MUSL=y BLK=y ARCH=aarch64 LOG=error SMP=4 run
```

- This benchmark includes (all codes are really simple to read):
  - `malloc.c`: some memory tests for big/small malloc
  - `pthread.c`: tests for pthread library
  - `regex.c`
  - `stdio.c`: write/read to/from `tmpfile()`
  - `string.c`: some string tests
  - `uft8.c`: local related tests
  - `main.c` -> `test.c`: main.c is renamed to test.c when patching. It contains the main function for all tests.

# Expected Output

- By running command above, output is expected to be like this:
```
       d8888                            .d88888b.   .d8888b.
      d88888                           d88P" "Y88b d88P  Y88b
     d88P888                           888     888 Y88b.
    d88P 888 888d888  .d8888b  .d88b.  888     888  "Y888b.
   d88P  888 888P"   d88P"    d8P  Y8b 888     888     "Y88b.
  d88P   888 888     888      88888888 888     888       "888
 d8888888888 888     Y88b.    Y8b.     Y88b. .d88P Y88b  d88P
d88P     888 888      "Y8888P  "Y8888   "Y88888P"   "Y8888P"

arch = aarch64
platform = aarch64-qemu-virt
target = aarch64-unknown-none-softfloat
smp = 1
build_mode = release
log_level = error

[  0.013618 0 fatfs::dir:140] Is a directory
[  0.017966 0 fatfs::dir:140] Is a directory
[  0.022904 0 fatfs::dir:140] Is a directory
[  0.027088 0 fatfs::dir:140] Is a directory
b_malloc_sparse (0)
  time: 0.041970112

b_malloc_bubble (0)
  time: 0.035406064

b_malloc_tiny1 (0)
  time: 0.005255936

b_malloc_tiny2 (0)
  time: 0.003587440

b_malloc_big1 (0)
  time: 0.044762944

b_malloc_big2 (0)
  time: 0.035574048

b_malloc_thread_stress (0)
  time: 0.088184912

b_malloc_thread_local (0)
  time: 0.084185664

b_string_strstr ("abcdefghijklmnopqrstuvwxyz")
  time: 0.017637792

b_string_strstr ("azbycxdwevfugthsirjqkplomn")
  time: 0.026718000

b_string_strstr ("aaaaaaaaaaaaaacccccccccccc")
  time: 0.017456832

b_string_strstr ("aaaaaaaaaaaaaaaaaaaaaaaaac")
  time: 0.017303904

b_string_strstr ("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaac")
  time: 0.021478544

b_string_memset (0)
  time: 0.006125200

b_string_strchr (0)
  time: 0.016433616

b_string_strlen (0)
  time: 0.012831152

b_pthread_createjoin_serial1 (0)
  time: 0.185955808

b_pthread_createjoin_serial2 (0)
  time: 0.223816784

b_pthread_create_serial1 (0)
  time: 0.105454736

b_pthread_uselesslock (0)
  time: 0.112944368

b_pthread_createjoin_minimal1 (0)
  time: 0.549677808

b_pthread_createjoin_minimal2 (0)
  time: 0.225489504

b_utf8_bigbuf (0)
  time: 0.056279344

b_utf8_onebyone (0)
  time: 0.127939568

b_stdio_putcgetc (0)
[  2.098431 0:2 ruxos_posix_api::imp::ioctl:32] Only support fd = 1
  time: 0.395968752

b_stdio_putcgetc_unlocked (0)
[  2.493650 0:2 ruxos_posix_api::imp::ioctl:32] Only support fd = 1
  time: 0.245682432

b_regex_compile ("(a|b|c)*d*b")
  time: 0.093769280

b_regex_search ("(a|b|c)*d*b")
  time: 0.078445200

b_regex_search ("a{25}b")
  time: 0.210470176
```

# Bugs

- Cannot run `b_pthread_createjoin_minimal2` individually, which caused weird page fault.
  - how to replay this? Just comment out other `RUN(xxxx)` lines.

- Logs will affect result.
  - Though first bug exists, there are some tricks to pass `b_pthread_createjoin_minimal2` individually. When you try to debug first bug, you may add some logs, and you will find out this weird bug.

- Cannot run with `SMP=4`.

# Other Notifications

- Allignment for `TaskStack`, `TlsArea` has been changed to 8B, but it doesn't work.

- These bugs are more like memory bugs. Try to think with memory layout.

- For SMP bug, something seems to be wrong for tls (should checkout tls implementation, and pay attention to `tpidr_el0/tpidr_el1`). By running with `SMP=4`, it seems that this benchmark fails when reading `tpidr_el1`(a weird value).

- Memory is expanded to 4G (two files are changed, `platforms/aarch64-qemu-virt.toml`, `scripts/make/qemu.mk`), so CI fails (mostly because RISC-V cannot support such a large memory space).

- It is recommended to know how musl-libc is integrated in `ulib/ruxmusl`.

- Since `/etc/hosts` is not complemented, `getaddrinfo` uses previous implementation. See `ulib/ruxmusl/src/net.rs`.
