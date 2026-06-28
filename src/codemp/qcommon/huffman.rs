//! Mechanical port of `codemp/qcommon/huffman.cpp`.

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use core::ffi::{c_char, c_int, c_void};
use crate::codemp::game::q_shared_h::byte;

/* This is based on the Adaptive Huffman algorithm described in Sayood's Data
 * Compression book.  The ranks are not actually stored, but implicitly defined
 * by the location of a node within a doubly-linked list */

// Maximum symbol
pub const HMAX: c_int = 256;
// NYT = Not Yet Transmitted
pub const NYT: c_int = 256;
pub const INTERNAL_NODE: c_int = 257;

#[repr(C)]
pub struct node_t {
    /* tree structure */
    pub left: *mut node_t,
    pub right: *mut node_t,
    pub parent: *mut node_t,
    /* doubly-linked list */
    pub next: *mut node_t,
    pub prev: *mut node_t,
    /* highest ranked node in block */
    pub head: *mut *mut node_t,
    pub weight: c_int,
    pub symbol: c_int,
}

#[repr(C)]
pub struct huff_t {
    pub blocNode: c_int,
    pub blocPtrs: c_int,
    pub tree: *mut node_t,
    pub lhead: *mut node_t,
    pub ltail: *mut node_t,
    pub loc: [*mut node_t; (HMAX + 1) as usize],
    pub freelist: *mut *mut node_t,
    pub nodeList: [node_t; 768],
    pub nodePtrs: [*mut node_t; 768],
}

#[repr(C)]
pub struct msg_t {
    pub allowoverflow: c_int,
    pub overflowed: c_int,
    pub oob: c_int,
    pub data: *mut byte,
    pub maxsize: c_int,
    pub cursize: c_int,
    pub readcount: c_int,
    pub bit: c_int,
}

#[repr(C)]
pub struct huffman_t {
    pub compressor: huff_t,
    pub decompressor: huff_t,
}

extern "C" {
    pub fn Com_Memset(dest: *mut c_void, val: c_int, count: usize);
    pub fn Com_Memcpy(dest: *mut c_void, src: *const c_void, count: usize);
}

static mut bloc: c_int = 0;

pub fn Huff_putBit(bit: c_int, fout: *mut byte, offset: *mut c_int) {
    unsafe {
        bloc = *offset;
        if (bloc & 7) == 0 {
            *fout.add((bloc >> 3) as usize) = 0;
        }
        *fout.add((bloc >> 3) as usize) |= (bit << (bloc & 7)) as byte;
        bloc += 1;
        *offset = bloc;
    }
}

pub fn Huff_getBit(fin: *mut byte, offset: *mut c_int) -> c_int {
    let t: c_int;
    unsafe {
        bloc = *offset;
        t = ((*fin.add((bloc >> 3) as usize) >> (bloc & 7)) & 0x1) as c_int;
        bloc += 1;
        *offset = bloc;
    }
    t
}

/* Add a bit to the output file (buffered) */
fn add_bit(bit: c_char, fout: *mut byte) {
    unsafe {
        if (bloc & 7) == 0 {
            *fout.add((bloc >> 3) as usize) = 0;
        }
        *fout.add((bloc >> 3) as usize) |= (bit << (bloc & 7)) as byte;
        bloc += 1;
    }
}

/* Receive one bit from the input file (buffered) */
fn get_bit(fin: *mut byte) -> c_int {
    let t: c_int;
    unsafe {
        t = ((*fin.add((bloc >> 3) as usize) >> (bloc & 7)) & 0x1) as c_int;
        bloc += 1;
    }
    t
}

fn get_ppnode(huff: *mut huff_t) -> *mut *mut node_t {
    unsafe {
        if (*huff).freelist.is_null() {
            let result = &mut (*huff).nodePtrs[(*huff).blocPtrs as usize];
            (*huff).blocPtrs += 1;
            result as *mut *mut node_t
        } else {
            let tppnode = (*huff).freelist;
            (*huff).freelist = *(tppnode as *mut *mut *mut node_t);
            tppnode
        }
    }
}

