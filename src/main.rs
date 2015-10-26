extern crate ease;
#[macro_use]
extern crate log;
extern crate influent;
extern crate rustc_serialize;
extern crate simple_logger;
extern crate time;
extern crate yaml_rust;

use yaml_rust::YamlLoader;
use std::fs::{self, File};

// std
use std::io::prelude::*;
use std::path::Path;
use std::process::Command;
use std::sync::mpsc::Receiver;
use std::net::TcpStream;

// modules
mod perf;
mod health;

// crates
use influent::create_client;
use influent::client::Client;
use influent::client::Credentials;
use influent::measurement::{Measurement, Value};
use log::LogLevel;

macro_rules! parse_opt (
    ($name:ident, $doc:expr) => (
    let $name: Option<String> = match $doc.as_str() {
        Some(o) => Some(o.to_string()),
        None => None
    }
    );
);

#[derive(Clone,Debug)]
struct Args {
    carbon: Option<Carbon>,
    elasticsearch: Option<String>,
    stdout: Option<String>,
    influx: Option<Influx>,
    outputs: Vec<String>,
}

#[derive(Clone,Debug)]
struct Influx {
    user: String,
    password: String,
    host: String,
    port: String,
}

#[derive(Clone,Debug)]
struct Carbon {
    host: String,
    port: String,
    root_key: String,
}

impl Args {
    fn clean() -> Args {
        Args {
            carbon: None,
            elasticsearch: None,
            stdout: None,
            influx: None,
            outputs: Vec::new(),
        }
    }
}

fn get_config() -> Result<Args, String> {
    let mut f = try!(File::open("/etc/default/decode_ceph.yaml").map_err(|e| e.to_string()));

    let mut s = String::new();
    try!(f.read_to_string(&mut s).map_err(|e| e.to_string()));

    // Remove this hack when the new version of yaml_rust releases to get the real
    // error msg
    let docs = match YamlLoader::load_from_str(&s) {
        Ok(data) => data,
        Err(_) => {
            error!("Unable to load yaml data from config file");
            return Err("cannot load data from yaml".to_string());
        }
    };

    let doc = &docs[0];

    let elasticsearch = match doc["elasticsearch"].as_str() {
        Some(o) => Some(format!("http://{}/ceph/operations", o)),
        None => None,
    };
    parse_opt!(stdout, doc["stdout"]);
    let influx_doc = doc["influx"].clone();
    let influx_host = influx_doc["host"].as_str().unwrap_or("127.0.0.1");
    let influx_port = influx_doc["port"].as_str().unwrap_or("8086");
    let influx_password = influx_doc["password"].as_str().unwrap_or("root");
    let influx_user = influx_doc["user"].as_str().unwrap_or("root");
    let influx = Influx {
        host: influx_host.to_string(),
        port: influx_port.to_string(),
        password: influx_password.to_string(),
        user: influx_user.to_string(),
    };

    let carbon_doc = doc["carbon"].clone();

    let carbon_host = match carbon_doc["host"].as_str() {
        Some(h) => Some(h),
        None => None,
    };

    let carbon_port = carbon_doc["port"].as_str().unwrap_or("2003");
    let root_key = carbon_doc["root_key"].as_str().unwrap_or("ceph");

    let carbon = match carbon_host {
        Some(h) => Some(Carbon {
            host: h.to_string(),
            port: carbon_port.to_string(),
            root_key: root_key.to_string(),
        }),
        None => None,
    };

    let outputs: Vec<String> = match doc["outputs"].as_vec() {
        Some(o) => {
            o.iter()
             .map(|x| {
                 match x.as_str() {
                     Some(o) => o.to_string(),
                     None => "".to_string(),
                 }
             })
             .collect()
        }
        None => Vec::new(),
    };

    Ok(Args {
        carbon: carbon,
        elasticsearch: elasticsearch,
        stdout: stdout,
        influx: Some(influx),
        outputs: outputs,
    })
}

