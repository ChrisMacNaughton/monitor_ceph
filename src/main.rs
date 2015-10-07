extern crate rustc_serialize;
use rustc_serialize::json::Json;
extern crate yaml_rust;
use yaml_rust::YamlLoader;
use std::fs::File;
use std::io::prelude::*;
use std::process::Command;
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
    outputs: Vec<String>,
}

fn get_config() -> Result<Args, String>{
    let mut f = try!(File::open("/etc/default/decode_ceph.yaml").map_err(|e| e.to_string()));

    let mut s = String::new();
    try!(f.read_to_string(&mut s).map_err(|e| e.to_string()));

    //Remove this hack when the new version of yaml_rust releases to get the real error msg
    let docs = match YamlLoader::load_from_str(&s){
        Ok(data) => data,
        Err(_) => {
            return Err("Unable to load yaml data from config file".to_string());
        }
    };

    let doc = &docs[0];
    parse_opt!(carbon, doc["carbon"]);
    parse_opt!(elasticsearch, doc["elasticsearch"]);
    parse_opt!(stdout, doc["stdout"]);

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

    return Ok(Args {
        carbon: carbon,
        elasticsearch: elasticsearch,
        stdout: stdout,
        outputs: outputs,
    })
}

#[derive(Debug)]
struct CephHealth {
    ops: u64,
    write_bytes_sec: u64,
    data: u64,
    bytes_used: u64,
    bytes_avail: u64,
    bytes_total: u64
}

fn parse_u64(num: Result<rustc_serialize::json::Json, u64>) -> u64 {
    match num {
        Ok(num) =>  match num.as_u64() {
            Some(o) => o,
            None => 0,
        },
        Err(e) => e
    }

}

