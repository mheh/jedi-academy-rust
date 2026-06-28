/******************************************************************
; *
; * Copyright (c) 1996-1998 ADVANCED MICRO DEVICES, INC.
; * All Rights reserved.
; *
; * This software is unpublished and contains the trade secrets
; * and confidential proprietary information of AMD.  Unless
; * otherwise provided in the Software Agreement associated
; * herewith, it is licensed in confidence "AS IS" and
; * is not to be reproduced in whole or part by any means except
; * for backup.  Use, duplication, or disclosure by the Government
; * is subject to the restrictions in paragraph(b)(3)(B)of the
; * Rights in Technical Data and Computer Software clause in
; * DFAR 52.227-7013(a)(Oct 1988).  Software owned by Advanced
; * Micro Devices Inc., One AMD Place, P.O. Box 3453, Sunnyvale,
; * CA 94088-3453.
; *
; ******************************************************************
 *
 * AMD3D.H
 *
 * MACRO FORMAT
 * ============
 * This file contains inline assembly macros that
 * generate AMD-3D instructions in binary format.
 * Therefore, C or C++ programmer can use AMD-3D instructions
 * without any penalty in their C or C++ source code.
 *
 * The macro's name and format conventions are as follow:
 *
 *
 *      1. First argument of macro is a destination and
 *         second argument is a source operand.
 *              ex) _asm PFCMPEQ (m3, m4)
 *                                        |    |
 *                               dst  src
 *
 *      2. The destination operand can be m0 to m7 only.
 *     The source operand can be any one of the register
 *     m0 to m7 or _eax, _ecx, _edx, _ebx, _esi, or _edi
 *     that contains effective address.
 *      ex) _asm PFRCP    (M7, M6)
 *              ex) _asm PFRCPIT2 (m0, m4)
 *              ex) _asm PFMUL    (m3, _edi)
 *
 *  3. The prefetch(w) takes one src operand _eax, ecx, _edx,
 *     _ebx, _esi, or _edi that contains effective address.
 *      ex) _asm PREFETCH (_edi)
 *
 * EXAMPLE
 * =======
 * Following program doesn't do anything but it shows you
 * how to use inline assembly AMD-3D instructions in C.
 * Note that this will only work in flat memory model which
 * segment registers cs, ds, ss and es point to the same
 * linear address space total less than 4GB.
 *
 * Used Microsoft VC++ 5.0
 *
 * #include <stdio.h>
 * #include "amd3d.h"
 *
 * void main ()
 * {
 *      float x = (float)1.25;
 *      float y = (float)1.25;
 *      float z, zz;
 *
 *      _asm {
 *              movd mm1, x
 *              movd mm2, y
 *              pfmul (m1, m2)
 *              movd z, mm1
 *               femms
 *      }
 *
 *      printf ("value of z = %f\n", z);
 *
 *      //
 *      // Demonstration of using the memory instead of
 *      // multimedia register
 *      //
 *      _asm {
 *              movd mm3, x
 *              lea esi, y   // load effective address of y
 *              pfmul (m3, _esi)
 *              movd zz, mm3
 *              femms
 *      }
 *
 *      printf ("value of zz = %f\n", zz);
 *  }
 ******************************************************************/

#![allow(non_upper_case_globals)]

pub const M0: u8 = 0xc0;
pub const M1: u8 = 0xc1;
pub const M2: u8 = 0xc2;
pub const M3: u8 = 0xc3;
pub const M4: u8 = 0xc4;
pub const M5: u8 = 0xc5;
pub const M6: u8 = 0xc6;
pub const M7: u8 = 0xc7;

pub const m0: u8 = 0xc0;
pub const m1: u8 = 0xc1;
pub const m2: u8 = 0xc2;
pub const m3: u8 = 0xc3;
pub const m4: u8 = 0xc4;
pub const m5: u8 = 0xc5;
pub const m6: u8 = 0xc6;
pub const m7: u8 = 0xc7;
pub const _EAX: u8 = 0x00;
pub const _ECX: u8 = 0x01;
pub const _EDX: u8 = 0x02;
pub const _EBX: u8 = 0x03;
pub const _ESI: u8 = 0x06;
pub const _EDI: u8 = 0x07;
pub const _eax: u8 = 0x00;
pub const _ecx: u8 = 0x01;
pub const _edx: u8 = 0x02;
pub const _ebx: u8 = 0x03;
pub const _esi: u8 = 0x06;
pub const _edi: u8 = 0x07;

macro_rules! PF2ID {
    ($dst:expr, $src:expr) => {
        unsafe {
            core::arch::asm!(
                ".byte 0x0f",
                ".byte 0x0f",
                ".byte {0}",
                ".byte 0x1d",
                in(reg_byte) (($dst & 0x3f) << 3) | $src,
            );
        }
    };
}

