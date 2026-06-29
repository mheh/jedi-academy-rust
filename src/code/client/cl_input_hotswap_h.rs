use core::ffi::c_int;

const HOTSWAP_ID_WHITE: c_int = 0;
const HOTSWAP_ID_BLACK: c_int = 1;

const HOTSWAP_CAT_WEAPON: c_int = 0;
const HOTSWAP_CAT_ITEM: c_int = 1;
const HOTSWAP_CAT_FORCE: c_int = 2;


#[repr(C)]
pub struct HotSwapManager {
	down: bool,		//Is the button down?
	noExec: bool,	//Don't execute the button's bind.
	noBind: bool,	//Don't bind the button.
	forceBound: bool,//Is a force power currently bound?
	downTime: c_int,	//How long the button has been held down.
	bindTime: c_int,	//How long the button has been down with the selection up.
	uniqueID: c_int,	//Unique ID for this button.
}

impl HotSwapManager {
	//Return the binding for the button, or NULL if none.
	fn GetBinding(&self) -> *const core::ffi::c_char {
		core::ptr::null()
	}

	//Returns true if the weapon/force/item select screen is up.
	fn HUDInBindState(&self) -> bool {
		false
	}

	//Returns true if the weapon/force/item select screen is up.
	fn ForceSelectUp(&self) -> bool {
		false
	}
	fn WeaponSelectUp(&self) -> bool {
		false
	}
	fn ItemSelectUp(&self) -> bool {
		false
	}

	//Binds the button based on the current HUD selection.
	fn Bind(&mut self) {
	}

	//Execute the current bind, if there is one.
	fn Execute(&mut self) {
	}

	//Reset the object to the default state.
	fn Reset(&mut self) {
	}

	pub fn new(uniqueID: c_int) -> Self {
		HotSwapManager {
			down: false,
			noExec: false,
			noBind: false,
			forceBound: false,
			downTime: 0,
			bindTime: 0,
			uniqueID,
		}
	}

	//Call every frame.  Uses cg.frametime to increment timers.
	pub fn Update(&mut self) {
	}

	//Set the button down or up.
	pub fn SetDown(&mut self) {
	}
	pub fn SetUp(&mut self) {
	}

	//Returns true if the button is currently down.
	pub fn ButtonDown(&self) -> bool {
		self.down
	}
}


//External bind function for sharing with UI.
extern "C" {
	pub fn HotSwapBind(buttonID: c_int, category: c_int, value: c_int);
}
