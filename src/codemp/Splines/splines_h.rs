// Translated from oracle/codemp/Splines/splines.h
// Faithful C++ to Rust translation preserving original structure and semantics

#![allow(non_snake_case)]

use core::ffi::c_int;

// Extern C includes would correspond to:
// #ifdef Q3RADIANT
// #include "../qgl.h"
// #else
// #include "../renderer/qgl.h"
// #endif
// #include "util_list.h"
// #include "util_str.h"
// #include "math_vector.h"

// LOCAL STUB: These would be imported from sibling modules in full port
type idList<T> = Vec<T>;
type idStr = String;
#[repr(C)]
#[derive(Clone, Copy)]
pub struct idVec3_t {
    pub data: [f32; 3],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct vec4_t {
    pub data: [f32; 4],
}

impl idVec3_t {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        idVec3_t { data: [x, y, z] }
    }
    pub fn Zero(&mut self) {
        self.data = [0.0, 0.0, 0.0];
    }
    pub fn set(&mut self, x: f32, y: f32, z: f32) {
        self.data[0] = x;
        self.data[1] = y;
        self.data[2] = z;
    }
    pub fn Length(&self) -> f32 {
        (self.data[0] * self.data[0] + self.data[1] * self.data[1] + self.data[2] * self.data[2]).sqrt()
    }
}

pub type fileHandle_t = c_int;

extern "C" {
    pub fn glBox(color: *mut idVec3_t, point: *mut idVec3_t, size: f32);
    pub fn glLabeledPoint(color: *mut idVec3_t, point: *mut idVec3_t, size: f32, label: *const core::ffi::c_char);
    pub fn qglBegin(mode: core::ffi::c_uint);
    pub fn qglVertex3fv(v: *const f32);
    pub fn qglEnd();
    pub fn Q_stricmp(s1: *const core::ffi::c_char, s2: *const core::ffi::c_char) -> c_int;
}

pub static mut blue: vec4_t = vec4_t { data: [0.0, 0.0, 1.0, 1.0] };
pub static mut red: vec4_t = vec4_t { data: [1.0, 0.0, 0.0, 1.0] };

// LOCAL STUB: Vector math macros
fn DotProduct(a: idVec3_t, b: idVec3_t) -> f32 {
    a.data[0] * b.data[0] + a.data[1] * b.data[1] + a.data[2] * b.data[2]
}

fn __VectorMA(origin: idVec3_t, scale: f32, direction: idVec3_t, result: &mut idVec3_t) {
    result.data[0] = origin.data[0] + scale * direction.data[0];
    result.data[1] = origin.data[1] + scale * direction.data[1];
    result.data[2] = origin.data[2] + scale * direction.data[2];
}

pub struct idPointListInterface {
    pub selectedPoints: idList<i32>,
}

impl idPointListInterface {
    pub fn new() -> Self {
        idPointListInterface {
            selectedPoints: Vec::new(),
        }
    }

    pub fn numPoints(&self) -> i32 {
        0
    }

    pub fn addPoint_f(&mut self, x: f32, y: f32, z: f32) {
        // virtual
    }

    pub fn addPoint_v(&mut self, v: &idVec3_t) {
        // virtual
    }

    pub fn removePoint(&mut self, index: i32) {
        // virtual
    }

    pub fn getPoint(&mut self, index: i32) -> *mut idVec3_t {
        std::ptr::null_mut()
    }

    pub fn selectPointByRay_f(&mut self, ox: f32, oy: f32, oz: f32, dx: f32, dy: f32, dz: f32, single: bool) -> i32 {
        let origin = idVec3_t::new(ox, oy, oz);
        let dir = idVec3_t::new(dx, dy, dz);
        self.selectPointByRay_v(origin, dir, single)
    }

    pub fn selectPointByRay_v(&mut self, origin: idVec3_t, direction: idVec3_t, single: bool) -> i32 {
        let mut i: i32 = 0;
        let mut besti: i32 = -1;
        let mut count: i32 = 0;
        let mut d: f32 = 0.0;
        let mut bestd: f32 = 0.0;
        let mut temp: idVec3_t = idVec3_t::new(0.0, 0.0, 0.0);
        let mut temp2: idVec3_t = idVec3_t::new(0.0, 0.0, 0.0);

        // find the point closest to the ray
        besti = -1;
        bestd = 8.0;
        count = self.numPoints();

        while i < count {
            temp = *self.getPoint(i);
            temp2 = temp;
            temp.data[0] -= origin.data[0];
            temp.data[1] -= origin.data[1];
            temp.data[2] -= origin.data[2];
            d = DotProduct(temp, direction);
            __VectorMA(origin, d, direction, &mut temp);
            temp2.data[0] -= temp.data[0];
            temp2.data[1] -= temp.data[1];
            temp2.data[2] -= temp.data[2];
            d = temp2.Length();
            if d <= bestd {
                bestd = d;
                besti = i;
            }
            i += 1;
        }

        if besti >= 0 {
            self.selectPoint(besti, single);
        }

        besti
    }

    pub fn isPointSelected(&self, index: i32) -> i32 {
        let count = self.selectedPoints.len() as i32;
        let mut i = 0;
        while i < count {
            if self.selectedPoints[i as usize] == index {
                return i;
            }
            i += 1;
        }
        -1
    }

