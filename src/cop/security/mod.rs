mod eval;
mod io_methods;
mod json_load;
mod open;

use crate::cop::registry::CopRegistry;

pub fn register_all(registry: &mut CopRegistry) {
    registry.register(Box::new(eval::Eval));
    registry.register(Box::new(io_methods::IoMethods));
    registry.register(Box::new(json_load::JsonLoad));
    registry.register(Box::new(open::Open));
}
