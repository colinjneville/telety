use crate::Command;

pub mod v0;

#[cfg(feature = "v1")]
pub mod v1;

pub(crate) const VERSIONS: &[(usize, &[Command])] = &[
    (v0::VERSION, v0::COMMANDS),
    #[cfg(feature = "v1")]
    (v1::VERSION, v1::COMMANDS),
];