    pub fn selectPoint(&mut self, index: i32, single: bool) -> i32 {
        if index >= 0 && index < self.numPoints() {
            if single {
                self.deselectAll();
            } else {
                if self.isPointSelected(index) >= 0 {
                    // selectedPoints.Remove(index);
                    // LOCAL STUB: Remove logic
                    if let Some(pos) = self.selectedPoints.iter().position(|&x| x == index) {
                        self.selectedPoints.remove(pos);
                    }
                }
            }
            self.selectedPoints.push(index);
            return (self.selectedPoints.len() - 1) as i32;
        }
        -1
    }

    pub fn selectAll(&mut self) {
        self.selectedPoints.clear();
        let mut i = 0;
        while i < self.numPoints() {
            self.selectedPoints.push(i);
            i += 1;
        }
    }

    pub fn deselectAll(&mut self) {
        self.selectedPoints.clear();
    }

    pub fn numSelectedPoints(&self) -> i32 {
        self.selectedPoints.len() as i32
    }

    pub fn getSelectedPoint(&mut self, index: i32) -> *mut idVec3_t {
        assert!((index >= 0) && (index < self.numSelectedPoints()));
        self.getPoint(self.selectedPoints[index as usize])
    }

    pub fn updateSelection_f(&mut self, x: f32, y: f32, z: f32) {
        let move_v = idVec3_t::new(x, y, z);
        self.updateSelection_v(move_v);
    }

    pub fn updateSelection_v(&mut self, move_v: idVec3_t) {
        let count = self.selectedPoints.len() as i32;
        let mut i = 0;
        while i < count {
            let pt = self.getPoint(self.selectedPoints[i as usize]);
            unsafe {
                (*pt).data[0] += move_v.data[0];
                (*pt).data[1] += move_v.data[1];
                (*pt).data[2] += move_v.data[2];
            }
            i += 1;
        }
    }

    pub fn drawSelection(&mut self) {
        let count = self.selectedPoints.len() as i32;
        let mut i = 0;
        while i < count {
            unsafe {
                let mut red_copy = red;
                glBox(&mut red_copy, self.getPoint(self.selectedPoints[i as usize]), 4.0);
            }
            i += 1;
        }
    }
}

pub struct idSplineList {
    pub name: idStr,
    pub controlPoints: idList<*mut idVec3_t>,
    pub splinePoints: idList<*mut idVec3_t>,
    pub splineTime: idList<f64>,
    pub selected: *mut idVec3_t,
    pub pathColor: idVec3_t,
    pub segmentColor: idVec3_t,
    pub controlColor: idVec3_t,
    pub activeColor: idVec3_t,
    pub granularity: f32,
    pub editMode: bool,
    pub dirty: bool,
    pub activeSegment: i32,
    pub baseTime: i64,
    pub time: i64,
}

impl idSplineList {
    pub fn new() -> Self {
        let mut list = idSplineList {
            name: String::new(),
            controlPoints: Vec::new(),
            splinePoints: Vec::new(),
            splineTime: Vec::new(),
            selected: std::ptr::null_mut(),
            pathColor: idVec3_t::new(0.0, 0.0, 0.0),
            segmentColor: idVec3_t::new(0.0, 0.0, 0.0),
            controlColor: idVec3_t::new(0.0, 0.0, 0.0),
            activeColor: idVec3_t::new(0.0, 0.0, 0.0),
            granularity: 0.025,
            editMode: false,
            dirty: true,
            activeSegment: 0,
            baseTime: 0,
            time: 0,
        };
        list.clear();
        list
    }

    pub fn new_with_name(p: &str) -> Self {
        let mut list = idSplineList::new();
        list.name = p.to_string();
        list
    }

    pub fn clearControl(&mut self) {
        for i in 0..self.controlPoints.len() {
            unsafe {
                let _ = Box::from_raw(self.controlPoints[i]);
            }
        }
        self.controlPoints.clear();
    }

    pub fn clearSpline(&mut self) {
        for i in 0..self.splinePoints.len() {
            unsafe {
                let _ = Box::from_raw(self.splinePoints[i]);
            }
        }
        self.splinePoints.clear();
    }

    pub fn parse(&mut self, text: *const *const core::ffi::c_char) {
        // Declared but not defined in header
    }

    pub fn write(&mut self, file: fileHandle_t, name: *const core::ffi::c_char) {
        // Declared but not defined in header
    }

    pub fn clear(&mut self) {
        self.clearControl();
        self.clearSpline();
        self.splineTime.clear();
        self.selected = std::ptr::null_mut();
        self.dirty = true;
        self.activeSegment = 0;
        self.granularity = 0.025;
        self.pathColor.set(1.0, 0.5, 0.0);
        self.controlColor.set(0.7, 0.0, 1.0);
        self.segmentColor.set(0.0, 0.0, 1.0);
        self.activeColor.set(1.0, 0.0, 0.0);
    }

    pub fn initPosition(&mut self, startTime: i64, totalTime: i64) {
        // Declared but not defined in header
    }

    pub fn getPosition(&self, time: i64) -> *const idVec3_t {
        // Declared but not defined in header
        std::ptr::null()
    }

    pub fn draw(&mut self, editMode: bool) {
        // Declared but not defined in header
    }

