use crate::storage;
use std::path::Path;

#[allow(dead_code)]
pub fn run_reconciliation(path: &Path, scope_id: i64) -> Result<(), String> {
    storage::reconcile_subscriptions(path, scope_id)
}
