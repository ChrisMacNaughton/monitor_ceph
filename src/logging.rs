extern crate time;
extern crate uuid;
extern crate hyper;
// extern crate output_args;
// extern crate ceph;
use std::process::Command;

fn hostname() -> String{
    let output = Command::new("hostname")
                         .output()
                         .unwrap_or_else(|e| panic!("failed to execute hostname: {}", e));
    let host = match String::from_utf8(output.stdout) {
        Ok(v) => v.replace("\n", ""),
        Err(_) => "{}".to_string(),
   };
   trace!("Got hostname: '{}'", host);

   host
}

pub mod json {
    use hyper::header::ContentType;
    use output_args::*;
    use hyper::*;
    pub fn log(json_str: String, args: &Args) {
        if args.influx.is_none()
        {
            return;
        }
        let influx = &args.influx.clone().unwrap();
        
        let host_string = format!("http://{}:{}/record_ceph?measurement=monitor&hostname={}", influx.host, influx.port, super::hostname());
        let host: &str = host_string.as_ref();
        let body: &str = json_str.as_ref();
        
        send(host, body);
    }

    pub fn log_osd(json_str: String, args: &Args, osd_num: u64, drive_name: &String) {
        if args.influx.is_none()
        {
            return;
        }
        let influx = &args.influx.clone().unwrap();
        let host_string = format!("http://{}:{}/record_ceph?measurement=osd&osd_num={}&drive_name={}&hostname={}", influx.host, influx.port, osd_num, drive_name, super::hostname());
        let host: &str = host_string.as_ref();
        let body: &str = json_str.as_ref();
        
        send(host, body);
    }

    fn send(url: &str, body: &str) {
        let client = Client::new();
        client.post(url)
            .body(body)
            .header(ContentType::json())
            .send();
    }
}

pub mod mon_perf {
    use influent::measurement::{Measurement, Value};
    use output_args::*;

    use communication;

    pub fn log(perf_dump: ::ceph::mon::perf_dump::PerfDump, args: &Args) {
        log_to_stdout(&perf_dump, args);
        log_to_influx(&perf_dump, args);
        log_to_carbon(&perf_dump, args);
    }

    fn log_to_stdout(perf_dump: &::ceph::mon::perf_dump::PerfDump, args: &Args) {
        if args.outputs.contains(&"stdout".to_string()) {
            let hostname = super::hostname();
            println!("[{}] {:?}", hostname, perf_dump);
        }
    }

    fn log_to_influx(perf_dump: &::ceph::mon::perf_dump::PerfDump, args: &Args) {
        if args.outputs.contains(&"influx".to_string()) && args.influx.is_some() {
            let hostname = super::hostname();
            let mut measurement = Measurement::new("monitor");
            measurement.add_tag("hostname", hostname.as_ref());
            // Cluster data
            measurement.add_field("used", Value::Integer(perf_dump.cluster.osd_kb_used as i64));
            measurement.add_field("avail", Value::Integer(perf_dump.cluster.osd_kb_avail as i64));
            measurement.add_field("total", Value::Integer(perf_dump.cluster.osd_kb as i64));

            measurement.add_field("osds",
                                  Value::Integer(perf_dump.cluster.num_osd as i64));
            measurement.add_field("osds_up",
                                  Value::Integer(perf_dump.cluster.num_osd_up as i64));
            measurement.add_field("osds_in",
                                  Value::Integer(perf_dump.cluster.num_osd_in as i64));
            measurement.add_field("osd_epoch",
                                  Value::Integer(perf_dump.cluster.osd_epoch as i64));

            // PlacementGroup data

            measurement.add_field("pgs",
                                  Value::Integer(perf_dump.cluster.num_pg as i64));
            measurement.add_field("pgs_active_clean",
                                  Value::Integer(perf_dump.cluster.num_pg_active_clean as i64));
            measurement.add_field("pgs_active",
                                  Value::Integer(perf_dump.cluster.num_pg_active as i64));
            measurement.add_field("pgs_peering",
                                  Value::Integer(perf_dump.cluster.num_pg_peering as i64));

            // Object Data
            measurement.add_field("objects",
                                  Value::Integer(perf_dump.cluster.num_object as i64));
            measurement.add_field("objects_degraded",
                                  Value::Integer(perf_dump.cluster.num_object_degraded as i64));
            measurement.add_field("objects_unfound",
                                  Value::Integer(perf_dump.cluster.num_object_unfound as i64));

            // Monitor data
            measurement.add_field("monitors",
                                  Value::Integer(perf_dump.cluster.num_mon as i64));
            measurement.add_field("monitors_quorum",
                                  Value::Integer(perf_dump.cluster.num_mon_quorum as i64));

            communication::send_to_influx(args, measurement);
        }
    }


