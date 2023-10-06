
default:	build

all_clean:
	find . -type f \( -name "*.o" -o -name "*.elf" -o -name "*.bin" \) -exec rm -f {} +

clean:
	rm -f objs/nginx_app.o && find . -type f \( -name "*.bin" -o -name "*.elf" \) -exec rm -f {} +

.PHONY:	default clean

$(info $(MAKE) -f objs/Makefile)

build:
	$(MAKE) -f objs/Makefile

install:
	$(MAKE) -f objs/Makefile install

modules:
	$(MAKE) -f objs/Makefile modules

upgrade:
	/nginx/sbin/nginx -t

	kill -USR2 `cat /nginx/logs/nginx.pid`
	sleep 1
	test -f /nginx/logs/nginx.pid.oldbin

	kill -QUIT `cat /nginx/logs/nginx.pid.oldbin`

.PHONY:	build install modules upgrade