fn get_ceph_stats() -> Result<String, String> {
    //return Ok("{\"health\":{\"health\":{\"health_services\":[{\"mons\":[{\"name\":\"chris-local-machine-1\",\"kb_total\":232205304,\"kb_used\":81823684,\"kb_avail\":138563228,\"avail_percent\":59,\"last_updated\":\"2015-10-07 12:19:51.281273\",\"store_stats\":{\"bytes_total\":5408347,\"bytes_sst\":0,\"bytes_log\":4166001,\"bytes_misc\":1242346,\"last_updated\":\"0.000000\"},\"health\":\"HEALTH_OK\"},{\"name\":\"chris-local-machine-2\",\"kb_total\":232205304,\"kb_used\":79803236,\"kb_avail\":140583676,\"avail_percent\":60,\"last_updated\":\"2015-10-07 12:19:23.247120\",\"store_stats\":{\"bytes_total\":6844874,\"bytes_sst\":0,\"bytes_log\":5602535,\"bytes_misc\":1242339,\"last_updated\":\"0.000000\"},\"health\":\"HEALTH_OK\"},{\"name\":\"chris-local-machine-3\",\"kb_total\":232205304,\"kb_used\":78650196,\"kb_avail\":141736716,\"avail_percent\":61,\"last_updated\":\"2015-10-07 12:19:07.182466\",\"store_stats\":{\"bytes_total\":6531182,\"bytes_sst\":0,\"bytes_log\":5288894,\"bytes_misc\":1242288,\"last_updated\":\"0.000000\"},\"health\":\"HEALTH_OK\"}]}]},\"summary\":[],\"timechecks\":{\"epoch\":6,\"round\":38,\"round_status\":\"finished\",\"mons\":[{\"name\":\"chris-local-machine-1\",\"skew\":\"0.000000\",\"latency\":\"0.000000\",\"health\":\"HEALTH_OK\"},{\"name\":\"chris-local-machine-2\",\"skew\":\"0.000000\",\"latency\":\"0.000977\",\"health\":\"HEALTH_OK\"},{\"name\":\"chris-local-machine-3\",\"skew\":\"0.000000\",\"latency\":\"0.000818\",\"health\":\"HEALTH_OK\"}]},\"overall_status\":\"HEALTH_OK\",\"detail\":[]},\"fsid\":\"1bb15abc-4158-11e5-b499-00151737cf98\",\"election_epoch\":6,\"quorum\":[0,1,2],\"quorum_names\":[\"chris-local-machine-1\",\"chris-local-machine-2\",\"chris-local-machine-3\"],\"monmap\":{\"epoch\":2,\"fsid\":\"1bb15abc-4158-11e5-b499-00151737cf98\",\"modified\":\"2015-10-07 10:45:23.255204\",\"created\":\"0.000000\",\"mons\":[{\"rank\":0,\"name\":\"chris-local-machine-1\",\"addr\":\"10.0.2.22:6789/0\"},{\"rank\":1,\"name\":\"chris-local-machine-2\",\"addr\":\"10.0.2.78:6789/0\"},{\"rank\":2,\"name\":\"chris-local-machine-3\",\"addr\":\"10.0.2.141:6789/0\"}]},\"osdmap\":{\"osdmap\":{\"epoch\":9,\"num_osds\":3,\"num_up_osds\":3,\"num_in_osds\":3,\"full\":false,\"nearfull\":false}},\"pgmap\":{\"pgs_by_state\":[{\"state_name\":\"active+clean\",\"count\":192}],\"version\":487,\"num_pgs\":192,\"data_bytes\":4970896648,\"bytes_used\":252251439104,\"bytes_avail\":424777154560,\"bytes_total\":713334693888,\"write_bytes_sec\":26793300,\"op_per_sec\":8},\"mdsmap\":{\"epoch\":1,\"up\":0,\"in\":0,\"max\":1,\"by_rank\":[]}}".to_string());
    //return Ok("{\"health\":{\"health\":{\"health_services\":[{\"mons\":[{\"name\":\"ip-172-31-3-4\",\"kb_total\":257899908,\"kb_used\":2646276,\"kb_avail\":244667856,\"avail_percent\":94,\"last_updated\":\"2015-10-21 17:29:49.157456\",\"store_stats\":{\"bytes_total\":4211748,\"bytes_sst\":0,\"bytes_log\":2328812,\"bytes_misc\":1882936,\"last_updated\":\"0.000000\"},\"health\":\"HEALTH_OK\"},{\"name\":\"ip-172-31-18-59\",\"kb_total\":257899908,\"kb_used\":2626376,\"kb_avail\":244687756,\"avail_percent\":94,\"last_updated\":\"2015-10-21 17:29:55.825254\",\"store_stats\":{\"bytes_total\":5364733,\"bytes_sst\":0,\"bytes_log\":3481648,\"bytes_misc\":1883085,\"last_updated\":\"0.000000\"},\"health\":\"HEALTH_OK\"},{\"name\":\"ip-172-31-55-150\",\"kb_total\":257899908,\"kb_used\":2732484,\"kb_avail\":244581648,\"avail_percent\":94,\"last_updated\":\"2015-10-21 17:30:22.606563\",\"store_stats\":{\"bytes_total\":5470059,\"bytes_sst\":0,\"bytes_log\":3586875,\"bytes_misc\":1883184,\"last_updated\":\"0.000000\"},\"health\":\"HEALTH_OK\"}]}]},\"summary\":[],\"timechecks\":{\"epoch\":6,\"round\":64,\"round_status\":\"finished\",\"mons\":[{\"name\":\"ip-172-31-3-4\",\"skew\":\"0.000000\",\"latency\":\"0.000000\",\"health\":\"HEALTH_OK\"},{\"name\":\"ip-172-31-18-59\",\"skew\":\"-0.001446\",\"latency\":\"0.119155\",\"health\":\"HEALTH_OK\"},{\"name\":\"ip-172-31-55-150\",\"skew\":\"-0.005493\",\"latency\":\"0.003979\",\"health\":\"HEALTH_OK\"}]},\"overall_status\":\"HEALTH_OK\",\"detail\":[]},\"fsid\":\"1bb15abc-4158-11e5-b499-00151737cf98\",\"election_epoch\":6,\"quorum\":[0,1,2],\"quorum_names\":[\"ip-172-31-3-4\",\"ip-172-31-18-59\",\"ip-172-31-55-150\"],\"monmap\":{\"epoch\":2,\"fsid\":\"1bb15abc-4158-11e5-b499-00151737cf98\",\"modified\":\"2015-10-21 14:51:21.352722\",\"created\":\"0.000000\",\"mons\":[{\"rank\":0,\"name\":\"ip-172-31-3-4\",\"addr\":\"172.31.3.4:6789/0\"},{\"rank\":1,\"name\":\"ip-172-31-18-59\",\"addr\":\"172.31.18.59:6789/0\"},{\"rank\":2,\"name\":\"ip-172-31-55-150\",\"addr\":\"172.31.55.150:6789/0\"}]},\"osdmap\":{\"osdmap\":{\"epoch\":22,\"num_osds\":8,\"num_up_osds\":8,\"num_in_osds\":8,\"full\":false,\"nearfull\":false}},\"pgmap\":{\"pgs_by_state\":[{\"state_name\":\"active+clean\",\"count\":192}],\"version\":5467,\"num_pgs\":192,\"data_bytes\":1072168960,\"bytes_used\":22150647808,\"bytes_avail\":2003846721536,\"bytes_total\":2112716046336},\"mdsmap\":{\"epoch\":1,\"up\":0,\"in\":0,\"max\":1,\"by_rank\":[]}}".to_string());
    let output = Command::new("/usr/bin/ceph")
                     .arg("-s")
                     .arg("-f")
                     .arg("json")
                     .output()
                     .unwrap_or_else(|e| panic!("failed to execute ceph process: {}", e));
    let output_string = match String::from_utf8(output.stdout) {
        Ok(v) => v,
        Err(_) => "{}".to_string(),
    };
    Ok(output_string)
}

