// leave this line at the top of all AI_xxxx.cpp files for PCH reasons...

#![allow(non_snake_case)]

use crate::code::game::g_headers_h::*;
use core::ffi::{c_char, c_int};
use core::ptr::{addr_of, addr_of_mut};

//custom anims:
//	both_attack1 - running attack
//	both_attack2 - crouched attack
//	both_attack3 - standing attack
//	both_stand1idle1 - idle
//	both_crouch2stand1 - uncrouch
//	both_death4 - running death

const ASSASSIN_SHIELD_SIZE: f32 = 75.0;
const TURN_ON: c_int = 0x00000000;
const TURN_OFF: c_int = 0x00000100;



////////////////////////////////////////////////////////////////////////////////////////
//
////////////////////////////////////////////////////////////////////////////////////////
pub unsafe fn BubbleShield_IsOn() -> bool {
	((*NPC).flags & FL_SHIELDED) != 0
}

////////////////////////////////////////////////////////////////////////////////////////
//
////////////////////////////////////////////////////////////////////////////////////////
pub unsafe fn BubbleShield_TurnOn() {
	if !BubbleShield_IsOn() {
		(*NPC).flags |= FL_SHIELDED;
		(*(*NPC).client).ps.powerups[PW_GALAK_SHIELD as usize] = Q3_INFINITE;
		gi.G2API_SetSurfaceOnOff( addr_of_mut!((*NPC).ghoul2[(*NPC).playerModel as usize]), b"force_shield\0".as_ptr() as *const c_char, TURN_ON );
	}
}

////////////////////////////////////////////////////////////////////////////////////////
//
////////////////////////////////////////////////////////////////////////////////////////
pub unsafe fn BubbleShield_TurnOff() {
	if BubbleShield_IsOn() {
		(*NPC).flags &= !FL_SHIELDED;
		(*(*NPC).client).ps.powerups[PW_GALAK_SHIELD as usize] = 0;
		gi.G2API_SetSurfaceOnOff( addr_of_mut!((*NPC).ghoul2[(*NPC).playerModel as usize]), b"force_shield\0".as_ptr() as *const c_char, TURN_OFF );
	}
}


////////////////////////////////////////////////////////////////////////////////////////
// Push A Particular Ent
////////////////////////////////////////////////////////////////////////////////////////
// Porting note: C `vec3_t smackDir` parameter decays to float* at ABI level; translated as *mut vec3_t.
pub unsafe fn BubbleShield_PushEnt(pushed: *mut gentity_t, smackDir: *mut vec3_t) {
	G_Damage(pushed, NPC, NPC, smackDir, addr_of!((*NPC).currentOrigin), ((*g_spskill).integer+1)*Q_irand( 5, 10), DAMAGE_NO_KNOCKBACK, MOD_ELECTROCUTE);
	G_Throw(pushed, smackDir, 10);

	// Make Em Electric
	//------------------
 	(*pushed).s.powerups |= 1 << PW_SHOCKED;
	if !(*pushed).client.is_null() {
		(*(*pushed).client).ps.powerups[PW_SHOCKED as usize] = level.time + 1000;
	}
}

