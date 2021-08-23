mod closure;
mod opcode;
mod value;

pub use closure::Closure;
pub use value::{ Value, type_value };
pub use value::BuiltIn;
pub use value::RustFunc;
pub use opcode::{ Opcode, Opmode, OPMODES };