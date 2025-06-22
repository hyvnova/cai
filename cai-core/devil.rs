/*
 * "...And in search of a bigger freedom, the man danced with the devil.
 * A dance that would last for eternity"
 * 
 * Hereby I take the devil's hand and dance with him.
 * After here, it's the end, the downfall, the ABSOLUTE CINAME and epitome of this project.
 * Once this python-like abstractions are forcefully forced into the codebase there won't be any way back.
 * This is the point of no return.
 * Where is Alice?
 */

// Tiny, thread-safe, “put it in and pray” global store + macros.
//
// # TL;DR
// * `global_add!("key", value)` – stash **anything** behind a `&'static str`.
// * `global_get!(Type, "key")`   – grab it back (as `Arc<Type>`).  
//   If the key was never registered, **boom** – default panic *or* your custom
//   handler fires.
//
// ## Custom fail handler
// ```rust
// fn bail(k: &'static str) -> ! {
//     eprintln!(">> Missing {k:?}!  So long and thanks for all the bugs.");
//     std::process::exit(1);
// }
//
// global_add!("token", String::from("secret"), bail);
// ```
//
// ## Happy-path usage
// ```rust
// use std::sync::Arc;
// use crate::{global_add, global_get};
//
// global_add!("answer", 42u32);        // default panic on miss
// let answer: Arc<u32> = global_get!(u32, "answer");
// assert_eq!(*answer, 42);
// ```
// ───────────────────────────────────────────────────────────────────────────

use once_cell::sync::Lazy;
use std::{
    any::Any,
    collections::HashMap,
    sync::{Arc, RwLock},
};

type Key        = &'static str;
type AnyBox     = Arc<dyn Any + Send + Sync>;
type NotFoundFn = fn(Key) -> !;

/// One entry = stored value + the “what to do if missing” callback.
struct Entry {
    val: AnyBox,
    on_missing: NotFoundFn,
}

static GLOBALS: Lazy<RwLock<HashMap<Key, Entry>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

pub struct Globals;

impl Globals {
    /// Registers a value and its “missing” handler.
    pub fn add<T>(key: Key, value: T, on_missing: NotFoundFn)
    where
        T: Any + Send + Sync + 'static,
    {
        let mut map = GLOBALS.write().expect("poisoned lock");
        map.insert(
            key,
            Entry {
                val: Arc::new(value),
                on_missing,
            },
        );
    }

    /// Grabs the value or calls the pre-registered handler (and never returns).
    pub fn get<T>(key: Key) -> Arc<T>
    where
        T: Any + Send + Sync + 'static,
    {
        let map = GLOBALS.read().expect("poisoned lock");
        match map.get(key) {
            Some(entry) => entry
                .val
                .clone()
                .downcast::<T>()
                .expect("type mismatch for this key"),
            None => {
                // Unlock before we blow up.
                drop(map);
                Self::explode(key)
            }
        }
    }

    /// Fallback when the *entire* key is missing (no handler registered).
    fn explode(key: Key) -> ! {
        panic!("Global value {key:?} was never initialised");
    }
}


//! globals_macros.rs
//! Re-export these from lib.rs or main.rs with `pub use crate::globals_macros::*;`

#[macro_export]
macro_rules! global_add {
    // ------------- 2-arg form: uses default panic handler -------------------
    ($key:literal, $value:expr) => {{
        $crate::Globals::add($key, $value, |k| {
            panic!("Global {k:?} was requested before being registered")
        })
    }};

    // ------------- 3-arg form: caller supplies custom handler --------------
    ($key:literal, $value:expr, $handler:expr) => {{
        $crate::Globals::add($key, $value, $handler)
    }};
}

#[macro_export]
macro_rules! global_get {
    // `global_get!(Type, "key")`   …returns `Arc<Type>`
    ($ty:ty, $key:literal) => {{
        $crate::Globals::get::<$ty>($key)
    }};
}