macro_rules! PFACC {
    ($dst:expr, $src:expr) => {
        unsafe {
            core::arch::asm!(
                ".byte 0x0f",
                ".byte 0x0f",
                ".byte {0}",
                ".byte 0xae",
                in(reg_byte) (($dst & 0x3f) << 3) | $src,
            );
        }
    };
}

macro_rules! PFADD {
    ($dst:expr, $src:expr) => {
        unsafe {
            core::arch::asm!(
                ".byte 0x0f",
                ".byte 0x0f",
                ".byte {0}",
                ".byte 0x9e",
                in(reg_byte) (($dst & 0x3f) << 3) | $src,
            );
        }
    };
}

macro_rules! PFCMPEQ {
    ($dst:expr, $src:expr) => {
        unsafe {
            core::arch::asm!(
                ".byte 0x0f",
                ".byte 0x0f",
                ".byte {0}",
                ".byte 0xb0",
                in(reg_byte) (($dst & 0x3f) << 3) | $src,
            );
        }
    };
}

macro_rules! PFCMPGE {
    ($dst:expr, $src:expr) => {
        unsafe {
            core::arch::asm!(
                ".byte 0x0f",
                ".byte 0x0f",
                ".byte {0}",
                ".byte 0x90",
                in(reg_byte) (($dst & 0x3f) << 3) | $src,
            );
        }
    };
}

macro_rules! PFCMPGT {
    ($dst:expr, $src:expr) => {
        unsafe {
            core::arch::asm!(
                ".byte 0x0f",
                ".byte 0x0f",
                ".byte {0}",
                ".byte 0xa0",
                in(reg_byte) (($dst & 0x3f) << 3) | $src,
            );
        }
    };
}

macro_rules! PFMAX {
    ($dst:expr, $src:expr) => {
        unsafe {
            core::arch::asm!(
                ".byte 0x0f",
                ".byte 0x0f",
                ".byte {0}",
                ".byte 0xa4",
                in(reg_byte) (($dst & 0x3f) << 3) | $src,
            );
        }
    };
}

macro_rules! PFMIN {
    ($dst:expr, $src:expr) => {
        unsafe {
            core::arch::asm!(
                ".byte 0x0f",
                ".byte 0x0f",
                ".byte {0}",
                ".byte 0x94",
                in(reg_byte) (($dst & 0x3f) << 3) | $src,
            );
        }
    };
}

macro_rules! PFMUL {
    ($dst:expr, $src:expr) => {
        unsafe {
            core::arch::asm!(
                ".byte 0x0f",
                ".byte 0x0f",
                ".byte {0}",
                ".byte 0xb4",
                in(reg_byte) (($dst & 0x3f) << 3) | $src,
            );
        }
    };
}

macro_rules! PFRCP {
    ($dst:expr, $src:expr) => {
        unsafe {
            core::arch::asm!(
                ".byte 0x0f",
                ".byte 0x0f",
                ".byte {0}",
                ".byte 0x96",
                in(reg_byte) (($dst & 0x3f) << 3) | $src,
            );
        }
    };
}

macro_rules! PFRCPIT1 {
    ($dst:expr, $src:expr) => {
        unsafe {
            core::arch::asm!(
                ".byte 0x0f",
                ".byte 0x0f",
                ".byte {0}",
                ".byte 0xa6",
                in(reg_byte) (($dst & 0x3f) << 3) | $src,
            );
        }
    };
}

macro_rules! PFRCPIT2 {
    ($dst:expr, $src:expr) => {
        unsafe {
            core::arch::asm!(
                ".byte 0x0f",
                ".byte 0x0f",
                ".byte {0}",
                ".byte 0xb6",
                in(reg_byte) (($dst & 0x3f) << 3) | $src,
            );
        }
    };
}

macro_rules! PFRSQRT {
    ($dst:expr, $src:expr) => {
        unsafe {
            core::arch::asm!(
                ".byte 0x0f",
                ".byte 0x0f",
                ".byte {0}",
                ".byte 0x97",
                in(reg_byte) (($dst & 0x3f) << 3) | $src,
            );
        }
    };
}

macro_rules! PFRSQIT1 {
    ($dst:expr, $src:expr) => {
        unsafe {
            core::arch::asm!(
                ".byte 0x0f",
                ".byte 0x0f",
                ".byte {0}",
                ".byte 0xa7",
                in(reg_byte) (($dst & 0x3f) << 3) | $src,
            );
        }
    };
}

macro_rules! PFSUB {
    ($dst:expr, $src:expr) => {
        unsafe {
            core::arch::asm!(
                ".byte 0x0f",
                ".byte 0x0f",
                ".byte {0}",
                ".byte 0x9a",
                in(reg_byte) (($dst & 0x3f) << 3) | $src,
            );
        }
    };
}

