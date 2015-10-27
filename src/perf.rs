extern crate rustc_serialize;
extern crate time;
use influent::create_client;
use influent::client::Client;
use influent::client::Credentials;
use influent::measurement::{Measurement, Value};
use output_args::*;
use rustc_serialize::{Decoder, json};
use std::io::prelude::*;
use std::net::TcpStream;

fn get_time() -> f64 {
    let now = time::now();
    let milliseconds_since_epoch = now.to_timespec().sec * 1000;
    return milliseconds_since_epoch as f64;
}

#[derive(Debug, RustcDecodable)]
struct Wbthrottle {
    bytes_dirtied: i64,
    bytes_wb: i64,
    ios_dirtied: i64,
    ios_wb: i64,
    inodes_dirtied: i64,
    inodes_wb: i64,
}

#[derive(Debug, RustcDecodable)]
struct FilestoreJournalLatency {
    avgcount: i64,
    sum: f64,
}

#[derive(Debug, RustcDecodable)]
struct FilestoreJournalWrByte {
    avgcount: i64,
    sum: i64,
}

#[derive(Debug, RustcDecodable)]
struct Filestore {
    journal_queue_max_ops: i64,
    journal_queue_ops: i64,
    journal_ops: i64,
    journal_queue_max_bytes: i64,
    journal_queue_bytes: i64,
    journal_bytes: i64,
    journal_latency: FilestoreJournalLatency,
    journal_wr: i64,
    journal_wr_bytes: FilestoreJournalWrByte,
    journal_full: i64,
    committing: i64,
    commitcycle: i64,
    commitcycle_interval: FilestoreJournalLatency,
    commitcycle_latency: FilestoreJournalLatency,
    op_queue_max_ops: i64,
    op_queue_ops: i64,
    ops: i64,
    op_queue_max_bytes: i64,
    op_queue_bytes: i64,
    bytes: i64,
    apply_latency: FilestoreJournalLatency,
    queue_transaction_latency_avg: FilestoreJournalLatency,
}

#[derive(Debug, RustcDecodable)]
struct Leveldb {
    leveldb_get: i64,
    leveldb_transaction: i64,
    leveldb_compact: i64,
    leveldb_compact_range: i64,
    leveldb_compact_queue_merge: i64,
    leveldb_compact_queue_len: i64,
}

#[derive(Debug, RustcDecodable)]
struct MutexFilejournalCompletionsLock {
    wait: FilestoreJournalLatency,
}

#[derive(Debug, RustcDecodable)]
struct Objecter {
    op_active: i64,
    op_laggy: i64,
    op_send: i64,
    op_send_bytes: i64,
    op_resend: i64,
    op_ack: i64,
    op_commit: i64,
    op: i64,
    op_r: i64,
    op_w: i64,
    op_rmw: i64,
    op_pg: i64,
    osdop_stat: i64,
    osdop_create: i64,
    osdop_read: i64,
    osdop_write: i64,
    osdop_writefull: i64,
    osdop_append: i64,
    osdop_zero: i64,
    osdop_truncate: i64,
    osdop_delete: i64,
    osdop_mapext: i64,
    osdop_sparse_read: i64,
    osdop_clonerange: i64,
    osdop_getxattr: i64,
    osdop_setxattr: i64,
    osdop_cmpxattr: i64,
    osdop_rmxattr: i64,
    osdop_resetxattrs: i64,
    osdop_tmap_up: i64,
    osdop_tmap_put: i64,
    osdop_tmap_get: i64,
    osdop_call: i64,
    osdop_watch: i64,
    osdop_notify: i64,
    osdop_src_cmpxattr: i64,
    osdop_pgls: i64,
    osdop_pgls_filter: i64,
    osdop_other: i64,
    linger_active: i64,
    linger_send: i64,
    linger_resend: i64,
    poolop_active: i64,
    poolop_send: i64,
    poolop_resend: i64,
    poolstat_active: i64,
    poolstat_send: i64,
    poolstat_resend: i64,
    statfs_active: i64,
    statfs_send: i64,
    statfs_resend: i64,
    command_active: i64,
    command_send: i64,
    command_resend: i64,
    map_epoch: i64,
    map_full: i64,
    map_inc: i64,
    osd_sessions: i64,
    osd_session_open: i64,
    osd_session_close: i64,
    osd_laggy: i64,
}

