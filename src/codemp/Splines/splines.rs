
//#[allow(non_snake_case)]
//#[allow(unused_imports)]

use core::ffi::{c_char, c_int, c_void};

// External file system declarations
extern "C" {
    fn FS_Write(buffer: *const c_void, len: c_int, h: c_int) -> c_int;
    fn FS_ReadFile(qpath: *const c_char, buffer: *mut *mut c_void) -> c_int;
    fn FS_FreeFile(buffer: *mut c_void);
    fn FS_FOpenFileWrite(filename: *const c_char) -> c_int;
    fn FS_FCloseFile(f: c_int);
}

// Stub types for unported dependencies - placeholder declarations
// These would be replaced with actual types from imported modules
// in the full port
#[repr(C)]
pub struct idVec3_t {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[repr(C)]
pub struct vec4_t {
    data: [f32; 4],
}

#[repr(C)]
pub struct idStr {
    // Placeholder - real implementation would be in imported module
    ptr: *const c_char,
    len: usize,
}

#[repr(C)]
pub struct idList<T> {
    // Placeholder - real implementation would be in imported module
    _phantom: core::marker::PhantomData<T>,
}

// Type alias for file handles
pub type fileHandle_t = c_int;

// External C functions for GL operations
extern "C" {
    fn qglColor3fv(v: *const f32);
    fn qglPointSize(size: f32);
    fn qglBegin(mode: c_int);
    fn qglVertex3fv(v: *const f32);
    fn qglEnd();
    fn qglRasterPos3fv(v: *const f32);
    fn qglCallLists(n: c_int, type_: c_int, lists: *const c_void);
    fn strlen(s: *const c_char) -> usize;
    fn qglVertex3f(x: f32, y: f32, z: f32);
    fn qglLineWidth(width: f32);
}

// External C functions for parsing and utilities
extern "C" {
    fn Com_Printf(fmt: *const c_char, ...);
    fn Com_Parse(text: *mut *const c_char) -> *const c_char;
    fn Com_UngetToken();
    fn Com_ParseOnLine(text: *mut *const c_char) -> idStr;
    fn Com_Parse1DMatrix(text: *mut *const c_char, size: c_int, out: *mut f32);
    fn Com_BeginParseSession(filename: *const c_char);
    fn Com_EndParseSession();
    fn Com_MatchToken(text: *mut *const c_char, token: *const c_char);
    fn Com_ParseFloat(text: *mut *const c_char) -> f32;
    fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
}

// Extern functions for unported C utilities
extern "C" {
    fn VectorCopy(src: *const f32, dst: *mut f32);
    fn VectorAdd(a: *const f32, b: *const f32, out: *mut f32);
    fn pow(x: f64, y: f64) -> f64;
    fn atan2(y: f64, x: f64) -> f64;
    fn asin(x: f64) -> f64;
    fn atof(s: *const c_char) -> f64;
    fn atoi(s: *const c_char) -> c_int;
    fn atol(s: *const c_char) -> i64;
    fn va(fmt: *const c_char, ...) -> *const c_char;
}

const GL_POINTS: c_int = 0;
const GL_LINES: c_int = 1;
const GL_LINE_STRIP: c_int = 3;
const GL_LINE_LOOP: c_int = 2;
const GL_UNSIGNED_BYTE: c_int = 0x1401;

pub type qboolean = c_int;
const qtrue: qboolean = 1;
const qfalse: qboolean = 0;

#[inline]
fn Q_fabs(f: f32) -> f32 {
    unsafe {
        let tmp = *((&f) as *const f32 as *const c_int);
        let tmp = tmp & 0x7FFFFFFF;
        *((&tmp) as *const c_int as *const f32)
    }
}

//Global variables
pub static mut splineList: idCameraDef = idCameraDef {
    name: idStr { ptr: std::ptr::null(), len: 0 },
    currentCameraPosition: 0,
    lastDirection: idVec3_t { x: 0.0, y: 0.0, z: 0.0 },
    cameraRunning: false,
    cameraPosition: std::ptr::null_mut(),
    targetPositions: idList { _phantom: core::marker::PhantomData },
    events: idList { _phantom: core::marker::PhantomData },
    fov: idCameraFOV { fov: 0.0, startFOV: 0.0, endFOV: 0.0, startTime: 0, time: 0 },
    activeTarget: 0,
    totalTime: 0.0,
    baseTime: 0.0,
    startTime: 0,
    cameraEdit: false,
    editMode: false,
};

pub static mut g_splineList: *mut idCameraDef = unsafe { &mut splineList };

//static idVec3_t idSplineList::zero(0,0,0);
pub static idSplineList_zero: idVec3_t = idVec3_t { x: 0.0, y: 0.0, z: 0.0 };

#[no_mangle]
pub unsafe extern "C" fn glLabeledPoint(color: &idVec3_t, point: &idVec3_t, size: f32, label: *const c_char) {
    qglColor3fv(&color.x);
    qglPointSize(size);
    qglBegin(GL_POINTS);
    qglVertex3fv(&point.x);
    qglEnd();
    let mut v = *point;
    v.x += 1.0;
    v.y += 1.0;
    v.z += 1.0;
    qglRasterPos3fv(&v.x);
    qglCallLists(strlen(label) as c_int, GL_UNSIGNED_BYTE, label as *const c_void);
}

#[no_mangle]
pub unsafe extern "C" fn glBox(color: &idVec3_t, point: &idVec3_t, size: f32) {
    let mut mins = *point;
    let mut maxs = *point;
    mins.x -= size;
    mins.y += size;
    mins.z -= size;
    maxs.x += size;
    maxs.y -= size;
    maxs.z += size;
    qglColor3fv(&color.x);
    qglBegin(GL_LINE_LOOP);
    qglVertex3f(mins.x, mins.y, mins.z);
    qglVertex3f(maxs.x, mins.y, mins.z);
    qglVertex3f(maxs.x, maxs.y, mins.z);
    qglVertex3f(mins.x, maxs.y, mins.z);
    qglEnd();
    qglBegin(GL_LINE_LOOP);
    qglVertex3f(mins.x, mins.y, maxs.z);
    qglVertex3f(maxs.x, mins.y, maxs.z);
    qglVertex3f(maxs.x, maxs.y, maxs.z);
    qglVertex3f(mins.x, maxs.y, maxs.z);
    qglEnd();

    qglBegin(GL_LINES);
    qglVertex3f(mins.x, mins.y, mins.z);
    qglVertex3f(mins.x, mins.y, maxs.z);
    qglVertex3f(mins.x, maxs.y, maxs.z);
    qglVertex3f(mins.x, maxs.y, mins.z);
    qglVertex3f(maxs.x, mins.y, mins.z);
    qglVertex3f(maxs.x, mins.y, maxs.z);
    qglVertex3f(maxs.x, maxs.y, maxs.z);
    qglVertex3f(maxs.x, maxs.y, mins.z);
    qglEnd();
}

#[no_mangle]
pub unsafe extern "C" fn splineTest() {
    //g_splineList->load("p:/doom/base/maps/test_base1.camera");
}

#[no_mangle]
pub unsafe extern "C" fn splineDraw() {
    //g_splineList->addToRenderer();
}

//extern void D_DebugLine( const idVec3_t &color, const idVec3_t &start, const idVec3_t &end );

#[no_mangle]
pub unsafe extern "C" fn debugLine(color: &idVec3_t, x: f32, y: f32, z: f32, x2: f32, y2: f32, z2: f32) {
    //idVec3_t from(x, y, z);
    //idVec3_t to(x2, y2, z2);
    //D_DebugLine(color, from, to);
}

// Stub implementations of unported types
// These would normally be in separate modules

#[repr(C)]
pub struct idCameraDef {
    name: idStr,
    currentCameraPosition: c_int,
    lastDirection: idVec3_t,
    cameraRunning: bool,
    cameraPosition: *mut idCameraPosition,
    targetPositions: idList<*mut idCameraPosition>,
    events: idList<*mut idCameraEvent>,
    fov: idCameraFOV,
    activeTarget: c_int,
    totalTime: f32,
    baseTime: f32,
    startTime: i64,
    cameraEdit: bool,
    editMode: bool,
}

#[repr(C)]
pub struct idCameraPosition {
    startTime: i64,
    time: i64,
    type_: c_int,
    name: idStr,
    editMode: bool,
    velocities: idList<*mut idVelocity>,
    baseVelocity: f32,
}

#[repr(C)]
pub struct idVelocity {
    startTime: i64,
    time: i64,
    speed: f32,
}

#[repr(C)]
pub struct idCameraFOV {
    fov: f32,
    startFOV: f32,
    endFOV: f32,
    startTime: c_int,
    time: c_int,
}

#[repr(C)]
pub struct idCameraEvent {
    type_: c_int,
    paramStr: idStr,
    time: i64,
    triggered: bool,
}

#[repr(C)]
pub struct idSplineList {
    name: idStr,
    controlPoints: idList<*mut idVec3_t>,
    splinePoints: idList<*mut idVec3_t>,
    splineTime: idList<f64>,
    selected: *mut idVec3_t,
    pathColor: idVec3_t,
    segmentColor: idVec3_t,
    controlColor: idVec3_t,
    activeColor: idVec3_t,
    granularity: f32,
    editMode: bool,
    dirty: bool,
    activeSegment: c_int,
    baseTime: i64,
    time: i64,
}

#[repr(C)]
pub struct idFixedPosition {
    base: idCameraPosition,
    pos: idVec3_t,
}

#[repr(C)]
pub struct idInterpolatedPosition {
    base: idCameraPosition,
    first: bool,
    startPos: idVec3_t,
    endPos: idVec3_t,
    lastTime: i64,
    distSoFar: f32,
}

#[repr(C)]
pub struct idSplinePosition {
    base: idCameraPosition,
    target: idSplineList,
}

// Implementation stubs - these are placeholders for the full port
impl idSplineList {
    #[no_mangle]
    pub unsafe fn addToRenderer(&mut self) {
        if self.controlPoints.is_empty() {
            return;
        }

        let mut mins: idVec3_t = Default::default();
        let mut maxs: idVec3_t = Default::default();
        let yellow = idVec3_t { x: 1.0, y: 1.0, z: 0.0 };
        let mut white = idVec3_t { x: 1.0, y: 1.0, z: 1.0 };

        // Loop over control points
        // for(i = 0; i < controlPoints.Num(); i++) {
        //     VectorCopy(*controlPoints[i], mins);
        //     VectorCopy(mins, maxs);
        //     ...
        // }
    }

