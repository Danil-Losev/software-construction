use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

use super::system::System;

type PipelineRunner = fn(&std::path::Path);
static REGISTRY: OnceLock<RwLock<HashMap<&'static str, PipelineRunner>>> = OnceLock::new();

pub fn get_registry() -> &'static RwLock<HashMap<&'static str, PipelineRunner>> {
    REGISTRY.get_or_init(|| RwLock::new(HashMap::new()))
}

pub fn register_system<SystemType: System>() {
    if let Ok(mut map) = get_registry().write() {
        map.insert(SystemType::NAME, super::pipeline::run::<SystemType>);
    } else {
        eprintln!("Failed to insert system '{}' into registry: could not acquire write lock.", SystemType::NAME);
    }
}