#[derive(Debug, RustcDecodable)]
struct Osd {
    opq: i64,
    op_wip: i64,
    op: i64,
    op_in_bytes: i64,
    op_out_bytes: i64,
    op_latency: FilestoreJournalLatency,
    op_process_latency: FilestoreJournalLatency,
    op_r: i64,
    op_r_out_bytes: i64,
    op_r_latency: FilestoreJournalLatency,
    op_r_process_latency: FilestoreJournalLatency,
    op_w: i64,
    op_w_in_bytes: i64,
    op_w_rlat: FilestoreJournalLatency,
    op_w_latency: FilestoreJournalLatency,
    op_w_process_latency: FilestoreJournalLatency,
    op_rw: i64,
    op_rw_in_bytes: i64,
    op_rw_out_bytes: i64,
    op_rw_rlat: FilestoreJournalLatency,
    op_rw_latency: FilestoreJournalLatency,
    op_rw_process_latency: FilestoreJournalLatency,
    subop: i64,
    subop_in_bytes: i64,
    subop_latency: FilestoreJournalLatency,
    subop_w: i64,
    subop_w_in_bytes: i64,
    subop_w_latency: FilestoreJournalLatency,
    subop_pull: i64,
    subop_pull_latency: FilestoreJournalLatency,
    subop_push: i64,
    subop_push_in_bytes: i64,
    subop_push_latency: FilestoreJournalLatency,
    pull: i64,
    push: i64,
    push_out_bytes: i64,
    push_in: i64,
    push_in_bytes: i64,
    recovery_ops: i64,
    loadavg: i64,
    buffer_bytes: i64,
    numpg: i64,
    numpg_primary: i64,
    numpg_replica: i64,
    numpg_stray: i64,
    heartbeat_to_peers: i64,
    heartbeat_from_peers: i64,
    map_messages: i64,
    map_message_epochs: i64,
    map_message_epoch_dups: i64,
    messages_delayed_for_map: i64,
    stat_bytes: i64,
    stat_bytes_used: i64,
    stat_bytes_avail: i64,
    copyfrom: i64,
    tier_promote: i64,
    tier_flush: i64,
    tier_flush_fail: i64,
    tier_try_flush: i64,
    tier_try_flush_fail: i64,
    tier_evict: i64,
    tier_whiteout: i64,
    tier_dirty: i64,
    tier_clean: i64,
    tier_delay: i64,
    agent_wake: i64,
    agent_skip: i64,
    agent_flush: i64,
    agent_evict: i64,
}

#[allow(non_snake_case)]
#[derive(Debug, RustcDecodable)]
struct RecoverystatePerf {
    initial_latency: FilestoreJournalLatency,
    started_latency: FilestoreJournalLatency,
    reset_latency: FilestoreJournalLatency,
    start_latency: FilestoreJournalLatency,
    primary_latency: FilestoreJournalLatency,
    peering_latency: FilestoreJournalLatency,
    backfilling_latency: FilestoreJournalLatency,
    waitremotebackfillreserved_latency: FilestoreJournalLatency,
    waitlocalbackfillreserved_latency: FilestoreJournalLatency,
    notbackfilling_latency: FilestoreJournalLatency,
    repnotrecovering_latency: FilestoreJournalLatency,
    repwaitrecoveryreserved_latency: FilestoreJournalLatency,
    repwaitbackfillreserved_latency: FilestoreJournalLatency,
    RepRecovering_latency: FilestoreJournalLatency,
    activating_latency: FilestoreJournalLatency,
    waitlocalrecoveryreserved_latency: FilestoreJournalLatency,
    waitremoterecoveryreserved_latency: FilestoreJournalLatency,
    recovering_latency: FilestoreJournalLatency,
    recovered_latency: FilestoreJournalLatency,
    clean_latency: FilestoreJournalLatency,
    active_latency: FilestoreJournalLatency,
    replicaactive_latency: FilestoreJournalLatency,
    stray_latency: FilestoreJournalLatency,
    getinfo_latency: FilestoreJournalLatency,
    getlog_latency: FilestoreJournalLatency,
    waitactingchange_latency: FilestoreJournalLatency,
    incomplete_latency: FilestoreJournalLatency,
    getmissing_latency: FilestoreJournalLatency,
    waitupthru_latency: FilestoreJournalLatency,
}

