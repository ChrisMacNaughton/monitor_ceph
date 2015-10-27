#[macro_use]
extern crate log;
extern crate influent;
extern crate output_args;
extern crate regex;
extern crate rustc_serialize;
extern crate simple_logger;
extern crate time;

// std
use std::str::FromStr;
use std::path::Path;
use std::process::Command;
use std::sync::mpsc::Receiver;
use std::fs::{self};
use std::io::prelude::*;

// modules
mod perf;
mod health;
mod communication;

// crates
use output_args::*;
use regex::Regex;

fn get_config() -> output_args::Args {
    output_args::get_args()
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

fn get_osd_perf(osd_num: u32) -> Result<String, String> {
    let output = Command::new("/usr/bin/ceph")
                         .arg("daemon")
                         .arg(format!("osd.{}", osd_num))
                         .arg("perf")
                         .arg("dump")
                         .output()
                         .unwrap_or_else(|e| panic!("failed to execute ceph process: {}", e));
    let output_string = match String::from_utf8(output.stdout) {
        Ok(v) => v,
        Err(_) => "{}".to_string(),
    };
    Ok(output_string)
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
fn is_monitor() -> bool {
    // does it have a mon directory entry?
    match has_child_directory(Path::new("/var/lib/ceph/mon")){
        Ok(result) => result,
        Err(_) => false,
    }
}

//NOTE: This skips a lot of failure cases
// Check for osd sockets and give back a vec of osd numbers that are active
fn get_osds() -> Result<Vec<u32>, std::io::Error> {
    let mut osds: Vec<u32> = Vec::new();

    let osd_regex = Regex::new(r"ceph-osd.(?P<number>\d+).asok").unwrap();

    for entry in try!(fs::read_dir(Path::new("/var/run/ceph"))){
        //parse the unix socket names such as:
        //ceph-mon.ip-172-31-22-89.asok
        //ceph-osd.1.asok

        let entry = try!(entry);
        let sock_addr_osstr = entry.file_name();
        let file_name = match sock_addr_osstr.to_str(){
            Some(name) => name,
            None => {
                //Skip files we can't turn into a string
                continue;
            }
        };

        //Ignore failures
        match osd_regex.captures(file_name){
            Some(osd) => {
                if let Some(osd_number) = osd.name("number"){
                    let num = u32::from_str(osd_number).unwrap();
                    osds.push(num);
                }
                //Ignore failures
            }
            //Ignore non matches, ie: ceph monitors
            None => {},
        }
    }
    return Ok(osds);
}

fn main() {
    let args = get_config();

    simple_logger::init_with_level(args.log_level.clone()).unwrap();

    let periodic = timer_periodic(1000);

    let is_monitor = is_monitor();

    let osd_list = match get_osds(){
        Ok(json) => json,
        Err(error) => {
            warn!("Error getting osd list {:?}", error);
            //TODO: What should we do here?
            return;
        }
    };


    debug!("{:?}", args);
    loop {
        let _ = periodic.recv();

        // Grab stats from the ceph monitor
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
            ceph_event.log(&args);
        }

        //Now the osds
        for osd_num in osd_list.iter(){
            let json = match get_osd_perf(*osd_num){
                Ok(json) => json,
                Err(_) => "{}".to_string(),
            };

            let ceph_event = match perf::OsdPerf::decode(&json) {
                Ok(json) => json,
                Err(error) => {
                    warn!("There was an error: {:?}", error);
                    continue;
                }
            };
            ceph_event.log(&args, *osd_num);
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