fn get_ceph_stats() -> Result<String, String> {
    // Ok("{\"health\":{\"health\":{\"health_services\":[{\"mons\":[{\"name\":\"chris-local-machine-1\",\"kb_total\":232205304,\"kb_used\":81823684,\"kb_avail\":138563228,\"avail_percent\":59,\"last_updated\":\"2015-10-07 12:19:51.281273\",\"store_stats\":{\"bytes_total\":5408347,\"bytes_sst\":0,\"bytes_log\":4166001,\"bytes_misc\":1242346,\"last_updated\":\"0.000000\"},\"health\":\"HEALTH_OK\"},{\"name\":\"chris-local-machine-2\",\"kb_total\":232205304,\"kb_used\":79803236,\"kb_avail\":140583676,\"avail_percent\":60,\"last_updated\":\"2015-10-07 12:19:23.247120\",\"store_stats\":{\"bytes_total\":6844874,\"bytes_sst\":0,\"bytes_log\":5602535,\"bytes_misc\":1242339,\"last_updated\":\"0.000000\"},\"health\":\"HEALTH_OK\"},{\"name\":\"chris-local-machine-3\",\"kb_total\":232205304,\"kb_used\":78650196,\"kb_avail\":141736716,\"avail_percent\":61,\"last_updated\":\"2015-10-07 12:19:07.182466\",\"store_stats\":{\"bytes_total\":6531182,\"bytes_sst\":0,\"bytes_log\":5288894,\"bytes_misc\":1242288,\"last_updated\":\"0.000000\"},\"health\":\"HEALTH_OK\"}]}]},\"summary\":[],\"timechecks\":{\"epoch\":6,\"round\":38,\"round_status\":\"finished\",\"mons\":[{\"name\":\"chris-local-machine-1\",\"skew\":\"0.000000\",\"latency\":\"0.000000\",\"health\":\"HEALTH_OK\"},{\"name\":\"chris-local-machine-2\",\"skew\":\"0.000000\",\"latency\":\"0.000977\",\"health\":\"HEALTH_OK\"},{\"name\":\"chris-local-machine-3\",\"skew\":\"0.000000\",\"latency\":\"0.000818\",\"health\":\"HEALTH_OK\"}]},\"overall_status\":\"HEALTH_OK\",\"detail\":[]},\"fsid\":\"1bb15abc-4158-11e5-b499-00151737cf98\",\"election_epoch\":6,\"quorum\":[0,1,2],\"quorum_names\":[\"chris-local-machine-1\",\"chris-local-machine-2\",\"chris-local-machine-3\"],\"monmap\":{\"epoch\":2,\"fsid\":\"1bb15abc-4158-11e5-b499-00151737cf98\",\"modified\":\"2015-10-07 10:45:23.255204\",\"created\":\"0.000000\",\"mons\":[{\"rank\":0,\"name\":\"chris-local-machine-1\",\"addr\":\"10.0.2.22:6789/0\"},{\"rank\":1,\"name\":\"chris-local-machine-2\",\"addr\":\"10.0.2.78:6789/0\"},{\"rank\":2,\"name\":\"chris-local-machine-3\",\"addr\":\"10.0.2.141:6789/0\"}]},\"osdmap\":{\"osdmap\":{\"epoch\":9,\"num_osds\":3,\"num_up_osds\":3,\"num_in_osds\":3,\"full\":false,\"nearfull\":false}},\"pgmap\":{\"pgs_by_state\":[{\"state_name\":\"active+clean\",\"count\":192}],\"version\":487,\"num_pgs\":192,\"data_bytes\":4970896648,\"bytes_used\":252251439104,\"bytes_avail\":424777154560,\"bytes_total\":713334693888,\"write_bytes_sec\":26793300,\"op_per_sec\":8},\"mdsmap\":{\"epoch\":1,\"up\":0,\"in\":0,\"max\":1,\"by_rank\":[]}}".to_string())
    // Ok("{\"health\":{\"health\":{\"health_services\":[{\"mons\":[{\"name\":\"chris-local-machine-1\",\"kb_total\":232205304,\"kb_used\":81823684,\"kb_avail\":138563228,\"avail_percent\":59,\"last_updated\":\"2015-10-07 12:19:51.281273\",\"store_stats\":{\"bytes_total\":5408347,\"bytes_sst\":0,\"bytes_log\":4166001,\"bytes_misc\":1242346,\"last_updated\":\"0.000000\"},\"health\":\"HEALTH_OK\"},{\"name\":\"chris-local-machine-2\",\"kb_total\":232205304,\"kb_used\":79803236,\"kb_avail\":140583676,\"avail_percent\":60,\"last_updated\":\"2015-10-07 12:19:23.247120\",\"store_stats\":{\"bytes_total\":6844874,\"bytes_sst\":0,\"bytes_log\":5602535,\"bytes_misc\":1242339,\"last_updated\":\"0.000000\"},\"health\":\"HEALTH_OK\"},{\"name\":\"chris-local-machine-3\",\"kb_total\":232205304,\"kb_used\":78650196,\"kb_avail\":141736716,\"avail_percent\":61,\"last_updated\":\"2015-10-07 12:19:07.182466\",\"store_stats\":{\"bytes_total\":6531182,\"bytes_sst\":0,\"bytes_log\":5288894,\"bytes_misc\":1242288,\"last_updated\":\"0.000000\"},\"health\":\"HEALTH_OK\"}]}]},\"summary\":[],\"timechecks\":{\"epoch\":6,\"round\":38,\"round_status\":\"finished\",\"mons\":[{\"name\":\"chris-local-machine-1\",\"skew\":\"0.000000\",\"latency\":\"0.000000\",\"health\":\"HEALTH_OK\"},{\"name\":\"chris-local-machine-2\",\"skew\":\"0.000000\",\"latency\":\"0.000977\",\"health\":\"HEALTH_OK\"},{\"name\":\"chris-local-machine-3\",\"skew\":\"0.000000\",\"latency\":\"0.000818\",\"health\":\"HEALTH_OK\"}]},\"overall_status\":\"HEALTH_OK\",\"detail\":[]},\"fsid\":\"1bb15abc-4158-11e5-b499-00151737cf98\",\"election_epoch\":6,\"quorum\":[0,1,2],\"quorum_names\":[\"chris-local-machine-1\",\"chris-local-machine-2\",\"chris-local-machine-3\"],\"monmap\":{\"epoch\":2,\"fsid\":\"1bb15abc-4158-11e5-b499-00151737cf98\",\"modified\":\"2015-10-07 10:45:23.255204\",\"created\":\"0.000000\",\"mons\":[{\"rank\":0,\"name\":\"chris-local-machine-1\",\"addr\":\"10.0.2.22:6789/0\"},{\"rank\":1,\"name\":\"chris-local-machine-2\",\"addr\":\"10.0.2.78:6789/0\"},{\"rank\":2,\"name\":\"chris-local-machine-3\",\"addr\":\"10.0.2.141:6789/0\"}]},\"osdmap\":{\"osdmap\":{\"epoch\":9,\"num_osds\":3,\"num_up_osds\":3,\"num_in_osds\":3,\"full\":false,\"nearfull\":false}},\"pgmap\":{\"pgs_by_state\":[{\"state_name\":\"active+clean\",\"count\":192}],\"version\":487,\"num_pgs\":192,\"data_bytes\":4970896648,\"bytes_used\":252251439104,\"bytes_avail\":424777154560,\"bytes_total\":713334693888,\"write_bytes_sec\":26793300},\"mdsmap\":{\"epoch\":1,\"up\":0,\"in\":0,\"max\":1,\"by_rank\":[]}}".to_string())
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

fn i_hate_unwraps(json: &rustc_serialize::json::Json, key: &str) -> Result<rustc_serialize::json::Json, u64> {

    match json.find(key) {
        Some(v) => {
            Ok(v.clone())
        },
        None => Err(0),
    }

}

fn main() {
    let json = match get_ceph_stats(){
        Ok(json) => json,
        Err(_) => "{}".to_string(),
    };

    let obj = Json::from_str(json.as_ref()).unwrap();
    // println!("{:?}", obj);

    let ceph_event = CephHealth {
        ops: parse_u64(i_hate_unwraps(&obj["pgmap"], &"op_per_sec")),
        write_bytes_sec: parse_u64(i_hate_unwraps(&obj["pgmap"], "write_bytes_sec")),
        data: parse_u64(i_hate_unwraps(&obj["pgmap"], "data_bytes")),
        bytes_used: parse_u64(i_hate_unwraps(&obj["pgmap"], "bytes_used")),
        bytes_avail: parse_u64(i_hate_unwraps(&obj["pgmap"], "bytes_avail")),
        bytes_total: parse_u64(i_hate_unwraps(&obj["pgmap"], "bytes_total")),
    };

    let args = get_config();
    println!("{:?}", args);
    println!("{:?}", ceph_event);
}