    #[no_mangle]
    pub unsafe fn buildSpline(&mut self) {
        //int start = Sys_Milliseconds();
        //self.clearSpline();
        //for(int i = 3; i < controlPoints.Num(); i++) {
        //    for (float tension = 0.0f; tension < 1.001f; tension += granularity) {
        //        ...
        //    }
        //}
        //dirty = false;
    }

    #[no_mangle]
    pub unsafe fn draw(&mut self, editMode: bool) {
        //let mut i: c_int;
        //let yellow = vec4_t { ... };
        //
        //if (controlPoints.Num() == 0) {
        //    return;
        //}
        //
        //if (dirty) {
        //    buildSpline();
        //}
        //...
    }

    #[no_mangle]
    pub unsafe fn totalDistance(&mut self) -> f32 {
        // if (controlPoints.Num() == 0) {
        //     return 0.0;
        // }
        //
        // if (dirty) {
        //     buildSpline();
        // }
        //
        // let mut dist = 0.0;
        // let mut temp: idVec3_t;
        // let count = splinePoints.Num();
        // for(int i = 1; i < count; i++) {
        //     temp = *splinePoints[i-1];
        //     temp -= *splinePoints[i];
        //     dist += temp.Length();
        // }
        // return dist;
        0.0
    }

    #[no_mangle]
    pub unsafe fn initPosition(&mut self, bt: i64, totalTime: i64) {
        //if (dirty) {
        //    buildSpline();
        //}
        //
        //if (splinePoints.Num() == 0) {
        //    return;
        //}
        //
        //baseTime = bt;
        //time = totalTime;
        //
        //// calc distance to travel ( this will soon be broken into time segments )
        //splineTime.Clear();
        //splineTime.Append(bt);
        //double dist = totalDistance();
        //double distSoFar = 0.0;
        //idVec3_t temp;
        //int count = splinePoints.Num();
        ////for(int i = 2; i < count - 1; i++) {
        //for(int i = 1; i < count; i++) {
        //    temp = *splinePoints[i-1];
        //    temp -= *splinePoints[i];
        //    distSoFar += temp.Length();
        //    double percent = distSoFar / dist;
        //    percent *= totalTime;
        //    splineTime.Append(percent + bt);
        //}
        //assert(splineTime.Num() == splinePoints.Num());
        //activeSegment = 0;
    }

