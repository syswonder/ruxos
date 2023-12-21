/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

#ifndef PRINTF_CONFIG_H
#define PRINTF_CONFIG_H

#define PRINTF_ALIAS_STANDARD_FUNCTION_NAMES 1

#ifndef RUX_CONFIG_FP_SIMD

#define PRINTF_SUPPORT_DECIMAL_SPECIFIERS 0

#define PRINTF_SUPPORT_EXPONENTIAL_SPECIFIERS 0

#endif // RUX_CONFIG_FP_SIMD

#endif // PRINTF_CONFIG_H