    fn log_to_carbon(perf_dump: &::ceph::mon::perf_dump::PerfDump, args: &Args) {
        if args.outputs.contains(&"carbon".to_string()) && args.carbon.is_some() {
            let carbon = &args.carbon.clone().unwrap();
            let carbon_data = to_carbon_string(perf_dump, &carbon.root_key);
            let _ = communication::send_to_carbon(args, carbon_data);
        }
    }

    fn to_carbon_string(perf_dump: &::ceph::mon::perf_dump::PerfDump, root_key: &String) -> String {
        format!(r#"{root_key}.{} {} {timestamp}
{root_key}.{} {} {timestamp}
{root_key}.{} {} {timestamp}
{root_key}.{} {} {timestamp}
{root_key}.{} {} {timestamp}
{root_key}.{} {} {timestamp}
{root_key}.{} {} {timestamp}
{root_key}.{} {} {timestamp}
{root_key}.{} {} {timestamp}
{root_key}.{} {} {timestamp}
{root_key}.{} {} {timestamp}
{root_key}.{} {} {timestamp}
{root_key}.{} {} {timestamp}
{root_key}.{} {} {timestamp}
"#,
                "used",
                perf_dump.cluster.osd_kb_used,
                "avail",
                perf_dump.cluster.osd_kb_avail,
                "total",
                perf_dump.cluster.osd_kb,
                "osds",
                perf_dump.cluster.num_osd,
                "osds_up",
                perf_dump.cluster.num_osd_up,
                "osds_in",
                perf_dump.cluster.num_osd_in,
                "osd_epoch",
                perf_dump.cluster.osd_epoch,
                "pgs",
                perf_dump.cluster.num_pg,
                "pgs_active",
                perf_dump.cluster.num_pg_active,
                "pgs_active_clean",
                perf_dump.cluster.num_pg_active_clean,
                "pgs_peering",
                perf_dump.cluster.num_pg_peering,
                "objects",
                perf_dump.cluster.num_object,
                "objects_unfound",
                perf_dump.cluster.num_object_unfound,
                "objects_degraded",
                perf_dump.cluster.num_object_degraded,
                root_key = root_key.clone(),
                timestamp = super::get_time() / 1000.0)
    }
}
pub mod osd_perf {
    use communication;
    use influent::measurement::{Measurement, Value};
    use output_args::*;

    pub fn log(perf_dump: ::ceph::osd::perf_dump::PerfDump, args: &Args, osd_num: u64, drive_name: &String) {
        log_to_stdout(&perf_dump, args, osd_num, drive_name);
        log_to_influx(&perf_dump, args, osd_num, drive_name);
        log_to_carbon(&perf_dump, args, osd_num, drive_name);
    }

    fn log_to_stdout(perf_dump: &::ceph::osd::perf_dump::PerfDump, args: &Args, osd_num: u64, drive_name: &String) {
        if args.outputs.contains(&"stdout".to_string()) {
            let hostname = super::hostname();
            println!("[{}] osd.{}({}): {:?}", hostname, osd_num, drive_name, perf_dump);
        }
    }

