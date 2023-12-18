# How to run nginx on ruxos

## commands

### download web page

You should make sure there is a html folder in apps/c/nginx which contains the web page of nginx server.

If you not use your own web page , you can run commands below:

```shell
git clone https://github.com/syswonder/syswonder-web.git
mkdir -p apps/c/nginx/html
cp -r syswonder-web/docs/* apps/c/nginx/html
rm -f -r syswonder-web
```

### run nginx

The commands below is to run nginx with different features.  These examples run in aarch64 with musl, if you want to run in x86_64, just replace `ARCH=aarch64` with `ARCH=x86_64`, and if you do not want to run with musl , just delete `MUSL=y`.

use v9p and musl in aarch64：

```shell
make A=apps/c/nginx/ LOG=info NET=y BLK=y V9P=y V9P_PATH=apps/c/nginx/html/  ARCH=aarch64 SMP=4 ARGS="./nginx_app" MUSL=y run
```

not use v9p，but use musl in aarch64：

```shell
make A=apps/c/nginx/ LOG=info NET=y BLK=y ARCH=aarch64 SMP=4 ARGS="./nginx_app" MUSL=y run
```

If you change running option or source code , remember to clean the compile files and before running.

```shell
make clean_c A=apps/c/nginx
```

# nginx conf

You can change next files to change nginx conf:

`/nginx/conf/nginx.conf`

`/nginx/conf/mime.types`

After change you should copy them to disk.img (you can run `apps/c/nginx/create_nginx_img.sh` to do that)

