extern crate time;
// extern crate output_args;
// extern crate ceph;

pub mod osd_perf{
    use communication;
    use influent::measurement::{Measurement, Value};
    use output_args::*;

    pub fn log(perf_dump: ::ceph::osd::perf_dump::PerfDump, args: &Args, osd_num: u32) {
        log_to_stdout(&perf_dump, args, osd_num);
        log_to_influx(&perf_dump, args, osd_num);
        log_to_carbon(&perf_dump, args, osd_num);
    }

    fn log_to_stdout(perf_dump: &::ceph::osd::perf_dump::PerfDump, args: &Args, osd_num: u32) {
        if args.outputs.contains(&"stdout".to_string()) {
            println!("osd.{}: {:?}", osd_num, perf_dump);
        }
    }

    fn log_to_influx(perf_dump: &::ceph::osd::perf_dump::PerfDump, args: &Args, osd_num: u32) {
        if args.outputs.contains(&"influx".to_string()) && args.influx.is_some() {
            let osd = format!("{}", osd_num.clone());
            let mut measurement = Measurement::new("osd");

            measurement.add_tag("osd", osd.as_ref());
            measurement.add_field("load_avg",
                                  Value::Integer(perf_dump.osd.loadavg as i64));
            measurement.add_field("op_latency",
                                  Value::Integer(perf_dump.osd.op_latency.sum as i64));
            measurement.add_field("op_r_latency",
                                  Value::Integer(perf_dump.osd.op_r_latency.sum as i64));
            measurement.add_field("op_w_latency",
                                  Value::Integer(perf_dump.osd.op_w_latency.sum as i64));
            measurement.add_field("subop_latency",
                                  Value::Integer(perf_dump.osd.subop_latency.sum as i64));
            measurement.add_field("subop_w_latency",
                                  Value::Integer(perf_dump.osd.subop_w_latency.sum as i64));
            measurement.add_field("journal_latency",
                                  Value::Integer(perf_dump.filestore.journal_latency.sum as i64));
            measurement.add_field("apply_latency",
                                  Value::Integer(perf_dump.filestore.apply_latency.sum as i64));
            measurement.add_field("queue_transaction_latency_avg",
                                  Value::Integer(perf_dump.filestore.queue_transaction_latency_avg.sum as i64));

            communication::send_to_influx(args, measurement);
        }
    }


    fn log_to_carbon(perf_dump: &::ceph::osd::perf_dump::PerfDump, args: &Args, osd_num: u32) {
        if args.outputs.contains(&"carbon".to_string()) && args.carbon.is_some() {
            let carbon = &args.carbon.clone().unwrap();
            let carbon_data = to_carbon_string(perf_dump, &carbon.root_key, osd_num);
            let _ = communication::send_to_carbon(args, carbon_data);
        }
    }

    fn to_carbon_string(perf_dump: &::ceph::osd::perf_dump::PerfDump, root_key: &String, osd_num: u32) -> String {
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
                timestamp = get_time() / 1000.0)
    }

    fn get_time() -> f64 {
        let now = super::time::now();
        let milliseconds_since_epoch = now.to_timespec().sec * 1000;
        return milliseconds_since_epoch as f64;
    }
}