////////////////////////////////////////////////////////////////////////////////////////
// Go Through All The Ents Within The Radius Of The Shield And Push Them
////////////////////////////////////////////////////////////////////////////////////////
pub unsafe fn BubbleShield_PushRadiusEnts() {
	let mut radiusEnts: [*mut gentity_t; 128] = [core::ptr::null_mut(); 128];
	let radius: f32 = ASSASSIN_SHIELD_SIZE;
	let mut mins: vec3_t = [0.0; 3];
	let mut maxs: vec3_t = [0.0; 3];
	let mut smackDir: vec3_t = [0.0; 3];

	for i in 0..3 {
		mins[i] = (*NPC).currentOrigin[i] - radius;
		maxs[i] = (*NPC).currentOrigin[i] + radius;
	}

	let numEnts: c_int = gi.EntitiesInBox(addr_of!(mins), addr_of!(maxs), radiusEnts.as_mut_ptr(), 128);
	for entIndex in 0..numEnts {
		// Only Clients
		//--------------
		if radiusEnts[entIndex as usize].is_null() || (*radiusEnts[entIndex as usize]).client.is_null() {
			continue;
		}

		// Don't Push Away Other Assassin Droids
		//---------------------------------------
		if (*(*radiusEnts[entIndex as usize]).client).NPC_class == (*(*NPC).client).NPC_class {
			continue;
		}

		// Should Have Already Pushed The Enemy If He Touched Us
		//-------------------------------------------------------
		if !(*NPC).enemy.is_null() && (*NPCInfo).touchedByPlayer == (*NPC).enemy && radiusEnts[entIndex as usize] == (*NPC).enemy {
			continue;
		}

		// Do The Vector Distance Test
		//-----------------------------
		VectorSubtract((*radiusEnts[entIndex as usize]).currentOrigin, (*NPC).currentOrigin, smackDir);
		let smackDist = VectorNormalize(smackDir);
		if smackDist < radius {
			BubbleShield_PushEnt(radiusEnts[entIndex as usize], addr_of_mut!(smackDir));
		}
	}
}

////////////////////////////////////////////////////////////////////////////////////////
//
////////////////////////////////////////////////////////////////////////////////////////
pub unsafe fn BubbleShield_Update() {
	// Shields Go When You Die
	//-------------------------
	if (*NPC).health <= 0 {
		if BubbleShield_IsOn() {
			BubbleShield_TurnOff();
		}
		return;
	}


	// Recharge Shields
	//------------------
 	(*(*NPC).client).ps.stats[STAT_ARMOR as usize] += 1;
	if (*(*NPC).client).ps.stats[STAT_ARMOR as usize] > 250 {
		(*(*NPC).client).ps.stats[STAT_ARMOR as usize] = 250;
	}




	// If We Have Enough Armor And Are Not Shooting Right Now, Kick The Shield On
	//----------------------------------------------------------------------------
 	if (*(*NPC).client).ps.stats[STAT_ARMOR as usize] > 100 && TIMER_Done(NPC, b"ShieldsDown\0".as_ptr() as *const c_char) != 0 {
		// Check On Timers To Raise And Lower Shields
		//--------------------------------------------
		if (level.time - (*NPCInfo).enemyLastSeenTime) < 1000 && TIMER_Done(NPC, b"ShieldsUp\0".as_ptr() as *const c_char) != 0 {
			TIMER_Set(NPC, b"ShieldsDown\0".as_ptr() as *const c_char, 2000);		// Drop Shields
			TIMER_Set(NPC, b"ShieldsUp\0".as_ptr() as *const c_char, Q_irand(4000, 5000));	// Then Bring Them Back Up For At Least 3 sec
		}

		BubbleShield_TurnOn();
		if BubbleShield_IsOn() {
			// Update Our Shader Value
			//-------------------------
			// Porting note: C chained assignment expanded; RHS int truncated to u8 (C implicit cast to byte).
		 	 	let customRGBA_val = ((*(*NPC).client).ps.stats[STAT_ARMOR as usize] - 100) as u8;
				(*(*NPC).client).renderInfo.customRGBA[0] = customRGBA_val;
				(*(*NPC).client).renderInfo.customRGBA[1] = customRGBA_val;
				(*(*NPC).client).renderInfo.customRGBA[2] = customRGBA_val;
  				(*(*NPC).client).renderInfo.customRGBA[3] = customRGBA_val;


			// If Touched By An Enemy, ALWAYS Shove Them
			//-------------------------------------------
			if !(*NPC).enemy.is_null() && (*NPCInfo).touchedByPlayer == (*NPC).enemy {
				let mut dir: vec3_t = [0.0; 3];
				VectorSubtract((*(*NPC).enemy).currentOrigin, (*NPC).currentOrigin, dir);
				VectorNormalize(dir);
				BubbleShield_PushEnt((*NPC).enemy, addr_of_mut!(dir));
			}

			// Push Anybody Else Near
			//------------------------
			BubbleShield_PushRadiusEnts();
		}
	}


	// Shields Gone
	//--------------
	else {
		BubbleShield_TurnOff();
	}
}
