import subprocess
import time
import psutil
import sys

# Prerequisites:
# 1. Install QEMU and its dependencies
#
# sudo apt update && sudo apt install -y redis-tools
#
# 2. Install psutil for Python3
#
# sudo apt update && sudo apt install python3-pip && pip install psutil
#
# Usage:
# python3 app_eval.py <arch>
# <arch> can be "x86_64" or "aarch64"
#
# Example:
# python3 app_eval.py x86_64


def kill_qemu_processes():
    for proc in psutil.process_iter(['pid', 'name']):
        try:
            process_info = proc.info
            if 'qemu' in process_info['name']:
                subprocess.run(f"kill {process_info['pid']}",
                               shell=True,
                               timeout=30)
        except (psutil.NoSuchProcess, psutil.AccessDenied,
                psutil.ZombieProcess):
            pass


def check_output_contains(output: str, expect_path: str) -> bool:
    with open(expect_path) as f:
        for s in f:
            if not s in output:
                print(output)
                return False
    return True


def eval_libc_bench(arch: str):
    print("eval_libc_bench() is testing...")
    APP_PATH = "apps/c/libc-bench"
    CLEAN_CMD = f"make A={APP_PATH} MUSL=y clean"
    RUN_CMD = f"make A={APP_PATH} ARCH={arch} LOG=warn MUSL=y SMP=4 ACCEL=n run"

    subprocess.run(CLEAN_CMD,
                   shell=True,
                   capture_output=True,
                   text=True,
                   timeout=60)
    bench = subprocess.run(RUN_CMD,
                           shell=True,
                           capture_output=True,
                           text=True,
                           timeout=120)

    if check_output_contains(bench.stdout,
                             f"{APP_PATH}/expect_warn.out") == False:
        print(bench.stderr)
        return 1  # failed

    time.sleep(3)
    return bench.returncode


def eval_nginx(arch: str):
    print("eval_nginx() is testing...")
    GIT_URL = "https://github.com/syswonder/rux-nginx.git"
    APP_PATH = "apps/c/nginx"
    CLEAN_CMD = f"make A={APP_PATH} LOG=warn NET=y BLK=y ARCH={arch} SMP=4 MUSL=y clean"
    ClONE_CMD = f"git clone {GIT_URL} {APP_PATH}"
    BUILD_CMD = f"make A={APP_PATH} LOG=warn NET=y BLK=y ARCH={arch} SMP=4 MUSL=y build"
    RUN_CMD = f"make A={APP_PATH} LOG=warn NET=y BLK=y ARCH={arch} SMP=4 MUSL=y ACCEL=n run"
    TEST_CMD = f"wget localhost:5555 --tries=5 "
    RM_CMD = f"rm index.html"

    subprocess.run(ClONE_CMD,
                   shell=True,
                   capture_output=True,
                   text=True,
                   timeout=60)
    subprocess.run("git clone https://github.com/syswonder/syswonder-web.git",
                   capture_output=True,
                   text=True,
                   timeout=60,
                   shell=True)
    subprocess.run("mkdir -p apps/c/nginx/html",
                   capture_output=True,
                   text=True,
                   timeout=60,
                   shell=True)
    subprocess.run("cp -r syswonder-web/docs/* apps/c/nginx/html",
                   capture_output=True,
                   text=True,
                   timeout=60,
                   shell=True)
    subprocess.run("rm -f -r syswonder-web",
                   capture_output=True,
                   text=True,
                   timeout=60,
                   shell=True)

    subprocess.run(CLEAN_CMD,
                   shell=True,
                   capture_output=True,
                   text=True,
                   timeout=30)

    subprocess.run(BUILD_CMD,
                   shell=True,
                   capture_output=True,
                   text=True,
                   timeout=120)

    ngx_server = subprocess.Popen(
        RUN_CMD,
        stdout=subprocess.PIPE,
        # stderr=subprocess.PIPE,
        text=True,
        shell=True)
    time.sleep(30)

    test = subprocess.run(TEST_CMD,
                          shell=True,
                          capture_output=True,
                          text=True,
                          timeout=60)

    ngx_server.terminate()
    ngx_server.wait(timeout=60)
    time.sleep(3)

    # Saving to: ‘index.html’
    # 2024-06-25 16:42:03 (776 MB/s) - ‘index.html’ saved [3159/3159]
    if not "saved" in test.stderr:
        print(test.stdout)
        print(test.stderr)
        return 1  # failed

    test = subprocess.run(RM_CMD,
                          shell=True,
                          capture_output=True,
                          text=True,
                          timeout=10)

    return test.returncode


