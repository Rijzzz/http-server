use http_server::ServerStats;

#[test]
fn test_stats_counters_and_json_format() {
    let stats = ServerStats::new();

    stats.inc_conn();
    stats.inc_req();
    stats.add_bytes_in(1024);
    stats.add_bytes_out(2048);
    stats.inc_ws_rx();
    stats.inc_ws_tx();

    let json = stats.to_json();

    assert!(json.contains("\"active_connections\":1"));
    assert!(json.contains("\"total_requests\":1"));
    assert!(json.contains("\"bytes_received\":1024"));
    assert!(json.contains("\"bytes_sent\":2048"));
    assert!(json.contains("\"ws_frames_in\":1"));
    assert!(json.contains("\"ws_frames_out\":1"));

    stats.dec_conn();
    let json_after = stats.to_json();
    assert!(json_after.contains("\"active_connections\":0"));
}
