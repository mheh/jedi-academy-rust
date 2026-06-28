#![allow(non_snake_case)]

// qasm.h - x86 assembly interface definitions
// Original: oracle/code/unix/qasm.h

// ELF vs non-ELF label naming:
// In the original C:
// #ifdef ELF
// #define C(label) label
// #else
// #define C(label) _##label
// #endif
//
// This macro handles platform-specific symbol name mangling, handled by the
// linker in Rust. Assembly extern symbols below are preserved for reference.

// #define GLQUAKE	1

// Platform-specific x86 detection
// In the original C:
// #if defined(_WIN32) && !defined(WINDED)
// #if defined(_M_IX86)
// #define __i386__	1
// #endif
// #endif
//
// #ifdef __i386__
// #define id386	1
// #else
// #define id386	0
// #endif

#[cfg(target_arch = "x86")]
pub const id386: i32 = 1;

#[cfg(not(target_arch = "x86"))]
pub const id386: i32 = 0;

// !!! must be kept the same as in d_iface.h !!!
pub const TRANSPARENT_COLOR: u32 = 255;

// Assembly extern symbol declarations from the original .extern directives
// ported as comments for reference. In a Rust binary that doesn't call
// assembly code, these are not needed as Rust extern declarations.
//
// #ifndef GLQUAKE
// .extern C(d_zistepu)
// .extern C(d_pzbuffer)
// .extern C(d_zistepv)
// .extern C(d_zrowbytes)
// .extern C(d_ziorigin)
// .extern C(r_turb_s)
// .extern C(r_turb_t)
// .extern C(r_turb_pdest)
// .extern C(r_turb_spancount)
// .extern C(r_turb_turb)
// .extern C(r_turb_pbase)
// .extern C(r_turb_sstep)
// .extern C(r_turb_tstep)
// .extern	C(r_bmodelactive)
// .extern	C(d_sdivzstepu)
// .extern	C(d_tdivzstepu)
// .extern	C(d_sdivzstepv)
// .extern	C(d_tdivzstepv)
// .extern	C(d_sdivzorigin)
// .extern	C(d_tdivzorigin)
// .extern	C(sadjust)
// .extern	C(tadjust)
// .extern	C(bbextents)
// .extern	C(bbextentt)
// .extern	C(cacheblock)
// .extern	C(d_viewbuffer)
// .extern	C(cachewidth)
// .extern	C(d_pzbuffer)
// .extern	C(d_zrowbytes)
// .extern	C(d_zwidth)
// .extern C(d_scantable)
// .extern C(r_lightptr)
// .extern C(r_numvblocks)
// .extern C(prowdestbase)
// .extern C(pbasesource)
// .extern C(r_lightwidth)
// .extern C(lightright)
// .extern C(lightrightstep)
// .extern C(lightdeltastep)
// .extern C(lightdelta)
// .extern C(lightright)
// .extern C(lightdelta)
// .extern C(sourcetstep)
// .extern C(surfrowbytes)
// .extern C(lightrightstep)
// .extern C(lightdeltastep)
// .extern C(r_sourcemax)
// .extern C(r_stepback)
// .extern C(colormap)
// .extern C(blocksize)
// .extern C(sourcesstep)
// .extern C(lightleft)
// .extern C(blockdivshift)
// .extern C(blockdivmask)
// .extern C(lightleftstep)
// .extern C(r_origin)
// .extern C(r_ppn)
// .extern C(r_pup)
// .extern C(r_pright)
// .extern C(ycenter)
// .extern C(xcenter)
// .extern C(d_vrectbottom_particle)
// .extern C(d_vrectright_particle)
// .extern C(d_vrecty)
// .extern C(d_vrectx)
// .extern C(d_pix_shift)
// .extern C(d_pix_min)
// .extern C(d_pix_max)
// .extern C(d_y_aspect_shift)
// .extern C(screenwidth)
// .extern C(r_leftclipped)
// .extern C(r_leftenter)
// .extern C(r_rightclipped)
// .extern C(r_rightenter)
// .extern C(modelorg)
// .extern C(xscale)
// .extern C(r_refdef)
// .extern C(yscale)
// .extern C(r_leftexit)
// .extern C(r_rightexit)
// .extern C(r_lastvertvalid)
// .extern C(cacheoffset)
// .extern C(newedges)
// .extern C(removeedges)
// .extern C(r_pedge)
// .extern C(r_framecount)
// .extern C(r_u1)
// .extern C(r_emitted)
// .extern C(edge_p)
// .extern C(surface_p)
// .extern C(surfaces)
// .extern C(r_lzi1)
// .extern C(r_v1)
// .extern C(r_ceilv1)
// .extern C(r_nearzi)
// .extern C(r_nearzionly)
// .extern C(edge_aftertail)
// .extern C(edge_tail)
// .extern C(current_iv)
// .extern C(edge_head_u_shift20)
// .extern C(span_p)
// .extern C(edge_head)
// .extern C(fv)
// .extern C(edge_tail_u_shift20)
// .extern C(r_apverts)
// .extern C(r_anumverts)
// .extern C(aliastransform)
// .extern C(r_avertexnormals)
// .extern C(r_plightvec)
// .extern C(r_ambientlight)
// .extern C(r_shadelight)
// .extern C(aliasxcenter)
// .extern C(aliasycenter)
// .extern C(a_sstepxfrac)
// .extern C(r_affinetridesc)
// .extern C(acolormap)
// .extern C(d_pcolormap)
// .extern C(r_affinetridesc)
// .extern C(d_sfrac)
// .extern C(d_ptex)
// .extern C(d_pedgespanpackage)
// .extern C(d_tfrac)
// .extern C(d_light)
// .extern C(d_zi)
// .extern C(d_pdest)
// .extern C(d_pz)
// .extern C(d_aspancount)
// .extern C(erroradjustup)
// .extern C(errorterm)
// .extern C(d_xdenom)
// .extern C(r_p0)
// .extern C(r_p1)
// .extern C(r_p2)
// .extern C(a_tstepxfrac)
// .extern C(r_sstepx)
// .extern C(r_tstepx)
// .extern C(a_ststepxwhole)
// .extern C(zspantable)
// .extern C(skintable)
// .extern C(r_zistepx)
// .extern C(erroradjustdown)
// .extern C(d_countextrastep)
// .extern C(ubasestep)
// .extern C(a_ststepxwhole)
// .extern C(a_tstepxfrac)
// .extern C(r_lstepx)
// .extern C(a_spans)
// .extern C(erroradjustdown)
// .extern C(d_pdestextrastep)
// .extern C(d_pzextrastep)
// .extern C(d_sfracextrastep)
// .extern C(d_ptexextrastep)
// .extern C(d_countextrastep)
// .extern C(d_tfracextrastep)
// .extern C(d_lightextrastep)
// .extern C(d_ziextrastep)
// .extern C(d_pdestbasestep)
// .extern C(d_pzbasestep)
// .extern C(d_sfracbasestep)
// .extern C(d_ptexbasestep)
// .extern C(ubasestep)
// .extern C(d_tfracbasestep)
// .extern C(d_lightbasestep)
// .extern C(d_zibasestep)
// .extern C(zspantable)
// .extern C(r_lstepy)
// .extern C(r_sstepy)
// .extern C(r_tstepy)
// .extern C(r_zistepy)
// .extern C(D_PolysetSetEdgeTable)
// .extern C(D_RasterizeAliasPolySmooth)
//
// .extern float_point5
// .extern Float2ToThe31nd
// .extern izistep
// .extern izi
// .extern FloatMinus2ToThe31nd
// .extern float_1
// .extern float_particle_z_clip
// .extern float_minus_1
// .extern float_0
// .extern fp_16
// .extern fp_64k
// .extern fp_1m
// .extern fp_1m_minus_1
// .extern fp_8
// .extern entryvec_table
// .extern advancetable
// .extern sstep
// .extern tstep
// .extern pspantemp
// .extern counttemp
// .extern jumptemp
// .extern reciprocal_table
// .extern DP_Count
// .extern DP_u
// .extern DP_v
// .extern DP_32768
// .extern DP_Color
// .extern DP_Pix
// .extern DP_EntryTable
// .extern	pbase
// .extern s
// .extern t
// .extern sfracf
// .extern tfracf
// .extern snext
// .extern tnext
// .extern	spancountminus1
// .extern zi16stepu
// .extern sdivz16stepu
// .extern tdivz16stepu
// .extern	zi8stepu
// .extern sdivz8stepu
// .extern tdivz8stepu
// .extern reciprocal_table_16
// .extern entryvec_table_16
// .extern ceil_cw
// .extern single_cw
// .extern fp_64kx64k
// .extern pz
// .extern spr8entryvec_table
// #endif
//
// .extern C(snd_scaletable)
// .extern C(paintbuffer)
// .extern C(snd_linear_count)
// .extern C(snd_p)
// .extern C(snd_vol)
// .extern C(snd_out)
// .extern C(vright)
// .extern C(vup)
// .extern C(vpn)
// .extern C(BOPS_Error)

