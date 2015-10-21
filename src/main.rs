extern crate rustc_serialize;
extern crate time;
extern crate ease;
extern crate uuid;
extern crate yaml_rust;

use uuid::Uuid;
#[macro_use] extern crate log;
extern crate simple_logger;
extern crate influent;

use rustc_serialize::json;
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
    fsid: Uuid,
    ops: i64,
    write_bytes_sec: f64,
    read_bytes_sec: f64,
    data: f64,
    bytes_used: f64,
    bytes_avail: f64,
    bytes_total: f64
}

#[test]
fn test_autodecode(){
    let test_str = "{\"health\":{\"health\":{\"health_services\":[{\"mons\":[{\"name\":\"chris-local-machine-1\",\"kb_total\":232205304,\"kb_used\":81823684,\"kb_avail\":138563228,\"avail_percent\":59,\"last_updated\":\"2015-10-07 12:19:51.281273\",\"store_stats\":{\"bytes_total\":5408347,\"bytes_sst\":0,\"bytes_log\":4166001,\"bytes_misc\":1242346,\"last_updated\":\"0.000000\"},\"health\":\"HEALTH_OK\"},{\"name\":\"chris-local-machine-2\",\"kb_total\":232205304,\"kb_used\":79803236,\"kb_avail\":140583676,\"avail_percent\":60,\"last_updated\":\"2015-10-07 12:19:23.247120\",\"store_stats\":{\"bytes_total\":6844874,\"bytes_sst\":0,\"bytes_log\":5602535,\"bytes_misc\":1242339,\"last_updated\":\"0.000000\"},\"health\":\"HEALTH_OK\"},{\"name\":\"chris-local-machine-3\",\"kb_total\":232205304,\"kb_used\":78650196,\"kb_avail\":141736716,\"avail_percent\":61,\"last_updated\":\"2015-10-07 12:19:07.182466\",\"store_stats\":{\"bytes_total\":6531182,\"bytes_sst\":0,\"bytes_log\":5288894,\"bytes_misc\":1242288,\"last_updated\":\"0.000000\"},\"health\":\"HEALTH_OK\"}]}]},\"summary\":[],\"timechecks\":{\"epoch\":6,\"round\":38,\"round_status\":\"finished\",\"mons\":[{\"name\":\"chris-local-machine-1\",\"skew\":\"0.000000\",\"latency\":\"0.000000\",\"health\":\"HEALTH_OK\"},{\"name\":\"chris-local-machine-2\",\"skew\":\"0.000000\",\"latency\":\"0.000977\",\"health\":\"HEALTH_OK\"},{\"name\":\"chris-local-machine-3\",\"skew\":\"0.000000\",\"latency\":\"0.000818\",\"health\":\"HEALTH_OK\"}]},\"overall_status\":\"HEALTH_OK\",\"detail\":[]},\"fsid\":\"1bb15abc-4158-11e5-b499-00151737cf98\",\"election_epoch\":6,\"quorum\":[0,1,2],\"quorum_names\":[\"chris-local-machine-1\",\"chris-local-machine-2\",\"chris-local-machine-3\"],\"monmap\":{\"epoch\":2,\"fsid\":\"1bb15abc-4158-11e5-b499-00151737cf98\",\"modified\":\"2015-10-07 10:45:23.255204\",\"created\":\"0.000000\",\"mons\":[{\"rank\":0,\"name\":\"chris-local-machine-1\",\"addr\":\"10.0.2.22:6789/0\"},{\"rank\":1,\"name\":\"chris-local-machine-2\",\"addr\":\"10.0.2.78:6789/0\"},{\"rank\":2,\"name\":\"chris-local-machine-3\",\"addr\":\"10.0.2.141:6789/0\"}]},\"osdmap\":{\"osdmap\":{\"epoch\":9,\"num_osds\":3,\"num_up_osds\":3,\"num_in_osds\":3,\"full\":false,\"nearfull\":false}},\"pgmap\":{\"pgs_by_state\":[{\"state_name\":\"active+clean\",\"count\":192}],\"version\":487,\"num_pgs\":192,\"data_bytes\":4970896648,\"bytes_used\":252251439104,\"bytes_avail\":424777154560,\"bytes_total\":713334693888,\"write_bytes_sec\":26793300,\"op_per_sec\":8},\"mdsmap\":{\"epoch\":1,\"up\":0,\"in\":0,\"max\":1,\"by_rank\":[]}}";
    let decoded: TestCephHealth = json::decode(test_str).unwrap();
}

