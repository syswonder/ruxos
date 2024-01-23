use num_enum::TryFromPrimitive;

// TODO: syscall id are architecture-dependent
#[allow(clippy::upper_case_acronyms)]
#[allow(non_camel_case_types)]
#[repr(usize)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromPrimitive)]
pub enum SyscallId {
    INVALID = 999,
}