fn free_ppnode(huff: *mut huff_t, ppnode: *mut *mut node_t) {
    unsafe {
        *(ppnode as *mut *mut *mut node_t) = (*huff).freelist as *mut *mut node_t;
        (*huff).freelist = ppnode;
    }
}

/* Swap the location of these two nodes in the tree */
fn swap(huff: *mut huff_t, node1: *mut node_t, node2: *mut node_t) {
    unsafe {
        let par1 = (*node1).parent;
        let par2 = (*node2).parent;

        if !par1.is_null() {
            if (*par1).left == node1 {
                (*par1).left = node2;
            } else {
                (*par1).right = node2;
            }
        } else {
            (*huff).tree = node2;
        }

        if !par2.is_null() {
            if (*par2).left == node2 {
                (*par2).left = node1;
            } else {
                (*par2).right = node1;
            }
        } else {
            (*huff).tree = node1;
        }

        (*node1).parent = par2;
        (*node2).parent = par1;
    }
}

/* Swap these two nodes in the linked list (update ranks) */
fn swaplist(node1: *mut node_t, node2: *mut node_t) {
    unsafe {
        let par1 = (*node1).next;
        (*node1).next = (*node2).next;
        (*node2).next = par1;

        let par1 = (*node1).prev;
        (*node1).prev = (*node2).prev;
        (*node2).prev = par1;

        if (*node1).next == node1 {
            (*node1).next = node2;
        }
        if (*node2).next == node2 {
            (*node2).next = node1;
        }
        if !(*node1).next.is_null() {
            (*(*node1).next).prev = node1;
        }
        if !(*node2).next.is_null() {
            (*(*node2).next).prev = node2;
        }
        if !(*node1).prev.is_null() {
            (*(*node1).prev).next = node1;
        }
        if !(*node2).prev.is_null() {
            (*(*node2).prev).next = node2;
        }
    }
}

/* Do the increments */
fn increment(huff: *mut huff_t, mut node: *mut node_t) {
    unsafe {
        if node.is_null() {
            return;
        }

        if !(*node).next.is_null() && (*(*node).next).weight == (*node).weight {
            let lnode = *(*node).head;
            if lnode != (*node).parent {
                swap(huff, lnode, node);
            }
            swaplist(lnode, node);
        }
        if !(*node).prev.is_null() && (*(*node).prev).weight == (*node).weight {
            *(*node).head = (*node).prev;
        } else {
            *(*node).head = core::ptr::null_mut();
            free_ppnode(huff, (*node).head);
        }
        (*node).weight += 1;
        if !(*node).next.is_null() && (*(*node).next).weight == (*node).weight {
            (*node).head = (*(*node).next).head;
        } else {
            (*node).head = get_ppnode(huff);
            *(*node).head = node;
        }
        if !(*node).parent.is_null() {
            increment(huff, (*node).parent);
            if (*node).prev == (*node).parent {
                swaplist(node, (*node).parent);
                if *(*node).head == node {
                    *(*node).head = (*node).parent;
                }
            }
        }
    }
}

