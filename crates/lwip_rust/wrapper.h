/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

#include "depend/lwip/src/include/lwip/def.h"
#include "depend/lwip/src/include/lwip/dns.h"
#include "depend/lwip/src/include/lwip/etharp.h"
#include "depend/lwip/src/include/lwip/init.h"
#include "depend/lwip/src/include/lwip/ip4_addr.h"
#include "depend/lwip/src/include/lwip/ip_addr.h"
#include "depend/lwip/src/include/lwip/netif.h"
#include "depend/lwip/src/include/lwip/tcp.h"
#include "depend/lwip/src/include/lwip/timeouts.h"
#include "depend/lwip/src/include/lwip/udp.h"
#include "depend/lwip/src/include/netif/ethernet.h"

#include "custom/custom_pool.h"
