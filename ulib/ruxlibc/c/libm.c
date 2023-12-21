/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

#ifdef RUX_CONFIG_FP_SIMD

#include <math.h>

#include "libm.h"

double __math_divzero(uint32_t sign)
{
    return fp_barrier(sign ? -1.0 : 1.0) / 0.0;
}

double __math_invalid(double x)
{
    return (x - x) / (x - x);
}

#endif // RUX_CONFIG_FP_SIMD