pub fn Huff_addRef(huff: *mut huff_t, ch: byte) {
    unsafe {
        if (*huff).loc[ch as usize].is_null() {
            /* if this is the first transmission of this node */
            let tnode = &mut (*huff).nodeList[(*huff).blocNode as usize];
            (*huff).blocNode += 1;
            let tnode2 = &mut (*huff).nodeList[(*huff).blocNode as usize];
            (*huff).blocNode += 1;

            (*tnode2).symbol = INTERNAL_NODE;
            (*tnode2).weight = 1;
            (*tnode2).next = (*(*huff).lhead).next;
            if !(*(*huff).lhead).next.is_null() {
                (*(*(*huff).lhead).next).prev = tnode2;
                if (*(*(*huff).lhead).next).weight == 1 {
                    (*tnode2).head = (*(*(*huff).lhead).next).head;
                } else {
                    (*tnode2).head = get_ppnode(huff);
                    *(*tnode2).head = tnode2;
                }
            } else {
                (*tnode2).head = get_ppnode(huff);
                *(*tnode2).head = tnode2;
            }
            (*(*huff).lhead).next = tnode2;
            (*tnode2).prev = (*huff).lhead;

            (*tnode).symbol = ch as c_int;
            (*tnode).weight = 1;
            (*tnode).next = (*(*huff).lhead).next;
            if !(*(*huff).lhead).next.is_null() {
                (*(*(*huff).lhead).next).prev = tnode;
                if (*(*(*huff).lhead).next).weight == 1 {
                    (*tnode).head = (*(*(*huff).lhead).next).head;
                } else {
                    /* this should never happen */
                    (*tnode).head = get_ppnode(huff);
                    *(*tnode).head = tnode2;
                }
            } else {
                /* this should never happen */
                (*tnode).head = get_ppnode(huff);
                *(*tnode).head = tnode;
            }
            (*(*huff).lhead).next = tnode;
            (*tnode).prev = (*huff).lhead;
            (*tnode).left = core::ptr::null_mut();
            (*tnode).right = core::ptr::null_mut();

            if !(*(*huff).lhead).parent.is_null() {
                if (*(*(*huff).lhead).parent).left == (*huff).lhead {
                    /* lhead is guaranteed to by the NYT */
                    (*(*(*huff).lhead).parent).left = tnode2;
                } else {
                    (*(*(*huff).lhead).parent).right = tnode2;
                }
            } else {
                (*huff).tree = tnode2;
            }

            (*tnode2).right = tnode;
            (*tnode2).left = (*huff).lhead;

            (*tnode2).parent = (*(*huff).lhead).parent;
            (*(*huff).lhead).parent = tnode as *mut node_t;
            (*tnode).parent = tnode2 as *mut node_t;

            (*huff).loc[ch as usize] = tnode;

            increment(huff, (*tnode2).parent);
        } else {
            increment(huff, (*huff).loc[ch as usize]);
        }
    }
}

/* Get a symbol */
pub fn Huff_Receive(mut node: *mut node_t, ch: *mut c_int, fin: *mut byte) -> c_int {
    unsafe {
        while !node.is_null() && (*node).symbol == INTERNAL_NODE {
            if get_bit(fin) != 0 {
                node = (*node).right;
            } else {
                node = (*node).left;
            }
        }
        if node.is_null() {
            return 0;
            //		Com_Error(ERR_DROP, "Illegal tree!\n");
        }
        *ch = (*node).symbol;
        return (*node).symbol;
    }
}

/* Get a symbol */
pub fn Huff_offsetReceive(mut node: *mut node_t, ch: *mut c_int, fin: *mut byte, offset: *mut c_int) {
    unsafe {
        bloc = *offset;
        while !node.is_null() && (*node).symbol == INTERNAL_NODE {
            if get_bit(fin) != 0 {
                node = (*node).right;
            } else {
                node = (*node).left;
            }
        }
        if node.is_null() {
            *ch = 0;
            return;
            //		Com_Error(ERR_DROP, "Illegal tree!\n");
        }
        *ch = (*node).symbol;
        *offset = bloc;
    }
}

/* Send the prefix code for this node */
fn send(node: *mut node_t, child: *mut node_t, fout: *mut byte) {
    unsafe {
        if !(*node).parent.is_null() {
            send((*node).parent, node, fout);
        }
        if !child.is_null() {
            if (*node).right == child {
                add_bit(1, fout);
            } else {
                add_bit(0, fout);
            }
        }
    }
}