fn log_to_es(args: &Args, ceph_event: &health::CephHealth) {
    if args.outputs.contains(&"elasticsearch".to_string()) && args.elasticsearch.is_some() {
        let url = args.elasticsearch.clone().unwrap();
        let url = url.as_ref();
        debug!("Logging to {}", url);
        let parsed_url = match ease::Url::parse(url).map_err(|e| e.to_string()) {
            Ok(u) => u,
            Err(e) => {
                error!("{}", e);
                return;
            }
        };
        let mut req = ease::Request::new(parsed_url);
        req.body(ceph_event.to_json().clone());
        match req.post() {
            Ok(_) => {
                info!("Logged to ES");
                // return Ok(());},
            }
            Err(_) => {
                error!("ES POST FAILED");
                // return Err("Post operation failed".to_string());
            }
        }
    }
}

fn log_to_stdout(args: &Args, ceph_event: &health::CephHealth) {
    if args.outputs.contains(&"stdout".to_string()) {
        println!("{:?}", ceph_event);
    }
}

fn log_to_influx(args: &Args, ceph_event: &health::CephHealth) {
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

        let mut measurement = Measurement::new("monitor");
        measurement.add_field("ops",
                              Value::Integer(ceph_event.pgmap.op_per_sec.unwrap_or(0) as i64));
        measurement.add_field("writes",
                              Value::Integer(ceph_event.pgmap.write_bytes_sec.unwrap_or(0) as i64));
        measurement.add_field("reads",
                              Value::Integer(ceph_event.pgmap.read_bytes_sec.unwrap_or(0) as i64));
        measurement.add_field("data", Value::Integer(ceph_event.pgmap.data_bytes as i64));
        measurement.add_field("used", Value::Integer(ceph_event.pgmap.bytes_used as i64));
        measurement.add_field("avail", Value::Integer(ceph_event.pgmap.bytes_avail as i64));
        measurement.add_field("total", Value::Integer(ceph_event.pgmap.bytes_total as i64));
        measurement.add_field("osds",
                              Value::Integer(ceph_event.osdmap.osdmap.num_osds as i64));
        let res = client.write_one(measurement, None);

        debug!("{:?}", res);
    }
}

