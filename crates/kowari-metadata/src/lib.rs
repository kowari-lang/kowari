//! Module interface metadata (serialization).

pub fn serialize<T>(_item: &T) -> Vec<u8> {
    unimplemented!("metadata serialization not implemented yet")
}

pub fn deserialize<T>(_bytes: &[u8]) -> Option<T> {
    unimplemented!("metadata deserialization not implemented yet")
}
