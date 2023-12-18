# 运行方法

## 运行命令

### 下载网页

```shell
//首先要确保机器中有musl libc,可以运行Ruxos的c程序，具体可以参考Ruxos的README中运行c程序的部分
git clone https://github.com/syswonder/syswonder-web.git
mkdir -p apps/c/nginx/html
cp -r syswonder-web/docs/* apps/c/nginx/html
rm -f -r syswonder-web
```

当运行完成时，访问本机ip的5555端口即可看到结果

### 运行

使用v9p以及musl运行：

```shell
make A=apps/c/nginx/ LOG=info NET=y BLK=y V9P=y V9P_PATH=apps/c/nginx/html/  ARCH=aarch64 SMP=4 ARGS="./nginx_app" MUSL=y run
```

使用v9p,不使用musl而使用axlibc运行：

```shell
make A=apps/c/nginx/ LOG=info NET=y BLK=y V9P=y V9P_PATH=apps/c/nginx/html/  ARCH=aarch64 SMP=4 ARGS="./nginx_app" run
```

不使用v9p，使用musl运行：

```shell
make A=apps/c/nginx/ LOG=info NET=y BLK=y ARCH=aarch64 SMP=4 ARGS="./nginx_app" MUSL=y run
```

v9p以及musl都不使用,使用axlibc运行：

```shell
make A=apps/c/nginx/ LOG=info NET=y BLK=y  ARCH=aarch64 SMP=4 ARGS="./nginx_app" run
```

注意，如果再次运行时有所改动（比如修改了nginx源码或者将libc从musl变为axlibc)需要进行一次clean

```shell
make clean_c A=apps/c/nginx
```

# 运行方法的解释

## 运行要求：

机器上需要有以下文件

`/nginx/logs/error.log`

`/nginx/conf/nginx.conf`

`/nginx/conf/mime.types`

其中，error.log是日志文件（但是实际上没有用到），nginx.conf是nginx配置文件，告诉nginx如何运行以及一些运行的参数。mime.type是类型转化文件，告诉nginx如何看待不同类型的文件。

在apps/c/nginx文件中，可以运行create_nginx_img，创建含有上述文件的磁盘

## nginx.conf

需要着重设置的是nginx.conf，目前运行的syswonder-web，其具体内容如下：

```nginx
#user  nobody;
worker_processes  1;
daemon off;
master_process off;

#error_log  logs/error.log;
#error_log  logs/error.log  notice;
error_log  logs/error.log debug;

#pid        logs/nginx.pid;


events {
    worker_connections  32;
}


http {
    include       mime.types;
    default_type  application/octet-stream;

    #log_format  main  '$remote_addr - $remote_user [$time_local] "$request" '
    #                  '$status $body_bytes_sent "$http_referer" '
    #                  '"$http_user_agent" "$http_x_forwarded_for"';

    #access_log  logs/access.log  main;

    #sendfile        on;
    #tcp_nopush     on;

    #keepalive_timeout  0;
    keepalive_timeout  65;

    #gzip  on;

    server {
        listen       5555;
        server_name  localhost;

        #charset koi8-r;

        #access_log  logs/host.access.log  main;

        index index.html;
    
        root /v9fs;

        location / {
            try_files $uri $uri/ /404.html;
        }

        error_page 404 /404.html;
        location = /404.html {
            root /v9fs;
        }

        # redirect server error pages to the static page /50x.html
        #
        error_page   500 502 503 504  /50x.html;
        location = /50x.html {
            root   /v9fs;
        }

    }

}
```

上面的设置会在本机的5555端口建立一个服务器，向请求者发送Index.heml文件

其中需要注意的点是：1.ruxos是单进程系统，无法分出第二个进程，需要使用`daemon off;`设置将守护进程关闭。2.server的文件在机器的/v9fs/下，这是由`root /v9fs;`这一项设置的，可以根据需求设置其他的路径。3. `try_files $uri $uri/ /404.html;`的意思是尝试请求的uri是否存在，不存在的话返回404页面。

如果想要使用nginx的其他用法，需要对应修改nginx.conf文件，具体语法可以查阅官方文档。但是目前还没有尝试过除了http服务器之外的用法。

## 网页文件：

需要确保nginx.conf中设置的网页文件的位置正确，比如在上面的conf中，需要保证文件/v9fs/index.html存在才能访问到主页。其余的文件只需要按照index.html的内容在本文件夹下放到应有的位置。

运行前可以将网页复制到apps/c/nginx/html/文件夹下，并将主目录下的disk.img删除，这样程序在运行时会自动新建一个磁盘并将html文件夹下的文件复制到磁盘中

## 运行命令：

```shell
make A=apps/c/nginx/ LOG=info NET=y BLK=y V9P=y V9P_PATH=/home/oslab/Desktop/Ruxos/apps/c/nginx/html/  ARCH=aarch64 SMP=4 ARGS="./nginx_app" MUSL=y run
```

需要注意的是，nginx.conf中对外的端口是系统的端口，不是qemu以及宿主机的端口，需要对qemu以及宿主机进行相应的设置，使得对应的端口（上面的例子是5555端口）暴露在外面，才能被访问。

## 运行结果：

通过机器的ip地址访问5555端口可以得到syswonder网页

![res](res.png)

## app源码

目前是使用nginx-1.24.0修改后的源码，修改的内容在apps/c/nginx/nginx.patch中