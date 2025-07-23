/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

#ifndef __CUSTOM_POOL_H__
#define __CUSTOM_POOL_H__

#include "lwip/pbuf.h"

typedef struct rx_custom_pbuf_t {
    struct pbuf_custom p;
    void *buf;
    void *dev;
} rx_custom_pbuf_t;

void rx_custom_pbuf_init(void);
struct pbuf *rx_custom_pbuf_alloc(pbuf_free_custom_fn custom_free_function, void *buf, void *dev,
                                  u16_t length, void *payload_mem, u16_t payload_mem_len);
void rx_custom_pbuf_free(rx_custom_pbuf_t *p);

#endif /* __CUSTOM_POOL_H__ */