    #[no_mangle]
    pub unsafe fn calcSpline(&self, step: c_int, tension: f32) -> f32 {
        match step {
            0 => (pow(1.0 - tension as f64, 3.0) as f32) / 6.0,
            1 => ((3.0 * pow(tension as f64, 3.0) - 6.0 * pow(tension as f64, 2.0) + 4.0) as f32) / 6.0,
            2 => ((-3.0 * pow(tension as f64, 3.0) + 3.0 * pow(tension as f64, 2.0) + 3.0 * tension as f64 + 1.0) as f32) / 6.0,
            3 => (pow(tension as f64, 3.0) as f32) / 6.0,
            _ => 0.0,
        }
    }

    #[no_mangle]
    pub unsafe fn updateSelection(&mut self, move_: &idVec3_t) {
        if !self.selected.is_null() {
            self.dirty = true;
            // VectorAdd(*selected, move, *selected);
        }
    }

    #[no_mangle]
    pub unsafe fn setSelectedPoint(&mut self, p: *const idVec3_t) {
        if !p.is_null() {
            // p->Snap();
            // for(int i = 0; i < controlPoints.Num(); i++) {
            //     if (*p == *controlPoints[i]) {
            //         selected = controlPoints[i];
            //     }
            // }
        } else {
            self.selected = std::ptr::null_mut();
        }
    }

    #[no_mangle]
    pub unsafe fn getPosition(&mut self, t: i64) -> *const idVec3_t {
        //static idVec3_t interpolatedPos;
        ////static long lastTime = -1;
        //
        //int count = splineTime.Num();
        //if (count == 0) {
        //    return &zero;
        //}
        //
        //Com_Printf("Time: %d\n", t);
        //assert(splineTime.Num() == splinePoints.Num());
        //
        //while (activeSegment < count) {
        //    if (splineTime[activeSegment] >= t) {
        //        if (activeSegment > 0 && activeSegment < count - 1) {
        //            double timeHi = splineTime[activeSegment + 1];
        //            double timeLo = splineTime[activeSegment - 1];
        //            double percent = (timeHi - t) / (timeHi - timeLo);
        //            // pick two bounding points
        //            idVec3_t v1 = *splinePoints[activeSegment-1];
        //            idVec3_t v2 = *splinePoints[activeSegment+1];
        //            v2 *= (1.0 - percent);
        //            v1 *= percent;
        //            v2 += v1;
        //            interpolatedPos = v2;
        //            return &interpolatedPos;
        //        }
        //        return splinePoints[activeSegment];
        //    } else {
        //        activeSegment++;
        //    }
        //}
        //return splinePoints[count-1];
        std::ptr::null()
    }

    #[no_mangle]
    pub unsafe fn parse(&mut self, text: *mut *const c_char) {
        //const char *token;
        ////Com_MatchToken( text, "{" );
        //do {
        //    token = Com_Parse( text );
        //
        //    if ( !token[0] ) {
        //        break;
        //    }
        //    if ( !Q_stricmp (token, "}") ) {
        //        break;
        //    }
        //
        //    do {
        //        // if token is not a brace, it is a key for a key/value pair
        //        if ( !token[0] || !Q_stricmp (token, "(") || !Q_stricmp(token, "}")) {
        //            break;
        //        }
        //
        //        Com_UngetToken();
        //        idStr key = Com_ParseOnLine(text);
        //        const char *token = Com_Parse(text);
        //        if (Q_stricmp(key.c_str(), "granularity") == 0) {
        //            granularity = atof(token);
        //        } else if (Q_stricmp(key.c_str(), "name") == 0) {
        //            name = token;
        //        }
        //        token = Com_Parse(text);
        //
        //    } while (1);
        //
        //    if ( !Q_stricmp (token, "}") ) {
        //        break;
        //    }
        //
        //    Com_UngetToken();
        //    // read the control point
        //    idVec3_t point;
        //    Com_Parse1DMatrix( text, 3, point );
        //    addPoint(point.x, point.y, point.z);
        //} while (1);
        //
        // //Com_UngetToken();
        // //Com_MatchToken( text, "}" );
        // dirty = true;
    }