//
// !!! note that this file must match the corresponding C structures at all
// times !!!
//

// plane_t structure
// !!! if this is changed, it must be changed in model.h too !!!
// !!! if the size of this is changed, the array lookup in SV_HullPointContents
//     must be changed too !!!
pub const pl_normal: usize = 0;
pub const pl_dist: usize = 12;
pub const pl_type: usize = 16;
pub const pl_signbits: usize = 17;
pub const pl_pad: usize = 18;
pub const pl_size: usize = 20;

// hull_t structure
// !!! if this is changed, it must be changed in model.h too !!!
pub const hu_clipnodes: usize = 0;
pub const hu_planes: usize = 4;
pub const hu_firstclipnode: usize = 8;
pub const hu_lastclipnode: usize = 12;
pub const hu_clip_mins: usize = 16;
pub const hu_clip_maxs: usize = 28;
pub const hu_size: usize = 40;

// dnode_t structure
// !!! if this is changed, it must be changed in bspfile.h too !!!
pub const nd_planenum: usize = 0;
pub const nd_children: usize = 4;
pub const nd_mins: usize = 8;
pub const nd_maxs: usize = 20;
pub const nd_firstface: usize = 32;
pub const nd_numfaces: usize = 36;
pub const nd_size: usize = 40;

// sfxcache_t structure
// !!! if this is changed, it much be changed in sound.h too !!!
pub const sfxc_length: usize = 0;
pub const sfxc_loopstart: usize = 4;
pub const sfxc_speed: usize = 8;
pub const sfxc_width: usize = 12;
pub const sfxc_stereo: usize = 16;
pub const sfxc_data: usize = 20;

