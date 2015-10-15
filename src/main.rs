extern crate rustc_serialize;
extern crate time;
extern crate ease;
extern crate yaml_rust;
#[macro_use] extern crate log;
extern crate simple_logger;
extern crate influent;

use rustc_serialize::json::Json;
use yaml_rust::YamlLoader;
use std::fs::File;
use std::io::prelude::*;
use std::process::Command;
use std::sync::mpsc::Receiver;
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
    carbon: Option<String>,
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
    port: String
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

fn get_config() -> Result<Args, String>{
    let mut f = try!(File::open("/etc/default/decode_ceph.yaml").map_err(|e| e.to_string()));

    let mut s = String::new();
    try!(f.read_to_string(&mut s).map_err(|e| e.to_string()));

    //Remove this hack when the new version of yaml_rust releases to get the real error msg
    let docs = match YamlLoader::load_from_str(&s){
        Ok(data) => data,
        Err(_) => {
            error!("Unable to load yaml data from config file");
            return Err("cannot load data from yaml".to_string());
        }
    };

    let doc = &docs[0];
    parse_opt!(carbon, doc["carbon"]);
    // parse_opt!(elasticsearch, doc["elasticsearch"]);
    let elasticsearch = match doc["elasticsearch"].as_str() {
        Some(o) => Some(format!("http://{}/ceph/operations", o)),
        None => None
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

    let outputs: Vec<String> = match doc["outputs"].as_vec() {
        Some(o) => {
            o.iter().map( |x|
                match x.as_str() {
                    Some(o) => o.to_string(),
                    None => "".to_string(),
                }
            ).collect()
        },
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

#[derive(Debug)]
struct CephHealth {
    ops: i64,
    write_bytes_sec: f64,
    read_bytes_sec: f64,
    data: f64,
    bytes_used: f64,
    bytes_avail: f64,
    bytes_total: f64
}
fn get_time()->f64{
    let now = time::now();
    let milliseconds_since_epoch = now.to_timespec().sec * 1000;
    return milliseconds_since_epoch as f64;
}
// JSON value representation
impl CephHealth{
    fn to_json(&self)->String{

        format!("{{\"ops_per_sec\": \"{}\",\"write_bytes_sec\": \"{}\", \"read_bytes_sec\": \"{}\", \"data\":\"{}\", \
            \"bytes_used\":{}, \"bytes_avail\":{}, \"bytes_total\":\"{}\", \"postDate\": {}}}",
            self.ops,
            self.write_bytes_sec,
            self.read_bytes_sec,
            self.data,
            self.bytes_used,
            self.bytes_avail,
            self.bytes_total,
            get_time())
    }
}

fn to_kb(num: f64) -> f64 {
    num / 1024.0
}

fn to_mb(num: f64) -> f64 {
    to_kb(num) / 1024.0
}

fn to_gb(num: f64) -> f64 {
    to_mb(num) / 1024.0
}

fn to_tb(num: f64) -> f64 {
    to_gb(num) / 1024.0
}

fn parse_f64(num: Result<rustc_serialize::json::Json, f64>) -> f64 {
    match num {
        Ok(num) =>  match num.as_f64() {
            Some(o) => o,
            None => 0.0,
        },
        Err(e) => e
    }
}

fn parse_i64(num: Result<rustc_serialize::json::Json, f64>) -> i64 {
    match num {
        Ok(num) =>  match num.as_i64() {
            Some(o) => o,
            None => 0,
        },
        Err(_) => 0
    }
}

fn get_ceph_stats() -> Result<String, String> {
    // return Ok("{\"health\":{\"health\":{\"health_services\":[{\"mons\":[{\"name\":\"chris-local-machine-1\",\"kb_total\":232205304,\"kb_used\":81823684,\"kb_avail\":138563228,\"avail_percent\":59,\"last_updated\":\"2015-10-07 12:19:51.281273\",\"store_stats\":{\"bytes_total\":5408347,\"bytes_sst\":0,\"bytes_log\":4166001,\"bytes_misc\":1242346,\"last_updated\":\"0.000000\"},\"health\":\"HEALTH_OK\"},{\"name\":\"chris-local-machine-2\",\"kb_total\":232205304,\"kb_used\":79803236,\"kb_avail\":140583676,\"avail_percent\":60,\"last_updated\":\"2015-10-07 12:19:23.247120\",\"store_stats\":{\"bytes_total\":6844874,\"bytes_sst\":0,\"bytes_log\":5602535,\"bytes_misc\":1242339,\"last_updated\":\"0.000000\"},\"health\":\"HEALTH_OK\"},{\"name\":\"chris-local-machine-3\",\"kb_total\":232205304,\"kb_used\":78650196,\"kb_avail\":141736716,\"avail_percent\":61,\"last_updated\":\"2015-10-07 12:19:07.182466\",\"store_stats\":{\"bytes_total\":6531182,\"bytes_sst\":0,\"bytes_log\":5288894,\"bytes_misc\":1242288,\"last_updated\":\"0.000000\"},\"health\":\"HEALTH_OK\"}]}]},\"summary\":[],\"timechecks\":{\"epoch\":6,\"round\":38,\"round_status\":\"finished\",\"mons\":[{\"name\":\"chris-local-machine-1\",\"skew\":\"0.000000\",\"latency\":\"0.000000\",\"health\":\"HEALTH_OK\"},{\"name\":\"chris-local-machine-2\",\"skew\":\"0.000000\",\"latency\":\"0.000977\",\"health\":\"HEALTH_OK\"},{\"name\":\"chris-local-machine-3\",\"skew\":\"0.000000\",\"latency\":\"0.000818\",\"health\":\"HEALTH_OK\"}]},\"overall_status\":\"HEALTH_OK\",\"detail\":[]},\"fsid\":\"1bb15abc-4158-11e5-b499-00151737cf98\",\"election_epoch\":6,\"quorum\":[0,1,2],\"quorum_names\":[\"chris-local-machine-1\",\"chris-local-machine-2\",\"chris-local-machine-3\"],\"monmap\":{\"epoch\":2,\"fsid\":\"1bb15abc-4158-11e5-b499-00151737cf98\",\"modified\":\"2015-10-07 10:45:23.255204\",\"created\":\"0.000000\",\"mons\":[{\"rank\":0,\"name\":\"chris-local-machine-1\",\"addr\":\"10.0.2.22:6789/0\"},{\"rank\":1,\"name\":\"chris-local-machine-2\",\"addr\":\"10.0.2.78:6789/0\"},{\"rank\":2,\"name\":\"chris-local-machine-3\",\"addr\":\"10.0.2.141:6789/0\"}]},\"osdmap\":{\"osdmap\":{\"epoch\":9,\"num_osds\":3,\"num_up_osds\":3,\"num_in_osds\":3,\"full\":false,\"nearfull\":false}},\"pgmap\":{\"pgs_by_state\":[{\"state_name\":\"active+clean\",\"count\":192}],\"version\":487,\"num_pgs\":192,\"data_bytes\":4970896648,\"bytes_used\":252251439104,\"bytes_avail\":424777154560,\"bytes_total\":713334693888,\"write_bytes_sec\":26793300,\"op_per_sec\":8},\"mdsmap\":{\"epoch\":1,\"up\":0,\"in\":0,\"max\":1,\"by_rank\":[]}}".to_string());
    // return Ok("{\"health\":{\"health\":{\"health_services\":[{\"mons\":[{\"name\":\"chris-local-machine-1\",\"kb_total\":232205304,\"kb_used\":81823684,\"kb_avail\":138563228,\"avail_percent\":59,\"last_updated\":\"2015-10-07 12:19:51.281273\",\"store_stats\":{\"bytes_total\":5408347,\"bytes_sst\":0,\"bytes_log\":4166001,\"bytes_misc\":1242346,\"last_updated\":\"0.000000\"},\"health\":\"HEALTH_OK\"},{\"name\":\"chris-local-machine-2\",\"kb_total\":232205304,\"kb_used\":79803236,\"kb_avail\":140583676,\"avail_percent\":60,\"last_updated\":\"2015-10-07 12:19:23.247120\",\"store_stats\":{\"bytes_total\":6844874,\"bytes_sst\":0,\"bytes_log\":5602535,\"bytes_misc\":1242339,\"last_updated\":\"0.000000\"},\"health\":\"HEALTH_OK\"},{\"name\":\"chris-local-machine-3\",\"kb_total\":232205304,\"kb_used\":78650196,\"kb_avail\":141736716,\"avail_percent\":61,\"last_updated\":\"2015-10-07 12:19:07.182466\",\"store_stats\":{\"bytes_total\":6531182,\"bytes_sst\":0,\"bytes_log\":5288894,\"bytes_misc\":1242288,\"last_updated\":\"0.000000\"},\"health\":\"HEALTH_OK\"}]}]},\"summary\":[],\"timechecks\":{\"epoch\":6,\"round\":38,\"round_status\":\"finished\",\"mons\":[{\"name\":\"chris-local-machine-1\",\"skew\":\"0.000000\",\"latency\":\"0.000000\",\"health\":\"HEALTH_OK\"},{\"name\":\"chris-local-machine-2\",\"skew\":\"0.000000\",\"latency\":\"0.000977\",\"health\":\"HEALTH_OK\"},{\"name\":\"chris-local-machine-3\",\"skew\":\"0.000000\",\"latency\":\"0.000818\",\"health\":\"HEALTH_OK\"}]},\"overall_status\":\"HEALTH_OK\",\"detail\":[]},\"fsid\":\"1bb15abc-4158-11e5-b499-00151737cf98\",\"election_epoch\":6,\"quorum\":[0,1,2],\"quorum_names\":[\"chris-local-machine-1\",\"chris-local-machine-2\",\"chris-local-machine-3\"],\"monmap\":{\"epoch\":2,\"fsid\":\"1bb15abc-4158-11e5-b499-00151737cf98\",\"modified\":\"2015-10-07 10:45:23.255204\",\"created\":\"0.000000\",\"mons\":[{\"rank\":0,\"name\":\"chris-local-machine-1\",\"addr\":\"10.0.2.22:6789/0\"},{\"rank\":1,\"name\":\"chris-local-machine-2\",\"addr\":\"10.0.2.78:6789/0\"},{\"rank\":2,\"name\":\"chris-local-machine-3\",\"addr\":\"10.0.2.141:6789/0\"}]},\"osdmap\":{\"osdmap\":{\"epoch\":9,\"num_osds\":3,\"num_up_osds\":3,\"num_in_osds\":3,\"full\":false,\"nearfull\":false}},\"pgmap\":{\"pgs_by_state\":[{\"state_name\":\"active+clean\",\"count\":192}],\"version\":487,\"num_pgs\":192,\"data_bytes\":4970896648,\"bytes_used\":252251439104,\"bytes_avail\":424777154560,\"bytes_total\":713334693888,\"write_bytes_sec\":26793300},\"mdsmap\":{\"epoch\":1,\"up\":0,\"in\":0,\"max\":1,\"by_rank\":[]}}".to_string());
    let output = Command::new("/usr/bin/ceph")
                         .arg("-s")
                         .arg("-f")
                         .arg("json")
                         .output()
                         .unwrap_or_else(|e| { panic!("failed to execute ceph process: {}", e) });
    let output_string = match String::from_utf8(output.stdout) {
        Ok(v) => v,
        Err(_) => "{}".to_string(),
    };
    Ok(output_string)
}

fn i_hate_unwraps(json: &rustc_serialize::json::Json, key: &str) -> Result<rustc_serialize::json::Json, f64> {

    match json.find(key) {
        Some(v) => {
            Ok(v.clone())
        },
        None => Err(0.0),
    }

}

fn log_to_es(args: &Args, ceph_event: &CephHealth) {
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
        match req.post(){
            Ok(_) => {
                info!("Logged to ES");
                // return Ok(());},
            },
            Err(_) => {
                error!("ES POST FAILED");
                // return Err("Post operation failed".to_string());
            }
        };
    }


}

fn log_to_stdout(args: &Args, ceph_event: &CephHealth) {
    if args.outputs.contains(&"stdout".to_string()){
        println!("{:?}", ceph_event);
    }
}

// struct CephHealth {
//     ops: f64,
//     write_bytes_sec: f64,
//     read_bytes_sec: f64,
//     data: f64,
//     bytes_used: f64,
//     bytes_avail: f64,
//     bytes_total: f64
// }
fn log_to_influx(args: &Args, ceph_event: &CephHealth) {
    if args.outputs.contains(&"influx".to_string()) && args.influx.is_some() {
        let influx = &args.influx.clone().unwrap();
        let credentials = Credentials {
            username: influx.user.as_ref(),
            password: influx.password.as_ref(),
            database: "ceph"
        };
        let host = format!("http://{}:{}",influx.host, influx.port);
        let hosts = vec![host.as_ref()];
        let client = create_client(credentials, hosts);

        let mut measurement = Measurement::new("monitor");
        measurement.add_field("ops", Value::Integer(ceph_event.ops));
        measurement.add_field("writes", Value::Float(ceph_event.write_bytes_sec));
        measurement.add_field("reads", Value::Float(ceph_event.read_bytes_sec));
        measurement.add_field("data", Value::Float(ceph_event.data));
        measurement.add_field("used", Value::Float(ceph_event.bytes_used));
        measurement.add_field("avail", Value::Float(ceph_event.bytes_avail));
        measurement.add_field("total", Value::Float(ceph_event.bytes_total));

        let res = client.write_one(measurement, None);

        debug!("{:?}", res);
    }
}

fn main() {
    //TODO make configurable via cli or config arg
    simple_logger::init_with_level(LogLevel::Info).unwrap();

    let periodic = timer_periodic(1000);

    let args = match get_config() {
        Ok(a) => a,
        Err(_) => Args::clean(),
    };
    debug!("{:?}", args);
    loop{
        let _ = periodic.recv();
        let json = match get_ceph_stats(){
            Ok(json) => json,
            Err(_) => "{}".to_string(),
        };

        let obj = Json::from_str(json.as_ref()).unwrap();
        // println!("{:?}", obj);

        let ceph_event = CephHealth {
            ops: parse_i64(i_hate_unwraps(&obj["pgmap"], &"op_per_sec")),
            write_bytes_sec: to_mb(parse_f64(i_hate_unwraps(&obj["pgmap"], "write_bytes_sec"))),
            read_bytes_sec: to_mb(parse_f64(i_hate_unwraps(&obj["pgmap"], "read_bytes_sec"))),
            data: to_tb(parse_f64(i_hate_unwraps(&obj["pgmap"], "data_bytes"))),
            bytes_used: to_tb(parse_f64(i_hate_unwraps(&obj["pgmap"], "bytes_used"))),
            bytes_avail: to_tb(parse_f64(i_hate_unwraps(&obj["pgmap"], "bytes_avail"))),
            bytes_total: to_tb(parse_f64(i_hate_unwraps(&obj["pgmap"], "bytes_total"))),
        };

        log_to_es(&args, &ceph_event);
        log_to_stdout(&args, &ceph_event);
        log_to_influx(&args, &ceph_event);
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