#[derive(Debug, RustcDecodable)]
struct ThrottleFilestoreByte {
    val: i64,
    max: i64,
    get: i64,
    get_sum: i64,
    get_or_fail_fail: i64,
    get_or_fail_success: i64,
    take: i64,
    take_sum: i64,
    put: i64,
    put_sum: i64,
    wait: FilestoreJournalLatency,
}

#[allow(non_snake_case)]
#[derive(Debug, RustcDecodable)]
pub struct OsdPerf {
    WBThrottle: Wbthrottle,
    filestore: Filestore,
    leveldb: Leveldb,
    mutex_FileJournal_completions_lock: MutexFilejournalCompletionsLock,
    mutex_FileJournal_finisher_lock: MutexFilejournalCompletionsLock,
    mutex_FileJournal_write_lock: MutexFilejournalCompletionsLock,
    mutex_FileJournal_writeq_lock: MutexFilejournalCompletionsLock,
    mutex_JOS_ApplyManager_apply_lock: MutexFilejournalCompletionsLock,
    mutex_JOS_ApplyManager_com_lock: MutexFilejournalCompletionsLock,
    mutex_JOS_SubmitManager_lock: MutexFilejournalCompletionsLock,
    mutex_WBThrottle_lock: MutexFilejournalCompletionsLock,
    objecter: Objecter,
    osd: Osd,
    recoverystate_perf: RecoverystatePerf,
    throttle_filestore_bytes: ThrottleFilestoreByte,
    throttle_filestore_ops: ThrottleFilestoreByte,
    throttle_msgr_dispatch_throttler_client: ThrottleFilestoreByte,
    throttle_msgr_dispatch_throttler_cluster: ThrottleFilestoreByte,
    throttle_msgr_dispatch_throttler_hb_back_server: ThrottleFilestoreByte,
    throttle_msgr_dispatch_throttler_hb_front_server: ThrottleFilestoreByte,
    throttle_msgr_dispatch_throttler_hbclient: ThrottleFilestoreByte,
    throttle_msgr_dispatch_throttler_ms_objecter: ThrottleFilestoreByte,
    throttle_objecter_bytes: ThrottleFilestoreByte,
    throttle_objecter_ops: ThrottleFilestoreByte,
    throttle_osd_client_bytes: ThrottleFilestoreByte,
    throttle_osd_client_messages: ThrottleFilestoreByte,
}

#[allow(dead_code)]
impl OsdPerf{
    pub fn decode(json_data: &str) -> Result<Self, json::DecoderError> {
        let mut json = str::replace(json_data, "-", "_");
        json = str::replace(json.as_ref(), "::", "_");
        let decode: OsdPerf = try!(json::decode(json.as_ref()));
        return Ok(decode);
    }

    pub fn log(&self, args: &Args, osd_num: u32) {
        self.log_to_stdout(args, osd_num);
        self.log_to_influx(args, osd_num);
        self.log_to_carbon(args, osd_num);
    }

    fn log_to_stdout(&self, args: &Args, osd_num: u32) {
        if args.outputs.contains(&"stdout".to_string()) {
            println!("osd.{}: {:?}", osd_num, self);
        }
    }