    pub fn addToRenderer(&mut self) {
        // Declared but not defined in header
    }

    pub fn setSelectedPoint(&mut self, p: *mut idVec3_t) {
        self.selected = p;
    }

    pub fn getSelectedPoint(&self) -> *mut idVec3_t {
        self.selected
    }

    pub fn addPoint_v(&mut self, v: &idVec3_t) {
        let boxed = Box::new(*v);
        self.controlPoints.push(Box::into_raw(boxed));
        self.dirty = true;
    }

    pub fn addPoint_f(&mut self, x: f32, y: f32, z: f32) {
        let v = idVec3_t::new(x, y, z);
        self.addPoint_v(&v);
    }

    pub fn updateSelection(&mut self, move_v: &idVec3_t) {
        let count = self.selectedPoints.len() as i32;
        // Local stub: selectedPoints is not a field of idSplineList in the original
        // This is inherited from idPointListInterface interface method
    }

    pub fn startEdit(&mut self) {
        self.editMode = true;
    }

    pub fn stopEdit(&mut self) {
        self.editMode = false;
    }

    pub fn buildSpline(&mut self) {
        // Declared but not defined in header
    }

    pub fn setGranularity(&mut self, f: f32) {
        self.granularity = f;
    }

    pub fn getGranularity(&self) -> f32 {
        self.granularity
    }

    pub fn numPoints(&self) -> i32 {
        self.controlPoints.len() as i32
    }

    pub fn getPoint(&self, index: i32) -> *mut idVec3_t {
        assert!((index >= 0) && (index < self.controlPoints.len() as i32));
        self.controlPoints[index as usize]
    }

    pub fn getSegmentPoint(&self, index: i32) -> *mut idVec3_t {
        assert!((index >= 0) && (index < self.splinePoints.len() as i32));
        self.splinePoints[index as usize]
    }

    pub fn setSegmentTime(&mut self, index: i32, time: i32) {
        assert!((index >= 0) && (index < self.splinePoints.len() as i32));
        self.splineTime[index as usize] = time as f64;
    }

    pub fn getSegmentTime(&self, index: i32) -> f64 {
        assert!((index >= 0) && (index < self.splinePoints.len() as i32));
        self.splineTime[index as usize]
    }

    pub fn addSegmentTime(&mut self, index: i32, time: i32) {
        assert!((index >= 0) && (index < self.splinePoints.len() as i32));
        self.splineTime[index as usize] += time as f64;
    }

    pub fn totalDistance(&self) -> f32 {
        // Declared but not defined in header
        0.0
    }

    pub fn zero() -> idVec3_t {
        idVec3_t::new(0.0, 0.0, 0.0)
    }

    pub fn getActiveSegment(&self) -> i32 {
        self.activeSegment
    }

    pub fn setActiveSegment(&mut self, i: i32) {
        //assert!(i >= 0 && (splinePoints.Num() > 0 && i < splinePoints.Num()));
        self.activeSegment = i;
    }

    pub fn numSegments(&self) -> i32 {
        self.splinePoints.len() as i32
    }

    pub fn setColors(&mut self, path: &idVec3_t, segment: &idVec3_t, control: &idVec3_t, active: &idVec3_t) {
        self.pathColor = *path;
        self.segmentColor = *segment;
        self.controlColor = *control;
        self.activeColor = *active;
    }

    pub fn getName(&self) -> *const core::ffi::c_char {
        self.name.as_ptr() as *const core::ffi::c_char
    }

    pub fn setName(&mut self, p: &str) {
        self.name = p.to_string();
    }

    pub fn validTime(&mut self) -> bool {
        if self.dirty {
            self.buildSpline();
        }
        // gcc doesn't allow static casting away from bools
        // why?  I've no idea...
        (self.splineTime.len() as i32 > 0) && (self.splineTime.len() as i32 == self.splinePoints.len() as i32)
    }

    pub fn setTime(&mut self, t: i64) {
        self.time = t;
    }

    pub fn setBaseTime(&mut self, t: i64) {
        self.baseTime = t;
    }

    fn calcSpline(&mut self, step: i32, tension: f32) -> f32 {
        // Declared but not defined in header
        0.0
    }
}

// time in milliseconds
// velocity where 1.0 equal rough walking speed
#[repr(C)]
pub struct idVelocity {
    pub startTime: i64,
    pub time: i64,
    pub speed: f32,
}

impl idVelocity {
    pub fn new(start: i64, duration: i64, s: f32) -> Self {
        idVelocity {
            startTime: start,
            time: duration,
            speed: s,
        }
    }
}

// can either be a look at or origin position for a camera
//
pub struct idCameraPosition {
    pub velocities: idList<*mut idVelocity>,
    pub startTime: i64,
    pub time: i64,
    pub position_type: idCameraPositionType,
    pub name: idStr,
    pub editMode: bool,
    pub baseVelocity: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum idCameraPositionType {
    FIXED = 0x00,
    INTERPOLATED,
    SPLINE,
    POSITION_COUNT,
}

impl idCameraPosition {
    pub fn new() -> Self {
        idCameraPosition {
            velocities: Vec::new(),
            startTime: 0,
            time: 0,
            position_type: idCameraPositionType::FIXED,
            name: "position".to_string(),
            editMode: false,
            baseVelocity: 0.0,
        }
    }

