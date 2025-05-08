#ifndef __LWIPOPTS_H__
#define __LWIPOPTS_H__

#define NO_SYS 1

/*
   ------------------------------------
   ------------ Functions  ------------
   ------------------------------------
*/
#define IP_DEFAULT_TTL       64
#define LWIP_ETHERNET        1
#define LWIP_ARP             1
#define ARP_QUEUEING         0
#define IP_FORWARD           0
#define LWIP_ICMP            1
#define LWIP_RAW             0
#define LWIP_DHCP            0
#define LWIP_AUTOIP          0
#define LWIP_SNMP            0
#define LWIP_IGMP            0
#define LWIP_DNS             1
#define LWIP_UDP             1
#define LWIP_UDPLITE         0
#define LWIP_TCP             1
#define LWIP_CALLBACK_API    1
#define LWIP_NETIF_API       0
#define LWIP_NETIF_LOOPBACK  1
#define LWIP_HAVE_LOOPIF     0
#define LWIP_HAVE_SLIPIF     0
#define LWIP_NETCONN         0
#define LWIP_SOCKET          0
#define PPP_SUPPORT          0
#define LWIP_IPV4            1

// Enable SO_REUSEADDR
#define SO_REUSE 1

/*
   ------------------------------------
   ------ Memory and Performance ------
   ------------------------------------
*/

// Important performance options
// Smaller values increase performance
// Larger values increase simultaneously active TCP connections limit
#define MEMP_NUM_TCP_PCB 5

// Memory options
#define MEM_SIZE         (1 * 1024 * 1024)
#define MEMP_NUM_TCP_SEG 128
#define MEMP_NUM_PBUF    32
#define PBUF_POOL_SIZE   32

// Tcp options
#define TCP_MSS     1460
#define TCP_WND     (32 * TCP_MSS)
#define TCP_SND_BUF (16 * TCP_MSS)

// Disable checksum checks
#define CHECKSUM_CHECK_IP    0
#define CHECKSUM_CHECK_UDP   0
#define CHECKSUM_CHECK_TCP   0
#define CHECKSUM_CHECK_ICMP  0
#define CHECKSUM_CHECK_ICMP6 0

// Other performance options
#define LWIP_CHECKSUM_ON_COPY 1
#define SYS_LIGHTWEIGHT_PROT  0

/*
   ------------------------------------
   ---------- Debug options ----------
   ------------------------------------
*/

#define LWIP_DEBUG         0
#define LWIP_DBG_TYPES_ON  LWIP_DBG_OFF
#define LWIP_DBG_MIN_LEVEL LWIP_DBG_LEVEL_ALL

#define ETHARP_DEBUG     LWIP_DBG_OFF
#define NETIF_DEBUG      LWIP_DBG_ON
#define PBUF_DEBUG       LWIP_DBG_OFF
#define API_LIB_DEBUG    LWIP_DBG_ON
#define API_MSG_DEBUG    LWIP_DBG_ON
#define SOCKETS_DEBUG    LWIP_DBG_ON
#define ICMP_DEBUG       LWIP_DBG_ON
#define IGMP_DEBUG       LWIP_DBG_ON
#define INET_DEBUG       LWIP_DBG_ON
#define IP_DEBUG         LWIP_DBG_ON
#define IP_REASS_DEBUG   LWIP_DBG_ON
#define RAW_DEBUG        LWIP_DBG_ON
#define MEM_DEBUG        LWIP_DBG_ON
#define MEMP_DEBUG       LWIP_DBG_ON
#define SYS_DEBUG        LWIP_DBG_ON
#define TIMERS_DEBUG     LWIP_DBG_OFF
#define TCP_DEBUG        LWIP_DBG_ON
#define TCP_INPUT_DEBUG  LWIP_DBG_ON
#define TCP_FR_DEBUG     LWIP_DBG_ON
#define TCP_RTO_DEBUG    LWIP_DBG_ON
#define TCP_CWND_DEBUG   LWIP_DBG_ON
#define TCP_WND_DEBUG    LWIP_DBG_ON
#define TCP_OUTPUT_DEBUG LWIP_DBG_ON
#define TCP_RST_DEBUG    LWIP_DBG_ON
#define TCP_QLEN_DEBUG   LWIP_DBG_ON
#define UDP_DEBUG        LWIP_DBG_ON
#define TCPIP_DEBUG      LWIP_DBG_ON
#define SLIP_DEBUG       LWIP_DBG_ON
#define DHCP_DEBUG       LWIP_DBG_ON
#define AUTOIP_DEBUG     LWIP_DBG_ON
#define ACD_DEBUG        LWIP_DBG_ON
#define DNS_DEBUG        LWIP_DBG_ON

#define LWIP_STATS         0
#define LWIP_STATS_DISPLAY 0
#define LWIP_PERF          0

/*
   ------------------------------------
   ----------- Memory check -----------
   ------------------------------------
*/
#define MEMP_OVERFLOW_CHECK 0
#define MEMP_SANITY_CHECK   0
#define MEM_OVERFLOW_CHECK  0
#define MEM_SANITY_CHECK    0

#endif /* __LWIPOPTS_H__ */