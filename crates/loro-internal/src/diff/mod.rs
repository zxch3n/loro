pub mod diff_impl;
pub(crate) use diff_impl::myers_diff;
pub(crate) use diff_impl::DiffHandler;
pub(crate) use diff_impl::OperateProxy;