    pub fn new_with_name(p: &str) -> Self {
        idCameraPosition {
            velocities: Vec::new(),
            startTime: 0,
            time: 0,
            position_type: idCameraPositionType::FIXED,
            name: p.to_string(),
            editMode: false,
            baseVelocity: 0.0,
        }
    }

    pub fn new_with_time(t: i64) -> Self {
        idCameraPosition {
            velocities: Vec::new(),
            startTime: 0,
            time: t,
            position_type: idCameraPositionType::FIXED,
            name: "position".to_string(),
            editMode: false,
            baseVelocity: 0.0,
        }
    }

    pub fn clear(&mut self) {
        self.editMode = false;
        for i in 0..self.velocities.len() {
            unsafe {
                let _ = Box::from_raw(self.velocities[i]);
                self.velocities[i] = std::ptr::null_mut();
            }
        }
        self.velocities.clear();
    }

    // this can be done with RTTI syntax but i like the derived classes setting a type
    // makes serialization a bit easier to see
    //

    pub fn start(&mut self, t: i64) {
        self.startTime = t;
    }

    pub fn getTime(&self) -> i64 {
        self.time
    }

    pub fn setTime(&mut self, t: i64) {
        self.time = t;
    }

    pub fn getVelocity(&self, t: i64) -> f32 {
        let check = t - self.startTime;
        for i in 0..self.velocities.len() {
            unsafe {
                let vel = &*self.velocities[i];
                if check >= vel.startTime && check <= vel.startTime + vel.time {
                    return vel.speed;
                }
            }
        }
        self.baseVelocity
    }

    pub fn addVelocity(&mut self, start: i64, duration: i64, speed: f32) {
        let vel = Box::new(idVelocity::new(start, duration, speed));
        self.velocities.push(Box::into_raw(vel));
    }

    pub fn getPosition(&self, t: i64) -> *const idVec3_t {
        assert!(true);
        std::ptr::null()
    }

    pub fn draw_bool(&mut self, editMode: bool) {
        // virtual
    }

    pub fn draw(&mut self) {
        // virtual
    }

    pub fn parse(&mut self, text: *const *const core::ffi::c_char) {
        // virtual
    }

    pub fn write(&mut self, file: fileHandle_t, name: *const core::ffi::c_char) {
        // virtual
    }

    pub fn parseToken(&mut self, key: *const core::ffi::c_char, text: *const *const core::ffi::c_char) -> bool {
        // virtual
        false
    }

    pub fn getName(&self) -> *const core::ffi::c_char {
        self.name.as_ptr() as *const core::ffi::c_char
    }

    pub fn setName(&mut self, p: &str) {
        self.name = p.to_string();
    }

    pub fn startEdit(&mut self) {
        self.editMode = true;
    }

    pub fn stopEdit(&mut self) {
        self.editMode = false;
    }

    pub fn typeStr(&self) -> *const core::ffi::c_char {
        let pos_strs: [&str; 4] = ["FIXED", "INTERPOLATED", "SPLINE", ""];
        pos_strs[self.position_type as usize].as_ptr() as *const core::ffi::c_char
    }

    pub fn calcVelocity(&mut self, distance: f32) {
        let secs = (self.time as f32) / 1000.0;
        self.baseVelocity = distance / secs;
    }
}

pub struct idFixedPosition {
    pub base: idCameraPosition,
    pub pos: idVec3_t,
}

impl idFixedPosition {
    pub fn new() -> Self {
        let mut this = idFixedPosition {
            base: idCameraPosition::new(),
            pos: idVec3_t::new(0.0, 0.0, 0.0),
        };
        this.init();
        this
    }

    pub fn new_with_pos(p: idVec3_t) -> Self {
        let mut this = idFixedPosition {
            base: idCameraPosition::new(),
            pos: idVec3_t::new(0.0, 0.0, 0.0),
        };
        this.init();
        this.pos = p;
        this
    }

    fn init(&mut self) {
        self.pos.Zero();
        self.base.position_type = idCameraPositionType::FIXED;
    }

    pub fn addPoint_v(&mut self, v: &idVec3_t) {
        self.pos = *v;
    }

    pub fn addPoint_f(&mut self, x: f32, y: f32, z: f32) {
        self.pos.set(x, y, z);
    }

    pub fn getPosition(&self, t: i64) -> *const idVec3_t {
        &self.pos as *const idVec3_t
    }

    pub fn parse(&mut self, text: *const *const core::ffi::c_char) {
        // Declared but not defined in header
    }

    pub fn write(&mut self, file: fileHandle_t, name: *const core::ffi::c_char) {
        // Declared but not defined in header
    }

    pub fn numPoints(&self) -> i32 {
        1
    }

    pub fn getPoint(&self, index: i32) -> *mut idVec3_t {
        if index != 0 {
            assert!(true);
        };
        (&self.pos as *const idVec3_t) as *mut idVec3_t
    }

