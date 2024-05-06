import subprocess
import time
import psutil
import os

def kill_qemu_processes():
    for proc in psutil.process_iter(['pid', 'name']):
        try:
            process_info = proc.info
            if 'qemu' in process_info['name']:
                subprocess.run(f"kill {process_info['pid']}",shell=True,timeout=30)
        except (psutil.NoSuchProcess, psutil.AccessDenied, psutil.ZombieProcess):
            pass

def eval_libc_bench(arch:str):
    APP_PATH = "apps/c/libc-bench"
    CLEAN_CMD = f"make A={APP_PATH} MUSL=y clean"
    RUN_CMD = f"make A={APP_PATH} ARCH={arch} LOG=warn MUSL=y SMP=4 run"

    subprocess.run(CLEAN_CMD,shell=True,timeout=30)
    bench = subprocess.run(RUN_CMD,shell=True,timeout=120)
    
    return bench.returncode


def eval_redis(arch:str):
    GIT_URL = "https://github.com/syswonder/rux-redis.git"
    APP_PATH = "apps/c/redis"
    ClONE_CMD = f"git clone {GIT_URL} {APP_PATH}"
    BUILD_CMD = f"make A={APP_PATH} MUSL=y LOG=warn NET=y V9P=y BLK=y FEATURES=virtio-9p V9P_PATH=apps/c/redis ARCH={arch} SMP=4 ARGS=\"./redis-server,/v9fs/redis.conf\""
    CLEAN_CMD = f"make A={APP_PATH} MUSL=y clean"
    RUN_CMD = f"make A={APP_PATH} LOG=warn NET=y V9P=y BLK=y FEATURES=virtio-9p V9P_PATH=apps/c/redis ARCH={arch} SMP=4 ARGS=\"./redis-server,/v9fs/redis.conf\" run"
    BENCHMARK_SET_CMD = "redis-benchmark -p 5555 -n 100000 -q -t set -c 30"
    BENCHMARK_GET_CMD = "redis-benchmark -p 5555 -n 100000 -q -t get -c 30"

    subprocess.run(ClONE_CMD,shell=True,timeout=60)
    subprocess.run(CLEAN_CMD,shell=True,timeout=60)
    subprocess.run(BUILD_CMD,shell=True,timeout=60)
    subprocess.run("make disk_img",shell=True,timeout=60)

    redis_server = subprocess.Popen(RUN_CMD, shell=True)
    time.sleep(20)
    
    set_client = subprocess.run(BENCHMARK_SET_CMD,shell=True,timeout=60)
    get_client = subprocess.run(BENCHMARK_GET_CMD,shell=True,timeout=60)
    
    redis_server.terminate()
    redis_server.wait(timeout=60)
    
    return set_client.returncode | get_client.returncode
    
def eval_wamr(arch:str):
    GIT_URL = "https://github.com/syswonder/rux-wamr.git"
    APP_PATH = "apps/c/wamr"
    ClONE_CMD = f"git clone {GIT_URL} {APP_PATH}"
    CLEAN_CMD = f"make A={APP_PATH} ARCH={arch} MUSL=y clean"
    RUN_CMD = f"make A={APP_PATH} ARCH={arch} LOG=warn SMP=4 MUSL=y NET=y V9P=y V9P_PATH=apps/c/wamr/rootfs ARGS=\"iwasm,/main.wasm\" run"

    subprocess.run(ClONE_CMD,shell=True,timeout=30)
    subprocess.run(CLEAN_CMD,shell=True,timeout=30)
    subprocess.run("make disk_img",shell=True,timeout=30)      
    
    wamr_server = subprocess.Popen(RUN_CMD, shell=True) 
    wamr_server.wait(timeout=60)
    
    return wamr_server.returncode


html_setup = False
def eval_nginx(arch:str):
    GIT_URL = "https://github.com/syswonder/rux-nginx.git"
    APP_PATH = "apps/c/nginx"
    CLEAN_CMD = f"make A={APP_PATH} LOG=warn NET=y BLK=y ARCH={arch} SMP=4 MUSL=y clean"
    ClONE_CMD = f"git clone {GIT_URL} {APP_PATH}"
    RUN_CMD = f"make A={APP_PATH} LOG=warn NET=y BLK=y ARCH={arch} SMP=4 MUSL=y run"
    TEST_CMD = f"wget 127.0.0.1:5555"

    global html_setup
    if False==html_setup:
        subprocess.run(ClONE_CMD,shell=True,timeout=30)
        subprocess.run("git clone https://github.com/syswonder/syswonder-web.git",shell=True)
        subprocess.run("mkdir -p apps/c/nginx/html",shell=True)
        subprocess.run("cp -r syswonder-web/docs/* apps/c/nginx/html",shell=True)
        subprocess.run("rm -f -r syswonder-web",shell=True)
        html_setup = True
        
    subprocess.run(CLEAN_CMD,shell=True,timeout=30)

    ngx_server = subprocess.Popen(RUN_CMD, shell=True)
    time.sleep(30)
    
    test = subprocess.run(TEST_CMD,shell=True,timeout=30)
    
    ngx_server.terminate()
    ngx_server.wait()
    
    return test.returncode

if __name__ == "__main__":  
    for target in ["aarch64"]:
        kill_qemu_processes()
        status = eval_libc_bench(arch=target)
        if status != 0 :
            raise Exception(f"failed when eval_libc_bench() for {target} errcode={status}")

        kill_qemu_processes()
        status = eval_nginx(arch=target)
        if status != 0 :
            raise Exception(f"failed when eval_nginx() for {target} errcode={status}")
        
        kill_qemu_processes()
        status = eval_redis(arch=target)
        if status != 0 :
            raise Exception(f"failed when eval_redis() for {target} errcode={status}")
           
        kill_qemu_processes()
        status = eval_wamr(arch=target)
        if status != 0 :
            raise Exception(f"failed when eval_wamr() for {target} errcode={status}")
    
        kill_qemu_processes()