    #[no_mangle]
    pub unsafe fn write(&self, file: fileHandle_t, p: *const c_char) {
        //idStr s = va("\t\t%s {\n", p);
        //FS_Write(s.c_str(), s.length(), file);
        ////s = va("\t\tname %s\n", name.c_str());
        ////FS_Write(s.c_str(), s.length(), file);
        //s = va("\t\t\tgranularity %f\n", granularity);
        //FS_Write(s.c_str(), s.length(), file);
        //int count = controlPoints.Num();
        //for (int i = 0; i < count; i++) {
        //    s = va("\t\t\t( %f %f %f )\n", controlPoints[i]->x, controlPoints[i]->y, controlPoints[i]->z);
        //    FS_Write(s.c_str(), s.length(), file);
        //}
        //s = "\t\t}\n";
        //FS_Write(s.c_str(), s.length(), file);
    }
}

impl idCameraDef {
    #[no_mangle]
    pub unsafe fn getActiveSegmentInfo(&self, segment: c_int, origin: &mut idVec3_t, direction: &mut idVec3_t, fov: *mut f32) {
        //#if 0
        //    if (!cameraSpline.validTime()) {
        //        buildCamera();
        //    }
        //    double d = (double)segment / numSegments();
        //    getCameraInfo(d * totalTime * 1000, origin, direction, fov);
        //#endif
        ///*
        //    if (!cameraSpline.validTime()) {
        //        buildCamera();
        //    }
        //    origin = *cameraSpline.getSegmentPoint(segment);
        //
        //
        //    idVec3_t temp;
        //
        //    int numTargets = getTargetSpline()->controlPoints.Num();
        //    int count = cameraSpline.splineTime.Num();
        //    if (numTargets == 0) {
        //        // follow the path
        //        if (cameraSpline.getActiveSegment() < count - 1) {
        //            temp = *cameraSpline.splinePoints[cameraSpline.getActiveSegment()+1];
        //        }
        //    } else if (numTargets == 1) {
        //        temp = *getTargetSpline()->controlPoints[0];
        //    } else {
        //        temp = *getTargetSpline()->getSegmentPoint(segment);
        //    }
        //
        //    temp -= origin;
        //    temp.Normalize();
        //    direction = temp;
        //*/
    }

    #[no_mangle]
    pub unsafe fn getCameraInfo(&self, time: i64, origin: &mut idVec3_t, direction: &mut idVec3_t, fv: *mut f32) -> bool {
        //if ((time - startTime) / 1000 > totalTime) {
        //    return false;
        //}
        //
        //for (int i = 0; i < events.Num(); i++) {
        //    if (time >= startTime + events[i]->getTime() && !events[i]->getTriggered()) {
        //        events[i]->setTriggered(true);
        //        if (events[i]->getType() == idCameraEvent::EVENT_TARGET) {
        //            setActiveTargetByName(events[i]->getParam());
        //            getActiveTarget()->start(startTime + events[i]->getTime());
        //            //Com_Printf("Triggered event switch to target: %s\n",events[i]->getParam());
        //        } else if (events[i]->getType() == idCameraEvent::EVENT_TRIGGER) {
        //            //idEntity *ent = NULL;
        //            //ent = level.FindTarget( ent, events[i]->getParam());
        //            //if (ent) {
        //            //    ent->signal( SIG_TRIGGER );
        //            //    ent->ProcessEvent( &EV_Activate, world );
        //            //}
        //        } else if (events[i]->getType() == idCameraEvent::EVENT_FOV) {
        //            //*fv = fov = atof(events[i]->getParam());
        //        } else if (events[i]->getType() == idCameraEvent::EVENT_STOP) {
        //            return false;
        //        }
        //    }
        //}
        //
        //origin = *cameraPosition->getPosition(time);
        //
        //*fv = fov.getFOV(time);
        //
        //idVec3_t temp = origin;
        //
        //int numTargets = targetPositions.Num();
        //if (numTargets == 0) {
        ///*
        //    // follow the path
        //    if (cameraSpline.getActiveSegment() < count - 1) {
        //        temp = *cameraSpline.splinePoints[cameraSpline.getActiveSegment()+1];
        //        if (temp == origin) {
        //            int index = cameraSpline.getActiveSegment() + 2;
        //            while (temp == origin && index < count - 1) {
        //                temp = *cameraSpline.splinePoints[index++];
        //            }
        //        }
        //    }
        //*/
        //} else {
        //    temp = *getActiveTarget()->getPosition(time);
        //}
        //
        //temp -= origin;
        //temp.Normalize();
        //direction = temp;
        //
        //return true;
        false
    }

    #[no_mangle]
    pub unsafe fn waitEvent(&self, index: c_int) -> bool {
        ////for (int i = 0; i < events.Num(); i++) {
        ////    if (events[i]->getSegment() == index && events[i]->getType() == idCameraEvent::EVENT_WAIT) {
        ////        return true;
        ////    }
        ////}
        false
    }

    #[no_mangle]
    pub unsafe fn buildCamera(&mut self) {
        //int i;
        ////int lastSwitch = 0;
        //idList<float> waits;
        //idList<int> targets;
        //
        //totalTime = baseTime;
        //cameraPosition->setTime(totalTime * 1000);
        //// we have a base time layout for the path and the target path
        //// now we need to layer on any wait or speed changes
        //for (i = 0; i < events.Num(); i++) {
        //    //idCameraEvent *ev = events[i];
        //    events[i]->setTriggered(false);
        //    switch (events[i]->getType()) {
        //        case idCameraEvent::EVENT_TARGET : {
        //            targets.Append(i);
        //            break;
        //        }
        //        case idCameraEvent::EVENT_WAIT : {
        //            waits.Append(atof(events[i]->getParam()));
        //            cameraPosition->addVelocity(events[i]->getTime(), atof(events[i]->getParam()) * 1000, 0);
        //            break;
        //        }
        //        case idCameraEvent::EVENT_TARGETWAIT : {
        //            //targetWaits.Append(i);
        //            break;
        //        }
        //        case idCameraEvent::EVENT_SPEED : {
        ///*
        //            // take the average delay between up to the next five segments
        //            float adjust = atof(events[i]->getParam());
        //            int index = events[i]->getSegment();
        //            total = 0;
        //            count = 0;
        //
        //            // get total amount of time over the remainder of the segment
        //            for (j = index; j < cameraSpline.numSegments() - 1; j++) {
        //                total += cameraSpline.getSegmentTime(j + 1) - cameraSpline.getSegmentTime(j);
        //                count++;
        //            }
        //
        //            // multiply that by the adjustment
        //            double newTotal = total * adjust;
        //            // what is the difference..
        //            newTotal -= total;
        //            totalTime += newTotal / 1000;
        //
        //            // per segment difference
        //            newTotal /= count;
        //            int additive = newTotal;
        //
        //            // now propogate that difference out to each segment
        //            for (j = index; j < cameraSpline.numSegments(); j++) {
        //                cameraSpline.addSegmentTime(j, additive);
        //                additive += newTotal;
        //            }
        //            break;
        //*/
        //        }
        //default: break; // FIXME: what about other idCameraEvent?
        //    }
        //}
        //
        //
        //for (i = 0; i < waits.Num(); i++) {
        //    totalTime += waits[i];
        //}
        //
        //// on a new target switch, we need to take time to this point ( since last target switch )
        //// and allocate it across the active target, then reset time to this point
        //long timeSoFar = 0;
        //long total = (int)(totalTime * 1000);
        //for (i = 0; i < targets.Num(); i++) {
        //    long t;
        //    if (i < targets.Num() - 1) {
        //        t = events[targets[i+1]]->getTime();
        //    } else {
        //        t = total - timeSoFar;
        //    }
        //    // t is how much time to use for this target
        //    setActiveTargetByName(events[targets[i]]->getParam());
        //    getActiveTarget()->setTime(t);
        //    timeSoFar += t;
        //}
    }