    fn log_to_influx(perf_dump: &::ceph::osd::perf_dump::PerfDump, args: &Args, osd_num: u64, drive_name: &String) {
        if args.outputs.contains(&"influx".to_string()) && args.influx.is_some() {
            let osd = format!("{}", osd_num.clone());
            let hostname = super::hostname();
            let mut measurement = Measurement::new("osd");
            measurement.add_tag("hostname", hostname.as_ref());
            measurement.add_tag("osd", osd.as_ref());
            measurement.add_tag("drive", drive_name.as_ref());
            measurement.add_field("load_avg",
                                  Value::Integer(perf_dump.osd.loadavg as i64));
            measurement.add_field("op_queue_ops",
                                  Value::Integer(perf_dump.filestore.op_queue_ops as i64));
            measurement.add_field("ops",
                                  Value::Integer(perf_dump.filestore.ops as i64));

            measurement.add_field("op_latency",
                                  Value::Float(perf_dump.osd.op_latency.sum));
            measurement.add_field("op_r_latency",
                                  Value::Float(perf_dump.osd.op_r_latency.sum));
            measurement.add_field("op_w_latency",
                                  Value::Float(perf_dump.osd.op_w_latency.sum));
            measurement.add_field("subop_latency",
                                  Value::Float(perf_dump.osd.subop_latency.sum));
            measurement.add_field("subop_w_latency",
                                  Value::Float(perf_dump.osd.subop_w_latency.sum));
            measurement.add_field("journal_latency",
                                  Value::Float(perf_dump.filestore.journal_latency.sum));
            measurement.add_field("apply_latency",
                                  Value::Float(perf_dump.filestore.apply_latency.sum));
            measurement.add_field("commit_latency",
                                  Value::Float(perf_dump.filestore.commitcycle_latency.sum));
            measurement.add_field("queue_transaction_latency_avg",
                                  Value::Float(perf_dump.filestore.queue_transaction_latency_avg.sum));

            // USD Space data
            measurement.add_field("stat_bytes",
                                  Value::Integer(perf_dump.osd.stat_bytes as i64));
            measurement.add_field("stat_bytes_used",
                                  Value::Integer(perf_dump.osd.stat_bytes_used as i64));
            measurement.add_field("stat_bytes_avail",
                                  Value::Integer(perf_dump.osd.stat_bytes_avail as i64));

            communication::send_to_influx(args, measurement);
        }
    }


    fn log_to_carbon(perf_dump: &::ceph::osd::perf_dump::PerfDump, args: &Args, osd_num: u64, drive_name: &String) {
        if args.outputs.contains(&"carbon".to_string()) && args.carbon.is_some() {
            let carbon = &args.carbon.clone().unwrap();
            let carbon_data = to_carbon_string(perf_dump, &carbon.root_key, osd_num);
            let _ = communication::send_to_carbon(args, carbon_data);
        }
    }

    fn to_carbon_string(perf_dump: &::ceph::osd::perf_dump::PerfDump, root_key: &String, osd_num: u64) -> String {
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
                perf_dump.osd.loadavg,
                "op_latency",
                perf_dump.osd.op_latency.sum,
                "op_r_latency",
                perf_dump.osd.op_r_latency.sum,
                "op_w_latency",
                perf_dump.osd.op_w_latency.sum,
                "subop_latency",
                perf_dump.osd.subop_latency.sum,
                "subop_w_latency",
                perf_dump.osd.subop_w_latency.sum,
                "journal_latency",
                perf_dump.filestore.journal_latency.sum,
                "apply_latency",
                perf_dump.filestore.apply_latency.sum,
                "queue_transaction_latency_avg",
                perf_dump.filestore.queue_transaction_latency_avg.sum,
                root_key = format!("{}-osd.{}",root_key.clone(), osd_num),
                timestamp = super::get_time() / 1000.0)
    }
}

fn get_time() -> f64 {
    let now = super::time::now();
    let milliseconds_since_epoch = now.to_timespec().sec * 1000;
    return milliseconds_since_epoch as f64;
}