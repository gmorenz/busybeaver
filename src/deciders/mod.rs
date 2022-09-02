/// Deciders, returns true if the machine doesn't halt, and
/// false if we don't know. Panics with information on the machine
/// if the machine does halt (since that would be an unexpected result)


pub mod cyclers;
pub mod translated_cyclers;
