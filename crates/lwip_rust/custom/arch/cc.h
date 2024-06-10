#ifndef __ARCH_CC_H__
#define __ARCH_CC_H__

#ifndef _SSIZE_T_DEFINED
typedef long ssize_t;
#define _SSIZE_T_DEFINED
#endif

#define LWIP_NO_INTTYPES_H 1
#define U8_F               "hhu"
#define S8_F               "hhd"
#define X8_F               "hhx"
#define U16_F              "hu"
#define S16_F              "hd"
#define X16_F              "hx"
#define U32_F              "u"
#define S32_F              "d"
#define X32_F              "x"
#define SZT_F              "zu"

#define LWIP_NO_LIMITS_H 1
#define LWIP_NO_CTYPE_H  1

#define SSIZE_MAX        INT_MAX
#define LWIP_NO_UNISTD_H 1

#define LWIP_PLATFORM_DIAG(x) \
    do {                      \
    } while (0)

#define LWIP_PLATFORM_ASSERT(x)                                                       \
    do {                                                                              \
    } while (0)

#define LWIP_RAND() (rand())

#endif /* __ARCH_CC_H__ */