// channel_t structure
// !!! if this is changed, it much be changed in sound.h too !!!
pub const ch_sfx: usize = 0;
pub const ch_leftvol: usize = 4;
pub const ch_rightvol: usize = 8;
pub const ch_end: usize = 12;
pub const ch_pos: usize = 16;
pub const ch_looping: usize = 20;
pub const ch_entnum: usize = 24;
pub const ch_entchannel: usize = 28;
pub const ch_origin: usize = 32;
pub const ch_dist_mult: usize = 44;
pub const ch_master_vol: usize = 48;
pub const ch_size: usize = 52;

// portable_samplepair_t structure
// !!! if this is changed, it much be changed in sound.h too !!!
pub const psp_left: usize = 0;
pub const psp_right: usize = 4;
pub const psp_size: usize = 8;

//
// !!! note that this file must match the corresponding C structures at all
// times !!!
//

// !!! if this is changed, it must be changed in r_local.h too !!!
pub const NEAR_CLIP: f64 = 0.01;

// !!! if this is changed, it must be changed in r_local.h too !!!
pub const CYCLE: u32 = 128;

// espan_t structure
// !!! if this is changed, it must be changed in r_shared.h too !!!
pub const espan_t_u: usize = 0;
pub const espan_t_v: usize = 4;
pub const espan_t_count: usize = 8;
pub const espan_t_pnext: usize = 12;
pub const espan_t_size: usize = 16;

// sspan_t structure
// !!! if this is changed, it must be changed in d_local.h too !!!
pub const sspan_t_u: usize = 0;
pub const sspan_t_v: usize = 4;
pub const sspan_t_count: usize = 8;
pub const sspan_t_size: usize = 12;

