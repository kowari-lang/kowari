//! Wasmtime embedding for the stdlib `wasm` module.

pub struct Instance;

pub fn load(_path: &str) -> Result<Instance, String> {
    unimplemented!("wasm host not implemented yet")
}