    pub fn draw(&mut self, editMode: bool) {
        unsafe {
            let mut blue_copy = blue;
            glLabeledPoint(&mut blue_copy, (&mut self.pos) as *mut idVec3_t, if editMode { 5.0 } else { 3.0 }, "Fixed point\0".as_ptr() as *const core::ffi::c_char);
        }
    }
}

pub struct idInterpolatedPosition {
    pub base: idCameraPosition,
    pub first: bool,
    pub startPos: idVec3_t,
    pub endPos: idVec3_t,
    pub lastTime: i64,
    pub distSoFar: f32,
}

impl idInterpolatedPosition {
    pub fn new() -> Self {
        let mut this = idInterpolatedPosition {
            base: idCameraPosition::new(),
            first: true,
            startPos: idVec3_t::new(0.0, 0.0, 0.0),
            endPos: idVec3_t::new(0.0, 0.0, 0.0),
            lastTime: 0,
            distSoFar: 0.0,
        };
        this.init();
        this
    }

    pub fn new_with_pos(start: idVec3_t, end: idVec3_t, time: i64) -> Self {
        let mut this = idInterpolatedPosition {
            base: idCameraPosition::new_with_time(time),
            first: true,
            startPos: start,
            endPos: end,
            lastTime: 0,
            distSoFar: 0.0,
        };
        this.init();
        this
    }

    fn init(&mut self) {
        self.base.position_type = idCameraPositionType::INTERPOLATED;
        self.first = true;
        self.startPos.Zero();
        self.endPos.Zero();
    }

    pub fn getPosition(&mut self, t: i64) -> *const idVec3_t {
        // Declared but not defined in header
        std::ptr::null()
    }

    pub fn parse(&mut self, text: *const *const core::ffi::c_char) {
        // Declared but not defined in header
    }

    pub fn write(&mut self, file: fileHandle_t, name: *const core::ffi::c_char) {
        // Declared but not defined in header
    }

    pub fn numPoints(&self) -> i32 {
        2
    }

    pub fn getPoint(&mut self, index: i32) -> *mut idVec3_t {
        assert!((index >= 0) && (index < 2));
        if index == 0 {
            (&mut self.startPos) as *mut idVec3_t
        } else {
            (&mut self.endPos) as *mut idVec3_t
        }
    }

    pub fn addPoint_f(&mut self, x: f32, y: f32, z: f32) {
        if self.first {
            self.startPos.set(x, y, z);
            self.first = false;
        } else {
            self.endPos.set(x, y, z);
            self.first = true;
        }
    }

    pub fn addPoint_v(&mut self, v: &idVec3_t) {
        if self.first {
            self.startPos = *v;
            self.first = false;
        } else {
            self.endPos = *v;
            self.first = true;
        }
    }

    pub fn draw(&mut self, editMode: bool) {
        unsafe {
            let mut blue_copy = blue;
            glLabeledPoint(&mut blue_copy, (&mut self.startPos) as *mut idVec3_t, if editMode { 5.0 } else { 3.0 }, "Start interpolated\0".as_ptr() as *const core::ffi::c_char);
            glLabeledPoint(&mut blue_copy, (&mut self.endPos) as *mut idVec3_t, if editMode { 5.0 } else { 3.0 }, "End interpolated\0".as_ptr() as *const core::ffi::c_char);
            qglBegin(1); // GL_LINES
            qglVertex3fv((&self.startPos.data) as *const f32);
            qglVertex3fv((&self.endPos.data) as *const f32);
            qglEnd();
        }
    }

    pub fn start(&mut self, t: i64) {
        self.base.start(t);
        self.lastTime = self.base.startTime;
        self.distSoFar = 0.0;
        let mut temp = self.startPos;
        temp.data[0] -= self.endPos.data[0];
        temp.data[1] -= self.endPos.data[1];
        temp.data[2] -= self.endPos.data[2];
        self.base.calcVelocity(temp.Length());
    }
}

pub struct idSplinePosition {
    pub base: idCameraPosition,
    pub target: idSplineList,
}

impl idSplinePosition {
    pub fn new() -> Self {
        let mut this = idSplinePosition {
            base: idCameraPosition::new(),
            target: idSplineList::new(),
        };
        this.init();
        this
    }

    pub fn new_with_time(time: i64) -> Self {
        let mut this = idSplinePosition {
            base: idCameraPosition::new_with_time(time),
            target: idSplineList::new(),
        };
        this.init();
        this
    }

    fn init(&mut self) {
        self.base.position_type = idCameraPositionType::SPLINE;
    }

    pub fn start(&mut self, t: i64) {
        self.base.start(t);
        self.target.initPosition(t, self.base.time);
        self.base.calcVelocity(self.target.totalDistance());
    }

    pub fn getPosition(&self, t: i64) -> *const idVec3_t {
        self.target.getPosition(t)
    }

    //virtual const idVec3_t *getPosition(long t) const {

    pub fn addControlPoint(&mut self, v: &idVec3_t) {
        self.target.addPoint_v(v);
    }

    pub fn parse(&mut self, text: *const *const core::ffi::c_char) {
        // Declared but not defined in header
    }

    pub fn write(&mut self, file: fileHandle_t, name: *const core::ffi::c_char) {
        // Declared but not defined in header
    }

    pub fn numPoints(&self) -> i32 {
        self.target.numPoints()
    }

    pub fn getPoint(&self, index: i32) -> *mut idVec3_t {
        self.target.getPoint(index)
    }

    pub fn addPoint_v(&mut self, v: &idVec3_t) {
        self.target.addPoint_v(v);
    }

