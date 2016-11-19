// Copyright 2016 Walter Kuppens.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[derive(PartialEq)]
pub enum PageCross {
    Same,
    Backwards,
    Forwards,
}

/// Returns the page index of the given address. Each memory page for the
/// 6502 is 256 (FF) bytes in size and is relevant because some instructions
/// need extra cycles to use addresses in different pages.
#[inline(always)]
pub fn page(addr: usize) -> u8 {
    (addr as u16 >> 8) as u8
}

/// Determine if there was a page cross between the addresses and what
/// direction was crossed. Most instructions don't care which direction the
/// page cross was in so those instructions will check for either forwards
/// or backwards.
#[inline(always)]
pub fn page_cross(addr1: usize, addr2: usize) -> PageCross {
    let page1 = page(addr1);
    let page2 = page(addr2);

    if page1 > page2 {
        PageCross::Backwards
    } else if page1 < page2 {
        PageCross::Forwards
    } else {
        PageCross::Same
    }
}
