#[macro_use]
extern crate log;
extern crate influent;
extern crate output_args;
extern crate regex;
extern crate rustc_serialize;
extern crate simple_logger;
extern crate time;
extern crate ceph;

// std
use std::str::FromStr;
use std::path::Path;
use std::sync::mpsc::Receiver;
use std::fs;

// modules
mod logging;
mod communication;

// crates
use output_args::*;
use regex::Regex;
use ceph::*;

fn get_config() -> output_args::Args {
    output_args::get_args()
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
fn check_is_monitor() -> bool {
    // does it have a mon directory entry?
    match has_child_directory(Path::new("/var/lib/ceph/mon")){
        Ok(result) => result,
        Err(_) => {
            info!("No Monitor found");
            false
        }
    }
}

//NOTE: This skips a lot of failure cases
// Check for osd sockets and give back a vec of osd numbers that are active
fn get_osds_with_match() -> Result<Vec<u64>, std::io::Error> {
    let mut osds: Vec<u64> = Vec::new();

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
                    let num = u64::from_str(osd_number).unwrap();
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

fn get_osds() -> Vec<u64> {
    match get_osds_with_match() {
        Ok(list) => list,
        Err(_) => {
            info!("No OSDs found");
            vec![]
        }
    }
}

fn main() {
    let args = get_config();

    simple_logger::init_with_level(args.log_level.clone()).unwrap();

    let periodic = timer_periodic(5000);

    let mut is_monitor = check_is_monitor();

    let mut osd_list = get_osds();


    debug!("{:?}", args);
    let mut i = 0;
    loop {
        i = i + 1;
        trace!("Going around again!");
        // Grab stats from the ceph monitor
        if is_monitor {
            trace!("Getting MON info");
            let _ = match ceph::get_monitor_perf_dump() {
                Some(dump) => Some(logging::mon_perf::log(dump, &args)),
                None => {
                    is_monitor = check_is_monitor();
                    None
                }
            };
            
        }

        //Now the osds
        for osd_num in osd_list.clone().iter(){
            match ceph::get_osd_perf_dump(osd_num) {
                Some(osd) => {
                    let drive_name = ceph::osd_mount_point(osd_num).unwrap_or("".to_string());
                    logging::osd_perf::log(osd, &args, *osd_num, &drive_name);
                },
                None => continue,
            }
        }
        osd_list = match i % 10 {
            0 => get_osds(),
            _ => osd_list,
        };
        is_monitor = match i % 10 {
            0 => check_is_monitor(),
            _ => is_monitor,
        };
        let _ = periodic.recv();
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