    fn log_to_influx(&self, args: &Args, osd_num: u32) {
        if args.outputs.contains(&"influx".to_string()) && args.influx.is_some() {
            let influx = &args.influx.clone().unwrap();
            let credentials = Credentials {
                username: influx.user.as_ref(),
                password: influx.password.as_ref(),
                database: "ceph",
            };
            let host = format!("http://{}:{}", influx.host, influx.port);
            let hosts = vec![host.as_ref()];
            let client = create_client(credentials, hosts);
            let osd = format!("{}", osd_num.clone());
            let mut measurement = Measurement::new("osd");

            measurement.add_tag("osd", osd.as_ref());
            measurement.add_field("load_avg",
                                  Value::Integer(self.osd.loadavg));
            measurement.add_field("op_latency",
                                  Value::Integer(self.osd.op_latency.sum as i64));
            measurement.add_field("op_r_latency",
                                  Value::Integer(self.osd.op_r_latency.sum as i64));
            measurement.add_field("op_w_latency",
                                  Value::Integer(self.osd.op_w_latency.sum as i64));
            measurement.add_field("subop_latency",
                                  Value::Integer(self.osd.subop_latency.sum as i64));
            measurement.add_field("subop_w_latency",
                                  Value::Integer(self.osd.subop_w_latency.sum as i64));
            measurement.add_field("journal_latency",
                                  Value::Integer(self.filestore.journal_latency.sum as i64));
            measurement.add_field("apply_latency",
                                  Value::Integer(self.filestore.apply_latency.sum as i64));
            measurement.add_field("queue_transaction_latency_avg",
                                  Value::Integer(self.filestore.queue_transaction_latency_avg.sum as i64));

            let res = client.write_one(measurement, None);

            debug!("{:?}", res);
        }
    }

    fn log_packet_to_carbon(carbon_url: &str, carbon_data: String) -> Result<(), String> {
        let mut stream = try!(TcpStream::connect(carbon_url).map_err(|e| e.to_string()));
        let bytes_written = try!(stream.write(&carbon_data.into_bytes()[..])
                                       .map_err(|e| e.to_string()));
        debug!("Wrote: {} bytes to graphite", &bytes_written);
        Ok(())
    }

    fn log_to_carbon(&self, args: &Args, osd_num: u32) {
        if args.outputs.contains(&"carbon".to_string()) && args.carbon.is_some() {
            let carbon = &args.carbon.clone().unwrap();
            let carbon_data = self.to_carbon_string(&carbon.root_key, osd_num);

            let carbon_host = &carbon.host;
            let carbon_port = &carbon.port;
            let carbon_url = format!("{}:{}", carbon_host, carbon_port);
            let _ = OsdPerf::log_packet_to_carbon(carbon_url.as_ref(), carbon_data);
        }
    }

    fn to_carbon_string(&self, root_key: &String, osd_num: u32) -> String {
        format!(r#"{root_key}.{} {} {timestamp}
{root_key}.{} {} {timestamp}
{root_key}.{} {} {timestamp}
{root_key}.{} {} {timestamp}
{root_key}.{} {} {timestamp}
{root_key}.{} {} {timestamp}
{root_key}.{} {} {timestamp}
{root_key}.{} {} {timestamp}
{root_key}.{} {} {timestamp}
"#,
                "load_avg",
                self.osd.loadavg,
                "op_latency",
                self.osd.op_latency.sum,
                "op_r_latency",
                self.osd.op_r_latency.sum,
                "op_w_latency",
                self.osd.op_w_latency.sum,
                "subop_latency",
                self.osd.subop_latency.sum,
                "subop_w_latency",
                self.osd.subop_w_latency.sum,
                "journal_latency",
                self.filestore.journal_latency.sum,
                "apply_latency",
                self.filestore.apply_latency.sum,
                "queue_transaction_latency_avg",
                self.filestore.queue_transaction_latency_avg.sum,
                root_key = format!("{}-osd.{}",root_key.clone(), osd_num),
                timestamp = get_time() / 1000.0)
    }
}
