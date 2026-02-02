pub fn capture_all_monitors() -> Result<(Vec<u8>, u32, u32), String> {
    // Redirect to the hot-standby service
    crate::service::capture_service::get_snapshot_fast()
}