/* Send a symbol */
pub fn Huff_transmit(huff: *mut huff_t, ch: c_int, fout: *mut byte) {
    unsafe {
        if (*huff).loc[ch as usize].is_null() {
            /* node_t hasn't been transmitted, send a NYT, then the symbol */
            Huff_transmit(huff, NYT, fout);
            for i in (0..8).rev() {
                add_bit(((ch >> i) & 0x1) as c_char, fout);
            }
        } else {
            send((*huff).loc[ch as usize], core::ptr::null_mut(), fout);
        }
    }
}

pub fn Huff_offsetTransmit(huff: *mut huff_t, ch: c_int, fout: *mut byte, offset: *mut c_int) {
    unsafe {
        bloc = *offset;
        send((*huff).loc[ch as usize], core::ptr::null_mut(), fout);
        *offset = bloc;
    }
}

pub fn Huff_Decompress(mbuf: *mut msg_t, offset: c_int) {
    unsafe {
        let mut ch: c_int;
        let mut cch: c_int;
        let mut i: c_int;
        let mut j: c_int;
        let mut size: c_int;
        let mut seq: [byte; 65536] = [0; 65536];
        let buffer: *mut byte;
        let mut huff: huff_t = core::mem::zeroed();

        size = (*mbuf).cursize - offset;
        buffer = (*mbuf).data.add(offset as usize);

        if size <= 0 {
            return;
        }

        Com_Memset(&mut huff as *mut _ as *mut c_void, 0, core::mem::size_of::<huff_t>());
        // Initialize the tree & list with the NYT node
        let node_addr = &mut huff.nodeList[huff.blocNode as usize] as *mut node_t;
        huff.tree = node_addr;
        huff.lhead = node_addr;
        huff.ltail = node_addr;
        huff.loc[NYT as usize] = node_addr;
        huff.blocNode += 1;
        (*huff.tree).symbol = NYT;
        (*huff.tree).weight = 0;
        (*huff.lhead).next = core::ptr::null_mut();
        (*huff.lhead).prev = core::ptr::null_mut();
        (*huff.tree).parent = core::ptr::null_mut();
        (*huff.tree).left = core::ptr::null_mut();
        (*huff.tree).right = core::ptr::null_mut();

        cch = (*buffer as c_int) * 256 + (*buffer.add(1) as c_int);
        // don't overflow with bad messages
        if cch > (*mbuf).maxsize - offset {
            cch = (*mbuf).maxsize - offset;
        }
        bloc = 16;

        j = 0;
        while j < cch {
            ch = 0;
            // don't overflow reading from the messages
            // FIXME: would it be better to have a overflow check in get_bit ?
            if (bloc >> 3) > size {
                seq[j as usize] = 0;
                break;
            }
            Huff_Receive(huff.tree, &mut ch, buffer); /* Get a character */
            if ch == NYT {
                /* We got a NYT, get the symbol associated with it */
                ch = 0;
                i = 0;
                while i < 8 {
                    ch = (ch << 1) + get_bit(buffer);
                    i += 1;
                }
            }

            seq[j as usize] = ch as byte; /* Write symbol */

            Huff_addRef(&mut huff, ch as byte); /* Increment node */
            j += 1;
        }
        (*mbuf).cursize = cch + offset;
        Com_Memcpy(
            (*mbuf).data.add(offset as usize) as *mut c_void,
            seq.as_ptr() as *const c_void,
            cch as usize,
        );
    }
}

extern "C" {
    pub static mut oldsize: c_int;
}