// spanpackage_t structure
// !!! if this is changed, it must be changed in d_polyset.c too !!!
pub const spanpackage_t_pdest: usize = 0;
pub const spanpackage_t_pz: usize = 4;
pub const spanpackage_t_count: usize = 8;
pub const spanpackage_t_ptex: usize = 12;
pub const spanpackage_t_sfrac: usize = 16;
pub const spanpackage_t_tfrac: usize = 20;
pub const spanpackage_t_light: usize = 24;
pub const spanpackage_t_zi: usize = 28;
pub const spanpackage_t_size: usize = 32;

// edge_t structure
// !!! if this is changed, it must be changed in r_shared.h too !!!
pub const et_u: usize = 0;
pub const et_u_step: usize = 4;
pub const et_prev: usize = 8;
pub const et_next: usize = 12;
pub const et_surfs: usize = 16;
pub const et_nextremove: usize = 20;
pub const et_nearzi: usize = 24;
pub const et_owner: usize = 28;
pub const et_size: usize = 32;

// surf_t structure
// !!! if this is changed, it must be changed in r_shared.h too !!!
pub const SURF_T_SHIFT: u32 = 6;
pub const st_next: usize = 0;
pub const st_prev: usize = 4;
pub const st_spans: usize = 8;
pub const st_key: usize = 12;
pub const st_last_u: usize = 16;
pub const st_spanstate: usize = 20;
pub const st_flags: usize = 24;
pub const st_data: usize = 28;
pub const st_entity: usize = 32;
pub const st_nearzi: usize = 36;
pub const st_insubmodel: usize = 40;
pub const st_d_ziorigin: usize = 44;
pub const st_d_zistepu: usize = 48;
pub const st_d_zistepv: usize = 52;
pub const st_pad: usize = 56;
pub const st_size: usize = 64;

// clipplane_t structure
// !!! if this is changed, it must be changed in r_local.h too !!!
pub const cp_normal: usize = 0;
pub const cp_dist: usize = 12;
pub const cp_next: usize = 16;
pub const cp_leftedge: usize = 20;
pub const cp_rightedge: usize = 21;
pub const cp_reserved: usize = 22;
pub const cp_size: usize = 24;

// medge_t structure
// !!! if this is changed, it must be changed in model.h too !!!
pub const me_v: usize = 0;
pub const me_cachededgeoffset: usize = 4;
pub const me_size: usize = 8;

// mvertex_t structure
// !!! if this is changed, it must be changed in model.h too !!!
pub const mv_position: usize = 0;
pub const mv_size: usize = 12;

// refdef_t structure
// !!! if this is changed, it must be changed in render.h too !!!
pub const rd_vrect: usize = 0;
pub const rd_aliasvrect: usize = 20;
pub const rd_vrectright: usize = 40;
pub const rd_vrectbottom: usize = 44;
pub const rd_aliasvrectright: usize = 48;
pub const rd_aliasvrectbottom: usize = 52;
pub const rd_vrectrightedge: usize = 56;
pub const rd_fvrectx: usize = 60;
pub const rd_fvrecty: usize = 64;
pub const rd_fvrectx_adj: usize = 68;
pub const rd_fvrecty_adj: usize = 72;
pub const rd_vrect_x_adj_shift20: usize = 76;
pub const rd_vrectright_adj_shift20: usize = 80;
pub const rd_fvrectright_adj: usize = 84;
pub const rd_fvrectbottom_adj: usize = 88;
pub const rd_fvrectright: usize = 92;
pub const rd_fvrectbottom: usize = 96;
pub const rd_horizontalFieldOfView: usize = 100;
pub const rd_xOrigin: usize = 104;
pub const rd_yOrigin: usize = 108;
pub const rd_vieworg: usize = 112;
pub const rd_viewangles: usize = 124;
pub const rd_ambientlight: usize = 136;
pub const rd_size: usize = 140;

// mtriangle_t structure
// !!! if this is changed, it must be changed in model.h too !!!
pub const mtri_facesfront: usize = 0;
pub const mtri_vertindex: usize = 4;
pub const mtri_size: usize = 16;	// !!! if this changes, array indexing in !!!
									// !!! d_polysa.s must be changed to match !!!
pub const mtri_shift: u32 = 4;
