/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

pub use ruxdisplay::DisplayInfo as AxDisplayInfo;

/// Gets the framebuffer information.
pub fn ax_framebuffer_info() -> AxDisplayInfo {
    ruxdisplay::framebuffer_info()
}

/// Flushes the framebuffer, i.e. show on the screen.
pub fn ax_framebuffer_flush() {
    ruxdisplay::framebuffer_flush()
}
