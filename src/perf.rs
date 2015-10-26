// extern crate rustc_serialize;

// use rustc_serialize::{json, Decoder, Decodable};
/*
1 	floating point value
2 	unsigned 64-bit integer value
4 	average (sum + count pair)
8 	counter (vs gauge)
*/
/*
pub struct WBThrottle {
    bytes_dirtied: u64,
    bytes_wb: u64,
    ios_dirtied: u64,
    ios_wb: u64,
    inodes_dirtied: u64,
    inodes_wb: u64
}

pub struct Gauge{
    avgcount: u64,
    sum: u64,
}

pub struct Filestore {
    journal_queue_max_ops: u64,
    journal_queue_ops: u64,
    journal_ops: u64,
    journal_queue_max_bytes: u64,
    journal_queue_bytes: u64,
    journal_bytes: u64,
    journal_latency: {
        avgcount : f64,
        sum : f64},
    journal_wr: u64,
    journal_wr_bytes: {
        avgcount : u64,
        sum : u64},
    journal_full: u64,
    committing: u64,
    commitcycle: u64,
    commitcycle_interval: {Gauge,
    commitcycle_latency: Gauge,
    op_queue_max_ops: u64,
    op_queue_ops: u64,
    ops: u64,
    op_queue_max_bytes: u64,
    op_queue_bytes: u64,
    bytes: u64,
    apply_latency: Gauge,
    queue_transaction_latency_avg: Gauge,
    leveldb: { leveldb_get: u64,
    leveldb_transaction: u64,
    leveldb_compact: u64,
    leveldb_compact_range: u64,
    leveldb_compact_queue_merge: u64,
    leveldb_compact_queue_len: u64},
    mutex-FileJournal::completions_lock: { wait: Gauge,
    mutex-FileJournal::finisher_lock: { wait: Gauge,
    mutex-FileJournal::write_lock: { wait: Gauge,
    mutex-FileJournal::writeq_lock: { wait: Gauge,
    mutex-JOS::ApplyManager::apply_lock: { wait: Gauge,
    mutex-JOS::ApplyManager::com_lock: { wait: Gauge,
    mutex-JOS::SubmitManager::lock: { wait: Gauge,
    mutex-WBThrottle::lock: { wait: Gauge,

pub struct Objecter {
    op_active: u64,
    op_laggy: u64,
    op_send: u64,
    op_send_bytes: u64,
    op_resend: u64,
    op_ack: u64,
    op_commit: u64,
    op: u64,
    op_r: u64,
    op_w: u64,
    op_rmw: u64,
    op_pg: u64,
    osdop_stat: u64,
    osdop_create: u64,
    osdop_read: u64,
    osdop_write: u64,
    osdop_writefull: u64,
    osdop_append: u64,
    osdop_zero: u64,
    osdop_truncate: u64,
    osdop_delete: u64,
    osdop_mapext: u64,
    osdop_sparse_read: u64,
    osdop_clonerange: u64,
    osdop_getxattr: u64,
    osdop_setxattr: u64,
    osdop_cmpxattr: u64,
    osdop_rmxattr: u64,
    osdop_resetxattrs: u64,
    osdop_tmap_up: u64,
    osdop_tmap_put: u64,
    osdop_tmap_get: u64,
    osdop_call: u64,
    osdop_watch: u64,
    osdop_notify: u64,
    osdop_src_cmpxattr: u64,
    osdop_pgls: u64,
    osdop_pgls_filter: u64,
    osdop_other: u64,
    linger_active: u64,
    linger_send: u64,
    linger_resend: u64,
    poolop_active: u64,
    poolop_send: u64,
    poolop_resend: u64,
    poolstat_active: u64,
    poolstat_send: u64,
    poolstat_resend: u64,
    statfs_active: u64,
    statfs_send: u64,
    statfs_resend: u64,
    command_active: u64,
    command_send: u64,
    command_resend: u64,
    map_epoch: u64,
    map_full: u64,
    map_inc: u64,
    osd_sessions: u64,
    osd_session_open: u64,
    osd_session_close: u64,
    osd_laggy: u64
}

pub struct Osd {
    opq: u64,
    op_wip: u64,
    op: u64,
    op_in_bytes: u64,
    op_out_bytes: u64,
    op_latency: Gauge,
    op_process_latency: Gauge,
    op_r: u64,
    op_r_out_bytes: u64,
    op_r_latency: Gauge,
    op_r_process_latency: Gauge,
    op_w: u64,
    op_w_in_bytes: u64,
    op_w_rlat: Gauge,
    op_w_latency: Gauge,
    op_w_process_latency: Gauge,
    op_rw: u64,
    op_rw_in_bytes: u64,
    op_rw_out_bytes: u64,
    op_rw_rlat: Gauge,
    op_rw_latency: Gauge,
    op_rw_process_latency: Gauge,
    subop: u64,
    subop_in_bytes: u64,
    subop_latency: Gauge,
    subop_w: u64,
    subop_w_in_bytes: u64,
    subop_w_latency: Gauge,
    subop_pull: u64,
    subop_pull_latency: Gauge,
    subop_push: u64,
    subop_push_in_bytes: u64,
    subop_push_latency: Gauge,
    pull: u64,
    push: u64,
    push_out_bytes: u64,
    push_in: u64,
    push_in_bytes: u64,
    recovery_ops: u64,
    loadavg: u64,
    buffer_bytes: u64,
    numpg: u64,
    numpg_primary: u64,
    numpg_replica: u64,
    numpg_stray: u64,
    heartbeat_to_peers: u64,
    heartbeat_from_peers: u64,
    map_messages: u64,
    map_message_epochs: u64,
    map_message_epoch_dups: u64,
    messages_delayed_for_map: u64,
    stat_bytes: u64,
    stat_bytes_used: u64,
    stat_bytes_avail: u64,
    copyfrom: u64,
    tier_promote: u64,
    tier_flush: u64,
    tier_flush_fail: u64,
    tier_try_flush: u64,
    tier_try_flush_fail: u64,
    tier_evict: u64,
    tier_whiteout: u64,
    tier_dirty: u64,
    tier_clean: u64,
    tier_delay: u64,
    agent_wake: u64,
    agent_skip: u64,
    agent_flush: u64,
    agent_evict: u64
}

pub struct RecoverystatePerf {
    initial_latency: Gauge,
    started_latency: Gauge,
    reset_latency: Gauge,
    start_latency: Gauge,
    primary_latency: Gauge,
    peering_latency: Gauge,
    backfilling_latency: Gauge,
    waitremotebackfillreserved_latency: Gauge,
    waitlocalbackfillreserved_latency: Gauge,
    notbackfilling_latency: Gauge,
    repnotrecovering_latency: Gauge,
    repwaitrecoveryreserved_latency: Gauge,
    repwaitbackfillreserved_latency: Gauge,
    RepRecovering_latency: Gauge,
    activating_latency: Gauge,
    waitlocalrecoveryreserved_latency: Gauge,
    waitremoterecoveryreserved_latency: Gauge,
    recovering_latency: Gauge,
    recovered_latency: Gauge,
    clean_latency: Gauge,
    active_latency: Gauge,
    replicaactive_latency: Gauge,
    stray_latency: Gauge,
    getinfo_latency: Gauge,
    getlog_latency: Gauge,
    waitactingchange_latency: Gauge,
    incomplete_latency: Gauge,
    getmissing_latency: Gauge,
    waitupthru_latency: Gauge
}

pub struct ThrottleFilestoreBytes {
    val: u64,
    max: u64,
    get: u64,
    get_sum: u64,
    get_or_fail_fail: u64,
    get_or_fail_success: u64,
    take: u64,
    take_sum: u64,
    put: u64,
    put_sum: u64,
    wait: Gauge
}

pub struct ThrottleFilestoreOps {
    val: u64,
    max: u64,
    get: u64,
    get_sum: u64,
    get_or_fail_fail: u64,
    get_or_fail_success: u64,
    take: u64,
    take_sum: u64,
    put: u64,
    put_sum: u64,
    wait: Gauge
}

pub struct ThrottleMsgrDispatchThrottlerClient {
    val: u64,
    max: u64,
    get: u64,
    get_sum: u64,
    get_or_fail_fail: u64,
    get_or_fail_success: u64,
    take: u64,
    take_sum: u64,
    put: u64,
    put_sum: u64,
    wait: Gauge
}

pub struct ThrottleMsgrDispatchThrottlerCluster {
    val: u64,
    max: u64,
    get: u64,
    get_sum: u64,
    get_or_fail_fail: u64,
    get_or_fail_success: u64,
    take: u64,
    take_sum: u64,
    put: u64,
    put_sum: u64,
    wait: Gauge
}

pub struct ThrottleMsgrDispatchThrottlerHbBackServer {
    val: u64,
    max: u64,
    get: u64,
    get_sum: u64,
    get_or_fail_fail: u64,
    get_or_fail_success: u64,
    take: u64,
    take_sum: u64,
    put: u64,
    put_sum: u64,
    wait: Gauge
}

pub struct ThrottleMsgrDispatchThrottlerHbFrontServer {
    val: u64,
    max: u64,
    get: u64,
    get_sum: u64,
    get_or_fail_fail: u64,
    get_or_fail_success: u64,
    take: u64,
    take_sum: u64,
    put: u64,
    put_sum: u64,
    wait: Gauge
}

pub struct ThrottleMsgrDispatchThrottlerHbclient {
    val: u64,
    max: u64,
    get: u64,
    get_sum: u64,
    get_or_fail_fail: u64,
    get_or_fail_success: u64,
    take: u64,
    take_sum: u64,
    put: u64,
    put_sum: u64,
    wait: Gauge
}

pub struct  ThrottleMsgrDispatchThrottlerMsObjecter {
    val: u64,
    max: u64,
    get: u64,
    get_sum: u64,
    get_or_fail_fail: u64,
    get_or_fail_success: u64,
    take: u64,
    take_sum: u64,
    put: u64,
    put_sum: u64,
    wait: Gauge
}

pub struct ThrottleObjecterBytes {
    val: u64,
    max: u64,
    get: u64,
    get_sum: u64,
    get_or_fail_fail: u64,
    get_or_fail_success: u64,
    take: u64,
    take_sum: u64,
    put: u64,
    put_sum: u64,
    wait: avgcount : f64,
    sum: f64
}

pub struct throttleObjecterOps {
    val: u64,
    max: u64,
    get: u64,
    get_sum: u64,
    get_or_fail_fail: u64,
    get_or_fail_success: u64,
    take: u64,
    take_sum: u64,
    put: u64,
    put_sum: u64,
    wait: Gauge
}

pub struct ThrottleOsdClientBytes {
    val: u64,
    max: u64,
    get: u64,
    get_sum: u64,
    get_or_fail_fail: u64,
    get_or_fail_success: u64,
    take: u64,
    take_sum: u64,
    put: u64,
    put_sum: u64,
    wait: Gauge
}

pub struct  ThrottleOsdClientMessages {
    val: u64,
    max: u64,
    get: u64,
    get_sum: u64,
    get_or_fail_fail: u64,
    get_or_fail_success: u64,
    take: u64,
    take_sum: u64,
    put: u64,
    put_sum: u64,
    wait: Gauge
}
*/