def eval_redis(arch: str):
    print("eval_redis() is testing...")
    GIT_URL = "https://github.com/syswonder/rux-redis.git"
    APP_PATH = "apps/c/redis"
    ClONE_CMD = f"git clone {GIT_URL} {APP_PATH}"
    BUILD_CMD = f"make A={APP_PATH} MUSL=y LOG=warn NET=y V9P=y BLK=y FEATURES=virtio-9p V9P_PATH=apps/c/redis ARCH={arch} SMP=4 ARGS=\"./redis-server,/v9fs/redis.conf\""
    CLEAN_CMD = f"make A={APP_PATH} MUSL=y clean"
    RUN_CMD = f"make A={APP_PATH} LOG=warn NET=y MUSL=y V9P=y BLK=y FEATURES=virtio-9p V9P_PATH=apps/c/redis ARCH={arch} SMP=4 ARGS=\"./redis-server,/v9fs/redis.conf\" ACCEL=n run"
    BENCHMARK_SET_CMD = "redis-benchmark -h localhost -p 5555 -n 100000 -q -t set -c 30"
    BENCHMARK_GET_CMD = "redis-benchmark -h localhost -p 5555 -n 100000 -q -t get -c 30"

    subprocess.run(ClONE_CMD,
                   shell=True,
                   capture_output=True,
                   text=True,
                   timeout=30)
    subprocess.run(CLEAN_CMD,
                   shell=True,
                   capture_output=True,
                   text=True,
                   timeout=60)
    subprocess.run("make disk_img",
                   shell=True,
                   capture_output=True,
                   text=True,
                   timeout=60)
    subprocess.run(BUILD_CMD,
                   shell=True,
                   capture_output=True,
                   text=True,
                   timeout=120)

    redis_server = subprocess.Popen(
        RUN_CMD,
        shell=True,
        stdout=subprocess.PIPE,
        # stderr=subprocess.PIPE,
        text=True)
    time.sleep(10)

    set_client = subprocess.run(BENCHMARK_SET_CMD,
                                shell=True,
                                timeout=60,
                                capture_output=True,
                                text=True)
    get_client = subprocess.run(BENCHMARK_GET_CMD,
                                shell=True,
                                timeout=60,
                                capture_output=True,
                                text=True)

    redis_server.terminate()
    redis_server.wait(timeout=30)
    time.sleep(3)

    print(set_client.stdout)
    print(get_client.stdout)
    if not ("SET" in set_client.stdout and "GET" in get_client.stdout):
        print(set_client.stderr)
        print(get_client.stderr)
        return 1  # failed

    return set_client.returncode | get_client.returncode


def eval_wamr(arch: str):
    print("eval_wamr() is testing...")
    GIT_URL = "https://github.com/syswonder/rux-wamr.git"
    APP_PATH = "apps/c/wamr"
    ClONE_CMD = f"git clone {GIT_URL} {APP_PATH}"
    CLEAN_CMD = f"make A={APP_PATH} ARCH={arch} MUSL=y clean"
    RUN_CMD = f"make A={APP_PATH} ARCH={arch} LOG=warn SMP=4 MUSL=y NET=y V9P=y V9P_PATH=apps/c/wamr/rootfs ARGS=\"iwasm,/main.wasm\" ACCEL=n run"

    subprocess.run(ClONE_CMD,
                   shell=True,
                   capture_output=True,
                   text=True,
                   timeout=60)
    subprocess.run(CLEAN_CMD,
                   shell=True,
                   capture_output=True,
                   text=True,
                   timeout=60)
    subprocess.run("make disk_img",
                   shell=True,
                   capture_output=True,
                   text=True,
                   timeout=60)

    wamr_server = subprocess.run(RUN_CMD,
                                 shell=True,
                                 capture_output=True,
                                 text=True,
                                 timeout=120)

    if not "Hello world!" in wamr_server.stdout:
        print(wamr_server.stdout)
        print(wamr_server.stderr)
        return 1  # failed

    return wamr_server.returncode


# Usage: python3 app_eval.py <arch>
if __name__ == "__main__":
    target = sys.argv[1]

    kill_qemu_processes()
    status = eval_libc_bench(arch=target)
    if status != 0:
        print(f"failed when eval_libc_bench() for {target} errcode={status}")
        sys.exit(status)

    kill_qemu_processes()
    status = eval_nginx(arch=target)
    if status != 0:
        print(f"failed when eval_nginx() for {target} errcode={status}")
        sys.exit(status)

    kill_qemu_processes()
    status = eval_redis(arch=target)
    if status != 0:
        print(f"failed when eval_redis() for {target} errcode={status}")
        sys.exit(status)

    kill_qemu_processes()
    status = eval_wamr(arch=target)
    if status != 0:
        print(f"failed when eval_wamr() for {target} errcode={status}")
        sys.exit(status)

    kill_qemu_processes()
    print(f"all tests for {target} passed")
    sys.exit(0)