#[derive(RustcDecodable, RustcEncodable)]
struct TestCephHealth  {
    election_epoch: u64,
    fsid: String,
    health: Vec<Health>,
    quorum: Vec<u64>,
    quorum_names: Vec<String>,
}

#[derive(RustcDecodable, RustcEncodable)]
struct Health{
    detail: Vec<String>,
    health: SubHealth,
    overall_status: String,
    summary: Vec<String>,
    timechecks: Vec<TimeCheck>
}

#[derive(RustcDecodable, RustcEncodable)]
struct SubHealth{
    health_servies: HealthService,
}

#[derive(RustcDecodable, RustcEncodable)]
struct HealthService{
    mons: Vec<MonHealthDetails>
}

#[derive(RustcDecodable, RustcEncodable)]
struct TimeCheck{
    epoch: u64,
    round: u64,
    round_status: String,
    mon_health: Vec<MonHealth>
}

#[derive(RustcDecodable, RustcEncodable)]
struct MonHealthDetails{
    avail_percent: u64,
    health: String,
    kb_avail: u64,
    kb_total: u64,
    kb_used: u64,
    last_updated: String,
    name: String,
    store_stats: MonStoreStat,
    bytes_misc: u64,
    bytes_sst: u64,
    bytes_total: u64,
}

#[derive(RustcDecodable, RustcEncodable)]
struct MonStoreStat{
    bytes_long: u64,
}

#[derive(RustcDecodable, RustcEncodable)]
struct MonHealth{
    health: String,
    latency: String,
    name: String,
    skew: String,
}

#[derive(RustcDecodable, RustcEncodable)]
struct MdsMap{
    by_rank: Vec<String>,
    epoch: u64,
    in_map: u64,
    max: u64,
    up: u64,
}

#[derive(RustcDecodable, RustcEncodable)]
struct MonMap{
    created: String,
    epoch: u64,
    fsid: String,
    modified: String,
    mons: Vec<Mon>
}
#[derive(RustcDecodable, RustcEncodable)]
struct PgMap{
    bytes_avail: u64,
    bytes_total: u64,
    bytes_used: u64,
    data_bytes: u64,
    num_pgs: u64,
    pgs_by_state: Vec<PgState>,
    version: u64,
}

#[derive(RustcDecodable, RustcEncodable)]
struct OsdMap{
    epoch: u64,
    full: bool,
    nearfull: bool,
    num_in_osds: u64,
    num_osds: u64,
    num_up_osds: u64
}

#[derive(RustcDecodable, RustcEncodable)]
struct PgState{
    count: u64,
    state_name: String,
}

#[derive(RustcDecodable, RustcEncodable)]
struct Mon{
    addr: String,
    name: String,
    rank: u64,
}