    pub fn addPoint_f(&mut self, x: f32, y: f32, z: f32) {
        self.target.addPoint_f(x, y, z);
    }

    pub fn draw(&mut self, editMode: bool) {
        self.target.draw(editMode);
    }

    pub fn updateSelection(&mut self, move_v: &idVec3_t) {
        // idCameraPosition::updateSelection(move);
        self.target.buildSpline();
    }
}

pub struct idCameraFOV {
    pub time: i64,
    pub fov: f32,
    pub startFOV: f32,
    pub endFOV: f32,
    pub startTime: i64,
}

impl idCameraFOV {
    pub fn new() -> Self {
        idCameraFOV {
            time: 0,
            fov: 90.0,
            startFOV: 0.0,
            endFOV: 0.0,
            startTime: 0,
        }
    }

    pub fn new_with_fov(v: i32) -> Self {
        idCameraFOV {
            time: 0,
            fov: v as f32,
            startFOV: 0.0,
            endFOV: 0.0,
            startTime: 0,
        }
    }

    pub fn new_with_range(s: i32, e: i32, t: i64) -> Self {
        idCameraFOV {
            startFOV: s as f32,
            endFOV: e as f32,
            time: t,
            fov: s as f32,
            startTime: 0,
        }
    }

    pub fn setFOV(&mut self, f: f32) {
        self.fov = f;
    }

    pub fn getFOV(&mut self, t: i64) -> f32 {
        if self.time != 0 {
            assert!(self.startTime != 0);
            let percent = (t as f32) / (self.startTime as f32);
            let mut temp = self.startFOV - self.endFOV;
            temp *= percent;
            self.fov = self.startFOV + temp;
        }
        self.fov
    }

    pub fn start(&mut self, t: i64) {
        self.startTime = t;
    }

    pub fn parse(&mut self, text: *const *const core::ffi::c_char) {
        // Declared but not defined in header
    }

    pub fn write(&mut self, file: fileHandle_t, name: *const core::ffi::c_char) {
        // Declared but not defined in header
    }
}

pub struct idCameraEvent {
    pub event_type: idCameraEventType,
    pub paramStr: idStr,
    pub time: i64,
    pub triggered: bool,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum idCameraEventType {
    EVENT_NA = 0x00,
    EVENT_WAIT,
    EVENT_TARGETWAIT,
    EVENT_SPEED,
    EVENT_TARGET,
    EVENT_SNAPTARGET,
    EVENT_FOV,
    EVENT_SCRIPT,
    EVENT_TRIGGER,
    EVENT_STOP,
    EVENT_COUNT,
}

impl idCameraEvent {
    pub fn new() -> Self {
        idCameraEvent {
            paramStr: String::new(),
            event_type: idCameraEventType::EVENT_NA,
            time: 0,
            triggered: false,
        }
    }

    pub fn new_with_data(t: idCameraEventType, param: &str, n: i64) -> Self {
        idCameraEvent {
            event_type: t,
            paramStr: param.to_string(),
            time: n,
            triggered: false,
        }
    }

    pub fn getType(&self) -> idCameraEventType {
        self.event_type
    }

    pub fn typeStr(&self) -> *const core::ffi::c_char {
        let event_strs: [&str; 10] = [
            "EVENT_NA", "EVENT_WAIT", "EVENT_TARGETWAIT", "EVENT_SPEED", "EVENT_TARGET",
            "EVENT_SNAPTARGET", "EVENT_FOV", "EVENT_SCRIPT", "EVENT_TRIGGER", "EVENT_STOP"
        ];
        event_strs[self.event_type as usize].as_ptr() as *const core::ffi::c_char
    }

    pub fn getParam(&self) -> *const core::ffi::c_char {
        self.paramStr.as_ptr() as *const core::ffi::c_char
    }

    pub fn getTime(&self) -> i64 {
        self.time
    }

    pub fn setTime(&mut self, n: i64) {
        self.time = n;
    }

    pub fn parse(&mut self, text: *const *const core::ffi::c_char) {
        // Declared but not defined in header
    }

    pub fn write(&mut self, file: fileHandle_t, name: *const core::ffi::c_char) {
        // Declared but not defined in header
    }

    pub fn setTriggered(&mut self, b: bool) {
        self.triggered = b;
    }

    pub fn getTriggered(&self) -> bool {
        self.triggered
    }
}

pub struct idCameraDef {
    pub name: idStr,
    pub currentCameraPosition: i32,
    pub lastDirection: idVec3_t,
    pub cameraRunning: bool,
    pub cameraPosition: *mut idCameraPosition,
    pub targetPositions: idList<*mut idCameraPosition>,
    pub events: idList<*mut idCameraEvent>,
    pub fov: idCameraFOV,
    pub activeTarget: i32,
    pub totalTime: f32,
    pub baseTime: f32,
    pub startTime: i64,
    pub cameraEdit: bool,
    pub editMode: bool,
}

impl idCameraDef {
    pub fn new() -> Self {
        let mut def = idCameraDef {
            name: String::new(),
            currentCameraPosition: 0,
            lastDirection: idVec3_t::new(0.0, 0.0, 0.0),
            cameraRunning: false,
            cameraPosition: std::ptr::null_mut(),
            targetPositions: Vec::new(),
            events: Vec::new(),
            fov: idCameraFOV::new(),
            activeTarget: 0,
            totalTime: 0.0,
            baseTime: 30.0,
            startTime: 0,
            cameraEdit: false,
            editMode: false,
        };
        def.clear();
        def
    }

