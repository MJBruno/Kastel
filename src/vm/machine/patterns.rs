use crate::{runtime::value::Value, vm::machine::VirtualMachine};
#[allow(dead_code)]
impl VirtualMachine {
    pub(crate) fn pattern_equal(left: &Value, right: &Value) -> bool {
        left == right
    }
}