macro_rules! PFSUBR {
    ($dst:expr, $src:expr) => {
        unsafe {
            core::arch::asm!(
                ".byte 0x0f",
                ".byte 0x0f",
                ".byte {0}",
                ".byte 0xaa",
                in(reg_byte) (($dst & 0x3f) << 3) | $src,
            );
        }
    };
}

macro_rules! PI2FD {
    ($dst:expr, $src:expr) => {
        unsafe {
            core::arch::asm!(
                ".byte 0x0f",
                ".byte 0x0f",
                ".byte {0}",
                ".byte 0x0d",
                in(reg_byte) (($dst & 0x3f) << 3) | $src,
            );
        }
    };
}

macro_rules! FEMMS {
    () => {
        unsafe {
            core::arch::asm!(
                ".byte 0x0f",
                ".byte 0x0e",
            );
        }
    };
}

macro_rules! PAVGUSB {
    ($dst:expr, $src:expr) => {
        unsafe {
            core::arch::asm!(
                ".byte 0x0f",
                ".byte 0x0f",
                ".byte {0}",
                ".byte 0xbf",
                in(reg_byte) (($dst & 0x3f) << 3) | $src,
            );
        }
    };
}

macro_rules! PMULHRW {
    ($dst:expr, $src:expr) => {
        unsafe {
            core::arch::asm!(
                ".byte 0x0f",
                ".byte 0x0f",
                ".byte {0}",
                ".byte 0xb7",
                in(reg_byte) (($dst & 0x3f) << 3) | $src,
            );
        }
    };
}

macro_rules! PREFETCH {
    ($src:expr) => {
        unsafe {
            core::arch::asm!(
                ".byte 0x0f",
                ".byte 0x0d",
                ".byte {0}",
                in(reg_byte) 0x00 | $src,
            );
        }
    };
}

macro_rules! PREFETCHW {
    ($src:expr) => {
        unsafe {
            core::arch::asm!(
                ".byte 0x0f",
                ".byte 0x0d",
                ".byte {0}",
                in(reg_byte) 0x08 | $src,
            );
        }
    };
}

//
// Exactly same as above except macro names are all
// lower case latter.
//
macro_rules! pf2id {
    ($dst:expr, $src:expr) => {
        unsafe {
            core::arch::asm!(
                ".byte 0x0f",
                ".byte 0x0f",
                ".byte {0}",
                ".byte 0x1d",
                in(reg_byte) (($dst & 0x3f) << 3) | $src,
            );
        }
    };
}

macro_rules! pfacc {
    ($dst:expr, $src:expr) => {
        unsafe {
            core::arch::asm!(
                ".byte 0x0f",
                ".byte 0x0f",
                ".byte {0}",
                ".byte 0xae",
                in(reg_byte) (($dst & 0x3f) << 3) | $src,
            );
        }
    };
}

macro_rules! pfadd {
    ($dst:expr, $src:expr) => {
        unsafe {
            core::arch::asm!(
                ".byte 0x0f",
                ".byte 0x0f",
                ".byte {0}",
                ".byte 0x9e",
                in(reg_byte) (($dst & 0x3f) << 3) | $src,
            );
        }
    };
}

macro_rules! pfcmpeq {
    ($dst:expr, $src:expr) => {
        unsafe {
            core::arch::asm!(
                ".byte 0x0f",
                ".byte 0x0f",
                ".byte {0}",
                ".byte 0xb0",
                in(reg_byte) (($dst & 0x3f) << 3) | $src,
            );
        }
    };
}

macro_rules! pfcmpge {
    ($dst:expr, $src:expr) => {
        unsafe {
            core::arch::asm!(
                ".byte 0x0f",
                ".byte 0x0f",
                ".byte {0}",
                ".byte 0x90",
                in(reg_byte) (($dst & 0x3f) << 3) | $src,
            );
        }
    };
}

macro_rules! pfcmpgt {
    ($dst:expr, $src:expr) => {
        unsafe {
            core::arch::asm!(
                ".byte 0x0f",
                ".byte 0x0f",
                ".byte {0}",
                ".byte 0xa0",
                in(reg_byte) (($dst & 0x3f) << 3) | $src,
            );
        }
    };
}

macro_rules! pfmax {
    ($dst:expr, $src:expr) => {
        unsafe {
            core::arch::asm!(
                ".byte 0x0f",
                ".byte 0x0f",
                ".byte {0}",
                ".byte 0xa4",
                in(reg_byte) (($dst & 0x3f) << 3) | $src,
            );
        }
    };
}