    pub fn clear(&mut self) {
        self.currentCameraPosition = 0;
        self.cameraRunning = false;
        self.lastDirection.Zero();
        self.baseTime = 30.0;
        self.activeTarget = 0;
        self.name = "camera01".to_string();
        self.fov.setFOV(90.0);
        let mut i = 0;
        while i < self.targetPositions.len() {
            unsafe {
                let _ = Box::from_raw(self.targetPositions[i]);
            }
            i += 1;
        }
        let mut i = 0;
        while i < self.events.len() {
            unsafe {
                let _ = Box::from_raw(self.events[i]);
            }
            i += 1;
        }
        unsafe {
            if !self.cameraPosition.is_null() {
                let _ = Box::from_raw(self.cameraPosition);
            }
        }
        self.cameraPosition = std::ptr::null_mut();
        self.events.clear();
        self.targetPositions.clear();
    }

    pub fn startNewCamera(&mut self, position_type: idCameraPositionType) -> *mut idCameraPosition {
        self.clear();
        let new_pos: *mut idCameraPosition = if position_type == idCameraPositionType::SPLINE {
            let sp = Box::new(idSplinePosition::new());
            Box::into_raw(sp) as *mut idCameraPosition
        } else if position_type == idCameraPositionType::INTERPOLATED {
            let ip = Box::new(idInterpolatedPosition::new());
            Box::into_raw(ip) as *mut idCameraPosition
        } else {
            let fp = Box::new(idFixedPosition::new());
            Box::into_raw(fp) as *mut idCameraPosition
        };
        self.cameraPosition = new_pos;
        self.cameraPosition
    }

    pub fn addEvent_type(&mut self, t: idCameraEventType, param: *const core::ffi::c_char, time: i64) {
        // Declared but not defined in header
    }

    pub fn addEvent_ptr(&mut self, event: *mut idCameraEvent) {
        // Declared but not defined in header
    }

    pub fn sortEvents(p1: *const core::ffi::c_void, p2: *const core::ffi::c_void) -> i32 {
        // Declared but not defined in header
        0
    }

    pub fn numEvents(&self) -> i32 {
        self.events.len() as i32
    }

    pub fn getEvent(&self, index: i32) -> *mut idCameraEvent {
        assert!((index >= 0) && (index < self.events.len() as i32));
        self.events[index as usize]
    }

    pub fn parse(&mut self, text: *const *const core::ffi::c_char) {
        // Declared but not defined in header
    }

    pub fn load(&mut self, filename: *const core::ffi::c_char) -> bool {
        // Declared but not defined in header (uses qboolean = bool in C)
        false
    }

    pub fn save(&mut self, filename: *const core::ffi::c_char) {
        // Declared but not defined in header
    }

    pub fn buildCamera(&mut self) {
        // Declared but not defined in header
    }

    //idSplineList *getcameraPosition() {
    //	return &cameraPosition;
    //}

    pub fn newFromType(t: idCameraPositionType) -> *mut idCameraPosition {
        match t {
            idCameraPositionType::FIXED => {
                let fp = Box::new(idFixedPosition::new());
                Box::into_raw(fp) as *mut idCameraPosition
            },
            idCameraPositionType::INTERPOLATED => {
                let ip = Box::new(idInterpolatedPosition::new());
                Box::into_raw(ip) as *mut idCameraPosition
            },
            idCameraPositionType::SPLINE => {
                let sp = Box::new(idSplinePosition::new());
                Box::into_raw(sp) as *mut idCameraPosition
            },
            _ => std::ptr::null_mut(),
        }
    }

    pub fn addTarget(&mut self, name: *const core::ffi::c_char, position_type: idCameraPositionType) {
        // Declared but not defined in header
    }

    pub fn getActiveTarget(&mut self) -> *mut idCameraPosition {
        if self.targetPositions.len() == 0 {
            self.addTarget(std::ptr::null(), idCameraPositionType::FIXED);
        }
        self.targetPositions[self.activeTarget as usize]
    }

    pub fn getActiveTarget_index(&mut self, index: i32) -> *mut idCameraPosition {
        if self.targetPositions.len() == 0 {
            self.addTarget(std::ptr::null(), idCameraPositionType::FIXED);
            return self.targetPositions[0];
        }
        self.targetPositions[index as usize]
    }

    pub fn numTargets(&self) -> i32 {
        self.targetPositions.len() as i32
    }

    pub fn setActiveTargetByName(&mut self, name: *const core::ffi::c_char) {
        let mut i = 0;
        while i < self.targetPositions.len() {
            unsafe {
                if Q_stricmp(name, (*self.targetPositions[i]).getName()) == 0 {
                    self.setActiveTarget(i as i32);
                    return;
                }
            }
            i += 1;
        }
    }

    pub fn setActiveTarget(&mut self, index: i32) {
        assert!((index >= 0) && (index < self.targetPositions.len() as i32));
        self.activeTarget = index;
    }