pub fn Huff_Compress(mbuf: *mut msg_t, offset: c_int) {
    unsafe {
        let mut i: c_int;
        let mut ch: c_int;
        let mut size: c_int;
        let mut seq: [byte; 65536] = [0; 65536];
        let buffer: *mut byte;
        let mut huff: huff_t = core::mem::zeroed();

        size = (*mbuf).cursize - offset;
        buffer = (*mbuf).data.add(offset as usize);

        if size <= 0 {
            return;
        }

        Com_Memset(&mut huff as *mut _ as *mut c_void, 0, core::mem::size_of::<huff_t>());
        // Add the NYT (not yet transmitted) node into the tree/list */
        let node_addr = &mut huff.nodeList[huff.blocNode as usize] as *mut node_t;
        huff.tree = node_addr;
        huff.lhead = node_addr;
        huff.loc[NYT as usize] = node_addr;
        huff.blocNode += 1;
        (*huff.tree).symbol = NYT;
        (*huff.tree).weight = 0;
        (*huff.lhead).next = core::ptr::null_mut();
        (*huff.lhead).prev = core::ptr::null_mut();
        (*huff.tree).parent = core::ptr::null_mut();
        (*huff.tree).left = core::ptr::null_mut();
        (*huff.tree).right = core::ptr::null_mut();
        huff.loc[NYT as usize] = huff.tree;

        seq[0] = (size >> 8) as byte;
        seq[1] = (size & 0xff) as byte;

        bloc = 16;

        i = 0;
        while i < size {
            ch = *buffer.add(i as usize) as c_int;
            Huff_transmit(&mut huff, ch, seq.as_mut_ptr()); /* Transmit symbol */
            Huff_addRef(&mut huff, ch as byte); /* Do update */
            i += 1;
        }

        bloc += 8; /* next byte */

        (*mbuf).cursize = (bloc >> 3) + offset;
        Com_Memcpy(
            (*mbuf).data.add(offset as usize) as *mut c_void,
            seq.as_ptr() as *const c_void,
            (bloc >> 3) as usize,
        );
    }
}

pub fn Huff_Init(huff: *mut huffman_t) {
    unsafe {
        Com_Memset(
            &mut (*huff).compressor as *mut _ as *mut c_void,
            0,
            core::mem::size_of::<huff_t>(),
        );
        Com_Memset(
            &mut (*huff).decompressor as *mut _ as *mut c_void,
            0,
            core::mem::size_of::<huff_t>(),
        );

        // Initialize the tree & list with the NYT node
        let decomp_node_addr = &mut (*huff).decompressor.nodeList[(*huff).decompressor.blocNode as usize] as *mut node_t;
        (*huff).decompressor.tree = decomp_node_addr;
        (*huff).decompressor.lhead = decomp_node_addr;
        (*huff).decompressor.ltail = decomp_node_addr;
        (*huff).decompressor.loc[NYT as usize] = decomp_node_addr;
        (*huff).decompressor.blocNode += 1;
        (*(*huff).decompressor.tree).symbol = NYT;
        (*(*huff).decompressor.tree).weight = 0;
        (*(*huff).decompressor.lhead).next = core::ptr::null_mut();
        (*(*huff).decompressor.lhead).prev = core::ptr::null_mut();
        (*(*huff).decompressor.tree).parent = core::ptr::null_mut();
        (*(*huff).decompressor.tree).left = core::ptr::null_mut();
        (*(*huff).decompressor.tree).right = core::ptr::null_mut();

        // Add the NYT (not yet transmitted) node into the tree/list */
        let comp_node_addr = &mut (*huff).compressor.nodeList[(*huff).compressor.blocNode as usize] as *mut node_t;
        (*huff).compressor.tree = comp_node_addr;
        (*huff).compressor.lhead = comp_node_addr;
        (*huff).compressor.loc[NYT as usize] = comp_node_addr;
        (*huff).compressor.blocNode += 1;
        (*(*huff).compressor.tree).symbol = NYT;
        (*(*huff).compressor.tree).weight = 0;
        (*(*huff).compressor.lhead).next = core::ptr::null_mut();
        (*(*huff).compressor.lhead).prev = core::ptr::null_mut();
        (*(*huff).compressor.tree).parent = core::ptr::null_mut();
        (*(*huff).compressor.tree).left = core::ptr::null_mut();
        (*(*huff).compressor.tree).right = core::ptr::null_mut();
        (*huff).compressor.loc[NYT as usize] = (*huff).compressor.tree;
    }
}
