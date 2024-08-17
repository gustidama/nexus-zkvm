// Copyright 2023 RISC Zero, Inc.
// Copyright 2024 Nexus Laboratories, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

/// Minimum gap between heap and stack to avoid clashing.
const MEMORY_GAP: usize = 0x1000;

#[allow(clippy::missing_safety_doc)]
#[no_mangle]
pub unsafe extern "C" fn sys_alloc_aligned(bytes: usize, align: usize) -> *mut u8 {
    extern "C" {
        /// Symbol indicating the end of the program's data segment.
        static _end: u8;
    }

    /// Pointer to the next heap address to use, or 0 if the heap has not yet been initialized.
    static mut HEAP_POS: usize = 0;

    // SAFETY: This code is safe under single-threaded execution, as no other thread can modify `HEAP_POS`.
    let mut heap_pos = HEAP_POS;

    // Initialize heap position if it hasn't been initialized.
    if heap_pos == 0 {
        heap_pos = &_end as *const u8 as usize;
    }

    // Align the heap position to the specified alignment.
    let offset = heap_pos & (align - 1);
    if offset != 0 {
        heap_pos = heap_pos
            .checked_add(align - offset)
            .expect("Heap calculation has overflowed");
    }

    let ptr = heap_pos as *mut u8;
    heap_pos = heap_pos
        .checked_add(bytes)
        .expect("Heap calculation has overflowed");

    // Get the current stack pointer.
    let stack_ptr: usize;
    core::arch::asm!(
        "mv {}, sp",
        out(reg) stack_ptr
    );

    // Check if the heap is about to clash with the stack.
    let gap_check = heap_pos
        .checked_add(MEMORY_GAP)
        .expect("Heap calculation has overflowed");
    if gap_check > stack_ptr {
        panic!(
            "Heap clashing with stack (heap: 0x{:x}, stack: 0x{:x})",
            heap_pos, stack_ptr
        );
    }

    // Update the heap position and return the pointer to the allocated memory.
    HEAP_POS = heap_pos;
    ptr
}
