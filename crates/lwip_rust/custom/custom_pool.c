/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

#include "custom_pool.h"
#include "lwip/memp.h"
#include "lwip/pbuf.h"
#include "lwip/stats.h"

#define RX_POOL_SIZE 128

LWIP_MEMPOOL_DECLARE(RX_POOL, RX_POOL_SIZE, sizeof(rx_custom_pbuf_t), "Zero-copy RX PBUF pool")

void rx_custom_pbuf_init(void)
{
    LWIP_MEMPOOL_INIT(RX_POOL);
}

struct pbuf *rx_custom_pbuf_alloc(pbuf_free_custom_fn custom_free_function, void *buf, void *dev,
                                  u16_t length, void *payload_mem, u16_t payload_mem_len)
{
    rx_custom_pbuf_t *my_pbuf = (rx_custom_pbuf_t *)LWIP_MEMPOOL_ALLOC(RX_POOL);
    my_pbuf->p.custom_free_function = custom_free_function;
    my_pbuf->buf = buf;
    my_pbuf->dev = dev;
    struct pbuf *p =
        pbuf_alloced_custom(PBUF_RAW, length, PBUF_REF, &my_pbuf->p, payload_mem, payload_mem_len);
    return p;
}

void rx_custom_pbuf_free(rx_custom_pbuf_t *p)
{
    LWIP_MEMPOOL_FREE(RX_POOL, p);
}
