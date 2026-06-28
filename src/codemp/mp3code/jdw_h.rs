//! Mechanical port of `codemp/mp3code/jdw.h`.

/*
____________________________________________________________________________

    FreeAmp - The Free MP3 Player

    Copyright (C) 1998 EMusic.com

    This program is free software; you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation; either version 2 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program; if not, write to the Free Software
    Foundation, Inc., 675 Mass Ave, Cambridge, MA 02139, USA.

    $Id: jdw.h,v 1.2 1999/10/19 07:13:08 elrod Exp $
____________________________________________________________________________
*/

/* LOL */

// Rust has no preprocessor include guard or `#ifndef min`; this macro keeps the
// C name and expression shape (`((a>b)?b:a)`) for mechanical ports.
#[macro_export]
macro_rules! min {
    ($a:expr, $b:expr) => {
        if $a > $b {
            $b
        } else {
            $a
        }
    };
}