    #[no_mangle]
    pub unsafe fn startCamera(&mut self, t: i64) {
        //buildCamera();
        //cameraPosition->start(t);
        ////for (int i = 0; i < targetPositions.Num(); i++) {
        ////    targetPositions[i]->
        ////}
        //startTime = t;
        //cameraRunning = true;
    }

    #[no_mangle]
    pub unsafe fn parse(&mut self, text: *mut *const c_char) {
        //const char *token;
        //do {
        //    token = Com_Parse( text );
        //
        //    if ( !token[0] ) {
        //        break;
        //    }
        //    if ( !Q_stricmp (token, "}") ) {
        //        break;
        //    }
        //
        //    if (Q_stricmp(token, "time") == 0) {
        //        baseTime = Com_ParseFloat(text);
        //    }
        //
        //    if (Q_stricmp(token, "camera_fixed") == 0) {
        //        cameraPosition = new idFixedPosition();
        //        cameraPosition->parse(text);
        //    }
        //
        //    if (Q_stricmp(token, "camera_interpolated") == 0) {
        //        cameraPosition = new idInterpolatedPosition();
        //        cameraPosition->parse(text);
        //    }
        //
        //    if (Q_stricmp(token, "camera_spline") == 0) {
        //        cameraPosition = new idSplinePosition();
        //        cameraPosition->parse(text);
        //    }
        //
        //    if (Q_stricmp(token, "target_fixed") == 0) {
        //        idFixedPosition *pos = new idFixedPosition();
        //        pos->parse(text);
        //        targetPositions.Append(pos);
        //    }
        //
        //    if (Q_stricmp(token, "target_interpolated") == 0) {
        //        idInterpolatedPosition *pos = new idInterpolatedPosition();
        //        pos->parse(text);
        //        targetPositions.Append(pos);
        //    }
        //
        //    if (Q_stricmp(token, "target_spline") == 0) {
        //        idSplinePosition *pos = new idSplinePosition();
        //        pos->parse(text);
        //        targetPositions.Append(pos);
        //    }
        //
        //    if (Q_stricmp(token, "fov") == 0) {
        //        fov.parse(text);
        //    }
        //
        //    if (Q_stricmp(token, "event") == 0) {
        //        idCameraEvent *event = new idCameraEvent();
        //        event->parse(text);
        //        addEvent(event);
        //    }
        //
        //
        //} while (1);
        //
        //Com_UngetToken();
        //Com_MatchToken( text, "}" );
    }

    #[no_mangle]
    pub unsafe fn load(&mut self, filename: *const c_char) -> qboolean {
        //char *buf;
        //const char *buf_p;
        ////int length =
        //FS_ReadFile( filename, (void **)&buf );
        //if ( !buf ) {
        //    return qfalse;
        //}
        //
        //clear();
        //Com_BeginParseSession( filename );
        //buf_p = buf;
        //parse(&buf_p);
        //Com_EndParseSession();
        //FS_FreeFile( buf );
        //
        //return qtrue;
        qfalse
    }

    #[no_mangle]
    pub unsafe fn save(&self, filename: *const c_char) {
        //fileHandle_t file = FS_FOpenFileWrite(filename);
        //if (file) {
        //    int i;
        //    idStr s = "cameraPathDef { \n";
        //    FS_Write(s.c_str(), s.length(), file);
        //    s = va("\ttime %f\n", baseTime);
        //    FS_Write(s.c_str(), s.length(), file);
        //
        //    cameraPosition->write(file, va("camera_%s",cameraPosition->typeStr()));
        //
        //    for (i = 0; i < numTargets(); i++) {
        //        targetPositions[i]->write(file, va("target_%s", targetPositions[i]->typeStr()));
        //    }
        //
        //    for (i = 0; i < events.Num(); i++) {
        //        events[i]->write(file, "event");
        //    }
        //
        //    fov.write(file, "fov");
        //
        //    s = "}\n";
        //    FS_Write(s.c_str(), s.length(), file);
        //}
        //FS_FCloseFile(file);
    }

    #[no_mangle]
    pub unsafe fn sortEvents(p1: *const c_void, p2: *const c_void) -> c_int {
        //idCameraEvent *ev1 = (idCameraEvent*)(p1);
        //idCameraEvent *ev2 = (idCameraEvent*)(p2);
        //
        //if (ev1->getTime() > ev2->getTime()) {
        //    return -1;
        //}
        //if (ev1->getTime() < ev2->getTime()) {
        //    return 1;
        //}
        0
    }

    #[no_mangle]
    pub unsafe fn addEvent(&mut self, event: *mut idCameraEvent) {
        //events.Append(event);
        ////events.Sort(&sortEvents);
    }

