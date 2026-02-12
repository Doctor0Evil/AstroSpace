use crate::load_view::{LoadDiagnostics, LoadFlags};

pub trait EpochLogSink {
    fn log_load_view(
        &mut self,
        epoch_index: u64,
        diag: &LoadDiagnostics,
        flags: &LoadFlags,
    );
    // existing methods: log_tree_of_life, log_envelopes, log_deed_event, ...
}