macro_rules! pfmin {
    ($dst:expr, $src:expr) => {
        unsafe {
            core::arch::asm!(
                ".byte 0x0f",
                ".byte 0x0f",
                ".byte {0}",
                ".byte 0x94",
                in(reg_byte) (($dst & 0x3f) << 3) | $src,
            );
        }
    };
}

macro_rules! pfmul {
    ($dst:expr, $src:expr) => {
        unsafe {
            core::arch::asm!(
                ".byte 0x0f",
                ".byte 0x0f",
                ".byte {0}",
                ".byte 0xb4",
                in(reg_byte) (($dst & 0x3f) << 3) | $src,
            );
        }
    };
}

macro_rules! pfrcp {
    ($dst:expr, $src:expr) => {
        unsafe {
            core::arch::asm!(
                ".byte 0x0f",
                ".byte 0x0f",
                ".byte {0}",
                ".byte 0x96",
                in(reg_byte) (($dst & 0x3f) << 3) | $src,
            );
        }
    };
}

macro_rules! pfrcpit1 {
    ($dst:expr, $src:expr) => {
        unsafe {
            core::arch::asm!(
                ".byte 0x0f",
                ".byte 0x0f",
                ".byte {0}",
                ".byte 0xa6",
                in(reg_byte) (($dst & 0x3f) << 3) | $src,
            );
        }
    };
}

macro_rules! pfrcpit2 {
    ($dst:expr, $src:expr) => {
        unsafe {
            core::arch::asm!(
                ".byte 0x0f",
                ".byte 0x0f",
                ".byte {0}",
                ".byte 0xb6",
                in(reg_byte) (($dst & 0x3f) << 3) | $src,
            );
        }
    };
}

macro_rules! pfrsqrt {
    ($dst:expr, $src:expr) => {
        unsafe {
            core::arch::asm!(
                ".byte 0x0f",
                ".byte 0x0f",
                ".byte {0}",
                ".byte 0x97",
                in(reg_byte) (($dst & 0x3f) << 3) | $src,
            );
        }
    };
}

macro_rules! pfrsqit1 {
    ($dst:expr, $src:expr) => {
        unsafe {
            core::arch::asm!(
                ".byte 0x0f",
                ".byte 0x0f",
                ".byte {0}",
                ".byte 0xa7",
                in(reg_byte) (($dst & 0x3f) << 3) | $src,
            );
        }
    };
}

macro_rules! pfsub {
    ($dst:expr, $src:expr) => {
        unsafe {
            core::arch::asm!(
                ".byte 0x0f",
                ".byte 0x0f",
                ".byte {0}",
                ".byte 0x9a",
                in(reg_byte) (($dst & 0x3f) << 3) | $src,
            );
        }
    };
}

macro_rules! pfsubr {
    ($dst:expr, $src:expr) => {
        unsafe {
            core::arch::asm!(
                ".byte 0x0f",
                ".byte 0x0f",
                ".byte {0}",
                ".byte 0xaa",
                in(reg_byte) (($dst & 0x3f) << 3) | $src,
            );
        }
    };
}

macro_rules! pi2fd {
    ($dst:expr, $src:expr) => {
        unsafe {
            core::arch::asm!(
                ".byte 0x0f",
                ".byte 0x0f",
                ".byte {0}",
                ".byte 0x0d",
                in(reg_byte) (($dst & 0x3f) << 3) | $src,
            );
        }
    };
}

macro_rules! femms {
    () => {
        unsafe {
            core::arch::asm!(
                ".byte 0x0f",
                ".byte 0x0e",
            );
        }
    };
}

macro_rules! pavgusb {
    ($dst:expr, $src:expr) => {
        unsafe {
            core::arch::asm!(
                ".byte 0x0f",
                ".byte 0x0f",
                ".byte {0}",
                ".byte 0xbf",
                in(reg_byte) (($dst & 0x3f) << 3) | $src,
            );
        }
    };
}

macro_rules! pmulhrw {
    ($dst:expr, $src:expr) => {
        unsafe {
            core::arch::asm!(
                ".byte 0x0f",
                ".byte 0x0f",
                ".byte {0}",
                ".byte 0xb7",
                in(reg_byte) (($dst & 0x3f) << 3) | $src,
            );
        }
    };
}

macro_rules! prefetch {
    ($src:expr) => {
        unsafe {
            core::arch::asm!(
                ".byte 0x0f",
                ".byte 0x0d",
                ".byte {0}",
                in(reg_byte) 0x00 | $src,
            );
        }
    };
}

macro_rules! prefetchw {
    ($src:expr) => {
        unsafe {
            core::arch::asm!(
                ".byte 0x0f",
                ".byte 0x0d",
                ".byte {0}",
                in(reg_byte) 0x08 | $src,
            );
        }
    };
}