/*
Object(
{
"election_epoch": U64(6),
"fsid": String("ecbb8960-0e21-11e2-b495-83a88f44db01"),
"health":
    Object(
        {
        "detail": Array([]),
        "health": Object({
            "health_services":
                Array([Object({
                "mons": Array([Object({
                    "avail_percent": U64(87),
                    "health": String("HEALTH_OK"),
                    "kb_avail": U64(829003484),
                    "kb_total": U64(952772524),
                    "kb_used": U64(75347972),
                    "last_updated": String("2015-10-20 17:08:12.836783"),
                    "name": String("chris-local-machine-3"),
                    "store_stats": Object({"bytes_log": U64(257690),
                    "bytes_misc": U64(936989),
                    "bytes_sst": U64(0),
                    "bytes_total": U64(1194679),
                    "last_updated": String("0.000000")})}),
                Object({"avail_percent": U64(87),
                 "health": String("HEALTH_OK"), "kb_avail": U64(828993252), "kb_total": U64(952772524),
                 "kb_used": U64(75358204), "last_updated": String("2015-10-20 17:08:06.998879"),
                 "name": String("chris-local-machine-2"), "store_stats": Object({"bytes_log": U64(385439),
                 "bytes_misc": U64(937120), "bytes_sst": U64(0), "bytes_total": U64(1322559),
                "last_updated": String("0.000000")})}),

            Object({"avail_percent": U64(87),
                "health": String("HEALTH_OK"), "kb_avail": U64(828993252), "kb_total": U64(952772524),
                "kb_used": U64(75358204), "last_updated": String("2015-10-20 17:08:06.998800"),
                "name": String("chris-local-machine-4"), "store_stats": Object({"bytes_log": U64(385439),
                "bytes_misc": U64(937120), "bytes_sst": U64(0), "bytes_total": U64(1322559),
                "last_updated": String("0.000000")})})])}
            )])}),

        "overall_status": String("HEALTH_OK"),
        "summary": Array([]),
        "timechecks": Object({"epoch": U64(6),
            "mons": Array(
                [Object({"health": String("HEALTH_OK"),
                    "latency": String("0.000000"),
                    "name": String("chris-local-machine-3"),
                    "skew": String("0.000000")}),
                Object({"health": String("HEALTH_OK"), "latency": String("0.000762"),
                     "name": String("chris-local-machine-2"), "skew": String("0.000000")}),
                Object({"health": String("HEALTH_OK"),
                      "latency": String("0.000714"), "name": String("chris-local-machine-4"), "skew": String("0.000000")})]),
            "round": U64(220),
            "round_status": String("finished")})
        }),
    "mdsmap": Object({"by_rank": Array([]), "epoch": U64(1), "in": U64(0), "max": U64(1),
               "up": U64(0)}),
    "monmap": Object({"created": String("0.000000"), "epoch": U64(1), "fsid": String("ecbb8960-0e21-11e2-b495-83a88f44db01"),
                "modified": String("0.000000"),
            "mons": Array([Object({"addr": String("10.0.3.26:6789/0"), "name": String("chris-local-machine-3"),
                 "rank": U64(0)}), Object({"addr": String("10.0.3.144:6789/0"), "name": String("chris-local-machine-2"), "rank": U64(1)}),
                 Object({"addr": String("10.0.3.243:6789/0"), "name": String("chris-local-machine-4"), "rank": U64(2)})])}),
    "osdmap": Object({"osdmap": Object({"epoch": U64(15), "full": Boolean(false), "nearfull": Boolean(false),
                  "num_in_osds": U64(3), "num_osds": U64(3), "num_up_osds": U64(3)})}),
    "pgmap": Object({"bytes_avail": U64(2546671624192),
                  "bytes_total": U64(2926917193728), "bytes_used": U64(231496048640), "data_bytes": U64(0),
                  "num_pgs": U64(192),
                  "pgs_by_state": Array([Object({"count": U64(192), "state_name": String("active+clean")})]),
                  "version": U64(618)}),
    "quorum": Array([U64(0), U64(1), U64(2)]),
    "quorum_names": Array([String("chris-local-machine-3"), String("chris-local-machine-2"),
                  String("chris-local-machine-4")])})
 */


fn get_time()->f64{
    let now = time::now();
    let milliseconds_since_epoch = now.to_timespec().sec * 1000;
    return milliseconds_since_epoch as f64;
}

// JSON value representation
impl CephHealth{
    fn to_json(&self)->String{

        format!("{{\"fsid\":\"{}\",\"ops_per_sec\": \"{}\",\"write_bytes_sec\": \"{}\", \"read_bytes_sec\": \"{}\", \"data\":\"{}\", \
            \"bytes_used\":{}, \"bytes_avail\":{}, \"bytes_total\":\"{}\", \"postDate\": {}}}",
            self.fsid.to_hyphenated_string(),
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
        println!("{:?}", obj);

        let fsid = match obj.find("fsid"){
            Some(fsid_json) => {
                match fsid_json.as_string(){
                    Some(fsid) => fsid,
                    None => "",
                }
            },
            None => "",
        };

        let ceph_event = CephHealth {
            fsid: Uuid::parse_str(fsid).unwrap(),
            ops: parse_i64(i_hate_unwraps(&obj["pgmap"], &"op_per_sec")),
            write_bytes_sec: to_mb(parse_f64(i_hate_unwraps(&obj["pgmap"], "write_bytes_sec"))),
            read_bytes_sec: to_mb(parse_f64(i_hate_unwraps(&obj["pgmap"], "read_bytes_sec"))),
            data: to_tb(parse_f64(i_hate_unwraps(&obj["pgmap"], "data_bytes"))),
            bytes_used: to_tb(parse_f64(i_hate_unwraps(&obj["pgmap"], "bytes_used"))),
            bytes_avail: to_tb(parse_f64(i_hate_unwraps(&obj["pgmap"], "bytes_avail"))),
            bytes_total: to_tb(parse_f64(i_hate_unwraps(&obj["pgmap"], "bytes_total"))),
        };
        println!("Ceph event: {:?}", &ceph_event);

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