fn log_packet_to_carbon(carbon_url: &str, carbon_data: String) -> Result<(), String> {
    let mut stream = try!(TcpStream::connect(carbon_url).map_err(|e| e.to_string()));
    let bytes_written = try!(stream.write(&carbon_data.into_bytes()[..])
                                   .map_err(|e| e.to_string()));
    info!("Wrote: {} bytes to graphite", &bytes_written);
    Ok(())
}

fn log_to_carbon(args: &Args, ceph_event: &health::CephHealth) {
    if args.outputs.contains(&"carbon".to_string()) && args.carbon.is_some() {
        let carbon = &args.carbon.clone().unwrap();
        let carbon_data = ceph_event.to_carbon_string(&carbon.root_key);

        let carbon_host = &carbon.host;
        let carbon_port = &carbon.port;
        let carbon_url = format!("{}:{}", carbon_host, carbon_port);
        // println!("{}", carbon_data)
        let _ = log_packet_to_carbon(carbon_url.as_ref(), carbon_data);
    }
}

fn has_child_directory(dir: &Path) -> Result<bool, std::io::Error> {
    if try!(fs::metadata(dir)).is_dir() {
        for entry in try!(fs::read_dir(dir)) {
            let entry = try!(entry);
            if try!(fs::metadata(entry.path())).is_dir() {
                return Ok(true);
            }
        }
    }
    return Ok(false);
}

// Look for /var/lib/ceph/mon/ceph-ip-172-31-24-128
fn is_monitor() -> Result<bool, std::io::Error> {
    // does it have a mon directory entry?
    return has_child_directory(Path::new("/var/lib/ceph/mon"));
}

// Look for: /var/lib/ceph/osd/ceph-3/active
fn is_osd() -> Result<bool, std::io::Error> {
    // Lets check if the OSD is active
    for entry in try!(fs::read_dir(Path::new("/var/lib/ceph/osd"))) {
        // Usually only one entry in here
        let entry = try!(entry);
        if try!(fs::metadata(entry.path())).is_dir() {
            // descend and try to open the active file to check the OSD status
            let mut file_path = entry.path();
            file_path.push("active");

            let mut f = try!(File::open(file_path));
            let mut contents: String = String::new();
            try!(f.read_to_string(&mut contents));

            // Check if the OSD is active
            if contents.trim() == "ok" {
                return Ok(true);
            }
        }
    }
    return Ok(false);
}

fn main() {
    // TODO make configurable via cli or config arg
    simple_logger::init_with_level(LogLevel::Info).unwrap();

    let periodic = timer_periodic(1000);

    let is_monitor = match is_monitor() {
        Ok(result) => result,
        Err(_) => false,
    };

    let args = match get_config() {
        Ok(a) => a,
        Err(_) => Args::clean(),
    };
    debug!("{:?}", args);
    loop {
        let _ = periodic.recv();

        // Only grab stats from the ceph monitor
        if is_monitor {
            let _ = periodic.recv();
            let json = match get_ceph_stats() {
                Ok(json) => json,
                Err(_) => "{}".to_string(),
            };

            let ceph_event = match health::CephHealth::decode(&json) {
                Ok(json) => json,
                Err(error) => {
                    warn!("There was an error: {:?}", error);
                    continue;
                }
            };

            log_to_es(&args, &ceph_event);
            log_to_stdout(&args, &ceph_event);
            log_to_influx(&args, &ceph_event);
            log_to_carbon(&args, &ceph_event);
        }
    }
}

fn timer_periodic(ms: u32) -> Receiver<()> {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        loop {
            std::thread::sleep_ms(ms);
            if tx.send(()).is_err() {
                break;
            }
        }
    });
    rx
}