    #[no_mangle]
    pub unsafe fn addEvent_typed(&mut self, t: c_int, param: *const c_char, time: i64) {
        //addEvent(new idCameraEvent(t, param, time));
        //buildCamera();
    }
}

// Event type strings
pub static eventStr: [*const c_char; 10] = [
    b"NA\0" as *const c_char,
    b"WAIT\0" as *const c_char,
    b"TARGETWAIT\0" as *const c_char,
    b"SPEED\0" as *const c_char,
    b"TARGET\0" as *const c_char,
    b"SNAPTARGET\0" as *const c_char,
    b"FOV\0" as *const c_char,
    b"SCRIPT\0" as *const c_char,
    b"TRIGGER\0" as *const c_char,
    b"STOP\0" as *const c_char,
];

impl idCameraEvent {
    #[no_mangle]
    pub unsafe fn parse(&mut self, text: *mut *const c_char) {
        //const char *token;
        //Com_MatchToken( text, "{" );
        //do {
        //    token = Com_Parse( text );
        //
        //    if ( !token[0] ) {
        //        break;
        //    }
        //    if ( !strcmp (token, "}") ) {
        //        break;
        //    }
        //
        //    // here we may have to jump over brush epairs ( only used in editor )
        //    do {
        //        // if token is not a brace, it is a key for a key/value pair
        //        if ( !token[0] || !strcmp (token, "(") || !strcmp(token, "}")) {
        //            break;
        //        }
        //
        //        Com_UngetToken();
        //        idStr key = Com_ParseOnLine(text);
        //        const char *token = Com_Parse(text);
        //        if (Q_stricmp(key.c_str(), "type") == 0) {
        //            type = static_cast<idCameraEvent::eventType>(atoi(token));
        //        } else if (Q_stricmp(key.c_str(), "param") == 0) {
        //            paramStr = token;
        //        } else if (Q_stricmp(key.c_str(), "time") == 0) {
        //            time = atoi(token);
        //        }
        //        token = Com_Parse(text);
        //
        //    } while (1);
        //
        //    if ( !strcmp (token, "}") ) {
        //        break;
        //    }
        //
        //} while (1);
        //
        //Com_UngetToken();
        //Com_MatchToken( text, "}" );
    }

    #[no_mangle]
    pub unsafe fn write(&self, file: fileHandle_t, name: *const c_char) {
        //idStr s = va("\t%s {\n", name);
        //FS_Write(s.c_str(), s.length(), file);
        //s = va("\t\ttype %d\n", static_cast<int>(type));
        //FS_Write(s.c_str(), s.length(), file);
        //s = va("\t\tparam %s\n", paramStr.c_str());
        //FS_Write(s.c_str(), s.length(), file);
        //s = va("\t\ttime %d\n", time);
        //FS_Write(s.c_str(), s.length(), file);
        //s = "\t}\n";
        //FS_Write(s.c_str(), s.length(), file);
    }
}

// Position type strings
pub static positionStr: [*const c_char; 3] = [
    b"Fixed\0" as *const c_char,
    b"Interpolated\0" as *const c_char,
    b"Spline\0" as *const c_char,
];

impl idInterpolatedPosition {
    #[no_mangle]
    pub unsafe fn getPosition(&mut self, t: i64) -> *const idVec3_t {
        //static idVec3_t interpolatedPos;
        //
        //float velocity = getVelocity(t);
        //float timePassed = t - lastTime;
        //lastTime = t;
        //
        //// convert to seconds
        //timePassed /= 1000;
        //
        //float distToTravel = timePassed *= velocity;
        //
        //idVec3_t temp = startPos;
        //temp -= endPos;
        //float distance = temp.Length();
        //
        //distSoFar += distToTravel;
        //float percent = (float)(distSoFar) / distance;
        //
        //if (percent > 1.0) {
        //    percent = 1.0;
        //} else if (percent < 0.0) {
        //    percent = 0.0;
        //}
        //
        //// the following line does a straigt calc on percentage of time
        //// float percent = (float)(startTime + time - t) / time;
        //
        //idVec3_t v1 = startPos;
        //idVec3_t v2 = endPos;
        //v1 *= (1.0 - percent);
        //v2 *= percent;
        //v1 += v2;
        //interpolatedPos = v1;
        //return &interpolatedPos;
        std::ptr::null()
    }
}

impl idCameraFOV {
    #[no_mangle]
    pub unsafe fn parse(&mut self, text: *mut *const c_char) {
        //const char *token;
        //Com_MatchToken( text, "{" );
        //do {
        //    token = Com_Parse( text );
        //
        //    if ( !token[0] ) {
        //        break;
        //    }
        //    if ( !strcmp (token, "}") ) {
        //        break;
        //    }
        //
        //    // here we may have to jump over brush epairs ( only used in editor )
        //    do {
        //        // if token is not a brace, it is a key for a key/value pair
        //        if ( !token[0] || !strcmp (token, "(") || !strcmp(token, "}")) {
        //            break;
        //        }
        //
        //        Com_UngetToken();
        //        idStr key = Com_ParseOnLine(text);
        //        const char *token = Com_Parse(text);
        //        if (Q_stricmp(key.c_str(), "fov") == 0) {
        //            fov = atof(token);
        //        } else if (Q_stricmp(key.c_str(), "startFOV") == 0) {
        //            startFOV = atof(token);
        //        } else if (Q_stricmp(key.c_str(), "endFOV") == 0) {
        //            endFOV = atof(token);
        //        } else if (Q_stricmp(key.c_str(), "time") == 0) {
        //            time = atoi(token);
        //        }
        //        token = Com_Parse(text);
        //
        //    } while (1);
        //
        //    if ( !strcmp (token, "}") ) {
        //        break;
        //    }
        //
        //} while (1);
        //
        //Com_UngetToken();
        //Com_MatchToken( text, "}" );
    }