    pub fn setRunning(&mut self, b: bool) {
        self.cameraRunning = b;
    }

    pub fn setBaseTime(&mut self, f: f32) {
        self.baseTime = f;
    }

    pub fn getBaseTime(&self) -> f32 {
        self.baseTime
    }

    pub fn getTotalTime(&self) -> f32 {
        self.totalTime
    }

    pub fn startCamera(&mut self, t: i64) {
        // Declared but not defined in header
    }

    pub fn stopCamera(&mut self) {
        self.cameraRunning = true;
    }

    pub fn getActiveSegmentInfo(&mut self, segment: i32, origin: *mut idVec3_t, direction: *mut idVec3_t, fv: *mut f32) {
        // Declared but not defined in header
    }

    pub fn getCameraInfo_v(&mut self, time: i64, origin: *mut idVec3_t, direction: *mut idVec3_t, fv: *mut f32) -> bool {
        // Declared but not defined in header
        false
    }

    pub fn getCameraInfo_f(&mut self, time: i64, origin: *mut f32, direction: *mut f32, fv: *mut f32) -> bool {
        unsafe {
            let mut org = idVec3_t::new(0.0, 0.0, 0.0);
            let mut dir = idVec3_t::new(0.0, 0.0, 0.0);
            org.data[0] = *origin.offset(0);
            org.data[1] = *origin.offset(1);
            org.data[2] = *origin.offset(2);
            dir.data[0] = *direction.offset(0);
            dir.data[1] = *direction.offset(1);
            dir.data[2] = *direction.offset(2);
            let b = self.getCameraInfo_v(time, &mut org as *mut idVec3_t, &mut dir as *mut idVec3_t, fv);
            *origin.offset(0) = org.data[0];
            *origin.offset(1) = org.data[1];
            *origin.offset(2) = org.data[2];
            *direction.offset(0) = dir.data[0];
            *direction.offset(1) = dir.data[1];
            *direction.offset(2) = dir.data[2];
            b
        }
    }

    pub fn draw(&mut self, editMode: bool) {
        // gcc doesn't allow casting away from bools
        // why?  I've no idea...
        unsafe {
            if !self.cameraPosition.is_null() {
                let cam_pos = &mut *self.cameraPosition;
                cam_pos.draw_bool((editMode || self.cameraRunning) && self.cameraEdit);
                let count = self.targetPositions.len() as i32;
                let mut i = 0;
                while i < count {
                    let target_pos = &mut *self.targetPositions[i as usize];
                    target_pos.draw_bool((editMode || self.cameraRunning) && i == self.activeTarget && !self.cameraEdit);
                    i += 1;
                }
            }
        }
    }

    /*
    pub fn numSegments(&self) -> i32 {
        if self.cameraEdit {
            return self.cameraPosition.numSegments();
        }
        return self.getTargetSpline()->numSegments();
    }

    pub fn getActiveSegment(&self) -> i32 {
        if self.cameraEdit {
            return self.cameraPosition.getActiveSegment();
        }
        return self.getTargetSpline()->getActiveSegment();
    }

    pub fn setActiveSegment(&mut self, i: i32) {
        if self.cameraEdit {
            self.cameraPosition.setActiveSegment(i);
        } else {
            self.getTargetSpline()->setActiveSegment(i);
        }
    }
    */

    pub fn numPoints(&self) -> i32 {
        unsafe {
            if self.cameraEdit {
                return (*self.cameraPosition).numPoints();
            }
            return (*self.getActiveTarget()).numPoints();
        }
    }

    pub fn getPoint(&self, index: i32) -> *const idVec3_t {
        unsafe {
            if self.cameraEdit {
                return (*self.cameraPosition).getPoint(index);
            }
            return (*self.getActiveTarget()).getPoint(index);
        }
    }

    pub fn stopEdit(&mut self) {
        self.editMode = false;
        unsafe {
            if self.cameraEdit {
                (*self.cameraPosition).stopEdit();
            } else {
                (*self.getActiveTarget()).stopEdit();
            }
        }
    }

    pub fn startEdit(&mut self, camera: bool) {
        self.cameraEdit = camera;
        unsafe {
            if camera {
                (*self.cameraPosition).startEdit();
                let mut i = 0;
                while i < self.targetPositions.len() {
                    (*self.targetPositions[i]).stopEdit();
                    i += 1;
                }
            } else {
                (*self.getActiveTarget()).startEdit();
                (*self.cameraPosition).stopEdit();
            }
        }
        self.editMode = true;
    }

    pub fn waitEvent(&mut self, index: i32) -> bool {
        // Declared but not defined in header
        false
    }

    pub fn getName(&self) -> *const core::ffi::c_char {
        self.name.as_ptr() as *const core::ffi::c_char
    }

    pub fn setName(&mut self, p: &str) {
        self.name = p.to_string();
    }

    pub fn getPositionObj(&mut self) -> *mut idCameraPosition {
        unsafe {
            if self.cameraPosition.is_null() {
                let fp = Box::new(idFixedPosition::new());
                self.cameraPosition = Box::into_raw(fp) as *mut idCameraPosition;
            }
            self.cameraPosition
        }
    }
}

pub static mut g_splineMode: bool = false;

pub static mut g_splineList: *mut idCameraDef = std::ptr::null_mut();
