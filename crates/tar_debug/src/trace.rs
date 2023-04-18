use std::{ptr, fs, io::Write, time, process};

use serde::ser::{Serialize, SerializeStruct, Serializer};
use parking_lot::Mutex;

static TRACER: Mutex<Tracer> = Mutex::new(Tracer::new());

struct Tracer {
    current: *mut Session
}

impl Tracer {
    pub const fn new() -> Self {
        Self {
            current: ptr::null_mut()
        }
    }
}

unsafe impl Send for Tracer {}


pub struct Session {
    start: time::Instant,
    result: SessionResult,
    path: &'static str
}

impl Session {
    #[inline]
    pub fn new(path: &'static str) -> Box<Self> {
        let mut tracer = TRACER.lock();
        let mut ret = Box::new(Self {
            start: time::Instant::now(),
            result: SessionResult::new(),
            path
        });
        
        if !tracer.current.is_null() {
            unsafe { tracer.current.drop_in_place(); }
        }

        tracer.current = ret.as_mut();

        ret
    }

    #[inline]
    pub fn end(self) {
        drop(self)
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        'work: {
            let Ok(s) = serde_json::to_string(&self.result) else {
                break 'work;
            };

            let Ok(mut f) = fs::File::create(self.path) else {
                break 'work;
            };

            f.write_all(s.as_bytes()).unwrap_or_else(|err| println!("{}", err));
            break 'work;
        }

        TRACER.lock().current = ptr::null_mut();
    }
}


#[derive(Copy, Clone)]
pub enum TraceType {
    Block,
    Function,
    Stmt
}

impl TraceType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Block => "Block",
            Self::Function => "Function",
            Self::Stmt => "Statement"
        }
    }
}


pub struct Trace {
    name: &'static str,
    ts: time::Instant,
    ty: TraceType
}

impl Trace {
    pub fn new(name: &'static str, ty: TraceType) -> Self {
        Self {
            name, ty,
            ts: time::Instant::now(),
        }
    }

    #[inline]
    pub fn end(self) {
        drop(self)
    }
}

impl Drop for Trace {
    fn drop(&mut self) {
        let mut tracer = TRACER.lock();
        
        if tracer.current.is_null() {
            return;
        };

        let session = unsafe { &mut *tracer.current };

        session.result.trace_events.push(TraceEvent {
            name: self.name,
            ts: self.ts.duration_since(session.start).as_micros(),
            dur: self.ts.elapsed().as_micros(),
            pid: process::id(),
            tid: thread_id::get(),
            ty: self.ty
        });
    }
}


struct SessionResult {
    trace_events: Vec<TraceEvent>
}

impl SessionResult {
    #[inline]
    pub const fn new() -> Self {
        Self { trace_events: Vec::new() }
    } 
}

impl Serialize for SessionResult {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("SessionResult", 2)?;
        s.serialize_field("traceEvents", &self.trace_events)?;
        s.serialize_field("displayTimeUnit", &"ms")?;
        s.end()
    } 
}


struct TraceEvent {
    name: &'static str,
    ts: u128,
    dur: u128,
    pid: u32,
    tid: usize,
    ty: TraceType
}

impl Serialize for TraceEvent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("TraceEvent", 7)?;
        s.serialize_field("name", &self.name)?;
        s.serialize_field("cat", self.ty.as_str())?;
        s.serialize_field("ph", &"X")?;
        s.serialize_field("ts", &self.ts)?;
        s.serialize_field("dur", &self.dur)?;
        s.serialize_field("pid", &self.pid)?;
        s.serialize_field("tid", &self.tid)?;
        s.end()
    }
}