    #[no_mangle]
    pub unsafe fn write(&self, file: fileHandle_t, p: *const c_char) {
        //idStr s = va("\t%s {\n", p);
        //FS_Write(s.c_str(), s.length(), file);
        //
        //s = va("\t\tfov %f\n", fov);
        //FS_Write(s.c_str(), s.length(), file);
        //
        //s = va("\t\tstartFOV %f\n", startFOV);
        //FS_Write(s.c_str(), s.length(), file);
        //
        //s = va("\t\tendFOV %f\n", endFOV);
        //FS_Write(s.c_str(), s.length(), file);
        //
        //s = va("\t\ttime %i\n", time);
        //FS_Write(s.c_str(), s.length(), file);
        //
        //s = "\t}\n";
        //FS_Write(s.c_str(), s.length(), file);
    }
}

impl idCameraPosition {
    #[no_mangle]
    pub unsafe fn parseToken(&self, key: *const c_char, text: *mut *const c_char) -> bool {
        //const char *token = Com_Parse(text);
        //if (Q_stricmp(key, "time") == 0) {
        //    time = atol(token);
        //    return true;
        //} else if (Q_stricmp(key, "type") == 0) {
        //    type = static_cast<idCameraPosition::positionType>(atoi(token));
        //    return true;
        //} else if (Q_stricmp(key, "velocity") == 0) {
        //    long t = atol(token);
        //    token = Com_Parse(text);
        //    long d = atol(token);
        //    token = Com_Parse(text);
        //    float s = atof(token);
        //    addVelocity(t, d, s);
        //    return true;
        //} else if (Q_stricmp(key, "baseVelocity") == 0) {
        //    baseVelocity = atof(token);
        //    return true;
        //} else if (Q_stricmp(key, "name") == 0) {
        //    name = token;
        //    return true;
        //} else if (Q_stricmp(key, "time") == 0) {
        //    time = atoi(token);
        //    return true;
        //}
        //Com_UngetToken();
        false
    }

    #[no_mangle]
    pub unsafe fn write(&self, file: fileHandle_t, p: *const c_char) {
        //idStr s = va("\t\ttime %i\n", time);
        //FS_Write(s.c_str(), s.length(), file);
        //
        //s = va("\t\ttype %i\n", static_cast<int>(type));
        //FS_Write(s.c_str(), s.length(), file);
        //
        //s = va("\t\tname %s\n", name.c_str());
        //FS_Write(s.c_str(), s.length(), file);
        //
        //s = va("\t\tbaseVelocity %f\n", baseVelocity);
        //FS_Write(s.c_str(), s.length(), file);
        //
        //for (int i = 0; i < velocities.Num(); i++) {
        //    s = va("\t\tvelocity %i %i %f\n", velocities[i]->startTime, velocities[i]->time, velocities[i]->speed);
        //    FS_Write(s.c_str(), s.length(), file);
        //}
    }
}

impl idFixedPosition {
    #[no_mangle]
    pub unsafe fn parse(&mut self, text: *mut *const c_char) {
        //const char *token;
        //Com_MatchToken( text, "{" );
        //do {
        //    token = Com_Parse( text );
        //
        //    if ( !token[0] ) {
        //        break;
        //    }
        //    if ( !strcmp (token, "}") ) {
        //        break;
        //    }
        //
        //    // here we may have to jump over brush epairs ( only used in editor )
        //    do {
        //        // if token is not a brace, it is a key for a key/value pair
        //        if ( !token[0] || !strcmp (token, "(") || !strcmp(token, "}")) {
        //            break;
        //        }
        //
        //        Com_UngetToken();
        //        idStr key = Com_ParseOnLine(text);
        //
        //        const char *token = Com_Parse(text);
        //        if (Q_stricmp(key.c_str(), "pos") == 0) {
        //            Com_UngetToken();
        //            Com_Parse1DMatrix( text, 3, pos );
        //        } else {
        //            Com_UngetToken();
        //            idCameraPosition::parseToken(key.c_str(), text);
        //        }
        //        token = Com_Parse(text);
        //
        //    } while (1);
        //
        //    if ( !strcmp (token, "}") ) {
        //        break;
        //    }
        //
        //} while (1);
        //
        //Com_UngetToken();
        //Com_MatchToken( text, "}" );
    }

    #[no_mangle]
    pub unsafe fn write(&self, file: fileHandle_t, p: *const c_char) {
        //idStr s = va("\t%s {\n", p);
        //FS_Write(s.c_str(), s.length(), file);
        //idCameraPosition::write(file, p);
        //s = va("\t\tpos ( %f %f %f )\n", pos.x, pos.y, pos.z);
        //FS_Write(s.c_str(), s.length(), file);
        //s = "\t}\n";
        //FS_Write(s.c_str(), s.length(), file);
    }
}

impl idInterpolatedPosition {
    #[no_mangle]
    pub unsafe fn parse(&mut self, text: *mut *const c_char) {
        //const char *token;
        //Com_MatchToken( text, "{" );
        //do {
        //    token = Com_Parse( text );
        //
        //    if ( !token[0] ) {
        //        break;
        //    }
        //    if ( !strcmp (token, "}") ) {
        //        break;
        //    }
        //
        //    // here we may have to jump over brush epairs ( only used in editor )
        //    do {
        //        // if token is not a brace, it is a key for a key/value pair
        //        if ( !token[0] || !strcmp (token, "(") || !strcmp(token, "}")) {
        //            break;
        //        }
        //
        //        Com_UngetToken();
        //        idStr key = Com_ParseOnLine(text);
        //
        //        const char *token = Com_Parse(text);
        //        if (Q_stricmp(key.c_str(), "startPos") == 0) {
        //            Com_UngetToken();
        //            Com_Parse1DMatrix( text, 3, startPos );
        //        } else if (Q_stricmp(key.c_str(), "endPos") == 0) {
        //            Com_UngetToken();
        //            Com_Parse1DMatrix( text, 3, endPos );
        //        } else {
        //            Com_UngetToken();
        //            idCameraPosition::parseToken(key.c_str(), text);
        //        }
        //        token = Com_Parse(text);
        //
        //    } while (1);
        //
        //    if ( !strcmp (token, "}") ) {
        //        break;
        //    }
        //
        //} while (1);
        //
        //Com_UngetToken();
        //Com_MatchToken( text, "}" );
    }

    #[no_mangle]
    pub unsafe fn write(&self, file: fileHandle_t, p: *const c_char) {
        //idStr s = va("\t%s {\n", p);
        //FS_Write(s.c_str(), s.length(), file);
        //idCameraPosition::write(file, p);
        //s = va("\t\tstartPos ( %f %f %f )\n", startPos.x, startPos.y, startPos.z);
        //FS_Write(s.c_str(), s.length(), file);
        //s = va("\t\tendPos ( %f %f %f )\n", endPos.x, endPos.y, endPos.z);
        //FS_Write(s.c_str(), s.length(), file);
        //s = "\t}\n";
        //FS_Write(s.c_str(), s.length(), file);
    }
}

impl idSplinePosition {
    #[no_mangle]
    pub unsafe fn parse(&mut self, text: *mut *const c_char) {
        //const char *token;
        //Com_MatchToken( text, "{" );
        //do {
        //    token = Com_Parse( text );
        //
        //    if ( !token[0] ) {
        //        break;
        //    }
        //    if ( !strcmp (token, "}") ) {
        //        break;
        //    }
        //
        //    // here we may have to jump over brush epairs ( only used in editor )
        //    do {
        //        // if token is not a brace, it is a key for a key/value pair
        //        if ( !token[0] || !strcmp (token, "(") || !strcmp(token, "}")) {
        //            break;
        //        }
        //
        //        Com_UngetToken();
        //        idStr key = Com_ParseOnLine(text);
        //
        //        const char *token = Com_Parse(text);
        //        if (Q_stricmp(key.c_str(), "target") == 0) {
        //            target.parse(text);
        //        } else {
        //            Com_UngetToken();
        //            idCameraPosition::parseToken(key.c_str(), text);
        //        }
        //        token = Com_Parse(text);
        //
        //    } while (1);
        //
        //    if ( !strcmp (token, "}") ) {
        //        break;
        //    }
        //
        //} while (1);
        //
        //Com_UngetToken();
        //Com_MatchToken( text, "}" );
    }

    #[no_mangle]
    pub unsafe fn write(&self, file: fileHandle_t, p: *const c_char) {
        //idStr s = va("\t%s {\n", p);
        //FS_Write(s.c_str(), s.length(), file);
        //idCameraPosition::write(file, p);
        //target.write(file, "target");
        //s = "\t}\n";
        //FS_Write(s.c_str(), s.length(), file);
    }
}

impl idCameraDef {
    #[no_mangle]
    pub unsafe fn addTarget(&mut self, name: *const c_char, type_: c_int) {
        ////const char *text = (name == NULL) ? va("target0%d", numTargets()+1) : name; // TTimo: unused
        //idCameraPosition *pos = newFromType(type);
        //if (pos) {
        //    pos->setName(name);
        //    targetPositions.Append(pos);
        //    activeTarget = numTargets()-1;
        //    if (activeTarget == 0) {
        //        // first one
        //        addEvent(idCameraEvent::EVENT_TARGET, name, 0);
        //    }
        //}
    }

    #[no_mangle]
    pub unsafe fn newFromType(t: c_int) -> *mut idCameraPosition {
        //switch (t) {
        //    case idCameraPosition::FIXED : return new idFixedPosition();
        //    case idCameraPosition::INTERPOLATED : return new idInterpolatedPosition();
        //    case idCameraPosition::SPLINE : return new idSplinePosition();
        //    default:
        //        break;
        //};
        std::ptr::null_mut()
    }
}

pub static mut camera: idCameraDef = idCameraDef {
    name: idStr { ptr: std::ptr::null(), len: 0 },
    currentCameraPosition: 0,
    lastDirection: idVec3_t { x: 0.0, y: 0.0, z: 0.0 },
    cameraRunning: false,
    cameraPosition: std::ptr::null_mut(),
    targetPositions: idList { _phantom: core::marker::PhantomData },
    events: idList { _phantom: core::marker::PhantomData },
    fov: idCameraFOV { fov: 0.0, startFOV: 0.0, endFOV: 0.0, startTime: 0, time: 0 },
    activeTarget: 0,
    totalTime: 0.0,
    baseTime: 0.0,
    startTime: 0,
    cameraEdit: false,
    editMode: false,
};

#[no_mangle]
pub unsafe extern "C" fn loadCamera(name: *const c_char) -> qboolean {
    //camera.clear();
    //return static_cast<qboolean>(camera.load(name));
    qfalse
}

#[no_mangle]
pub unsafe extern "C" fn getCameraInfo(time: c_int, origin: *mut f32, angles: *mut f32) -> qboolean {
    //idVec3_t dir, org;
    //org[0] = origin[0];
    //org[1] = origin[1];
    //org[2] = origin[2];
    //float fov = 90;
    //if (camera.getCameraInfo(time, org, dir, &fov)) {
    //    origin[0] = org[0];
    //    origin[1] = org[1];
    //    origin[2] = org[2];
    //    angles[1] = atan2 (dir[1], dir[0])*180/3.14159;
    //    angles[0] = asin (dir[2])*180/3.14159;
    //    return qtrue;
    //}
    qfalse
}

#[no_mangle]
pub unsafe extern "C" fn startCamera(time: c_int) {
    //camera.startCamera(time);
}

impl Default for idVec3_t {
    fn default() -> Self {
        idVec3_t { x: 0.0, y: 0.0, z: 0.0 }
    }
}

impl idList<f32> {
    fn is_empty(&self) -> bool {
        false
    }
}
