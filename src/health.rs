extern crate rustc_serialize;

use rustc_serialize::{json, Decoder, Decodable};

#[test]
fn test_autodecode(){
    let test_str = "{\"health\":{\"health\":{\"health_services\":[{\"mons\":[{\"name\":\"chris-local-machine-1\",\"kb_total\":232205304,\"kb_used\":81823684,\"kb_avail\":138563228,\"avail_percent\":59,\"last_updated\":\"2015-10-07 12:19:51.281273\",\"store_stats\":{\"bytes_total\":5408347,\"bytes_sst\":0,\"bytes_log\":4166001,\"bytes_misc\":1242346,\"last_updated\":\"0.000000\"},\"health\":\"HEALTH_OK\"},{\"name\":\"chris-local-machine-2\",\"kb_total\":232205304,\"kb_used\":79803236,\"kb_avail\":140583676,\"avail_percent\":60,\"last_updated\":\"2015-10-07 12:19:23.247120\",\"store_stats\":{\"bytes_total\":6844874,\"bytes_sst\":0,\"bytes_log\":5602535,\"bytes_misc\":1242339,\"last_updated\":\"0.000000\"},\"health\":\"HEALTH_OK\"},{\"name\":\"chris-local-machine-3\",\"kb_total\":232205304,\"kb_used\":78650196,\"kb_avail\":141736716,\"avail_percent\":61,\"last_updated\":\"2015-10-07 12:19:07.182466\",\"store_stats\":{\"bytes_total\":6531182,\"bytes_sst\":0,\"bytes_log\":5288894,\"bytes_misc\":1242288,\"last_updated\":\"0.000000\"},\"health\":\"HEALTH_OK\"}]}]},\"summary\":[],\"timechecks\":{\"epoch\":6,\"round\":38,\"round_status\":\"finished\",\"mons\":[{\"name\":\"chris-local-machine-1\",\"skew\":\"0.000000\",\"latency\":\"0.000000\",\"health\":\"HEALTH_OK\"},{\"name\":\"chris-local-machine-2\",\"skew\":\"0.000000\",\"latency\":\"0.000977\",\"health\":\"HEALTH_OK\"},{\"name\":\"chris-local-machine-3\",\"skew\":\"0.000000\",\"latency\":\"0.000818\",\"health\":\"HEALTH_OK\"}]},\"overall_status\":\"HEALTH_OK\",\"detail\":[]},\"fsid\":\"1bb15abc-4158-11e5-b499-00151737cf98\",\"election_epoch\":6,\"quorum\":[0,1,2],\"quorum_names\":[\"chris-local-machine-1\",\"chris-local-machine-2\",\"chris-local-machine-3\"],\"monmap\":{\"epoch\":2,\"fsid\":\"1bb15abc-4158-11e5-b499-00151737cf98\",\"modified\":\"2015-10-07 10:45:23.255204\",\"created\":\"0.000000\",\"mons\":[{\"rank\":0,\"name\":\"chris-local-machine-1\",\"addr\":\"10.0.2.22:6789/0\"},{\"rank\":1,\"name\":\"chris-local-machine-2\",\"addr\":\"10.0.2.78:6789/0\"},{\"rank\":2,\"name\":\"chris-local-machine-3\",\"addr\":\"10.0.2.141:6789/0\"}]},\"osdmap\":{\"osdmap\":{\"epoch\":9,\"num_osds\":3,\"num_up_osds\":3,\"num_in_osds\":3,\"full\":false,\"nearfull\":false}},\"pgmap\":{\"pgs_by_state\":[{\"state_name\":\"active+clean\",\"count\":192}],\"version\":487,\"num_pgs\":192,\"data_bytes\":4970896648,\"bytes_used\":252251439104,\"bytes_avail\":424777154560,\"bytes_total\":713334693888,\"write_bytes_sec\":26793300,\"op_per_sec\":8},\"mdsmap\":{\"epoch\":1,\"up\":0,\"in\":0,\"max\":1,\"by_rank\":[]}}";
    let decoded: TestCephHealth = json::decode(test_str).unwrap();
    println!("Decoded: {:?}", decoded);
}

#[derive(Debug, RustcDecodable)]
pub struct TestCephHealth  {
    election_epoch: u64,
    fsid: String,
    health: Health,
    quorum: Vec<u64>,
    quorum_names: Vec<String>,
}

#[derive(Debug, RustcDecodable)]
pub struct Health{
    detail: Vec<String>,
    health: SubHealth,
    overall_status: String,
    summary: Vec<SummaryDetail>,
    timechecks: TimeCheck
}

#[derive(Debug, RustcDecodable)]
pub struct SummaryDetail{
     severity: String,
     summary: String,
}

#[derive(Debug, RustcDecodable)]
pub struct SubHealth{
    health_services: Vec<HealthService>,
}

#[derive(Debug, RustcDecodable)]
pub struct HealthService{
    mons: Vec<MonHealthDetails>
}

#[derive(Debug, RustcDecodable)]
pub struct TimeCheck{
    epoch: u64,
    mons: Vec<MonHealth>,
    round: u64,
    round_status: String,
}

#[derive(Debug, RustcDecodable)]
pub struct MonHealthDetails{
    avail_percent: u64,
    health: String,
    kb_avail: u64,
    kb_total: u64,
    kb_used: u64,
    last_updated: String,
    name: String,
    store_stats: MonStoreStat,
}

#[derive(Debug, RustcDecodable)]
pub struct MonStoreStat{
    bytes_log: u64,
    bytes_misc: u64,
    bytes_sst: u64,
    bytes_total: u64,
    last_updated: String,
}

#[derive(Debug, RustcDecodable)]
pub struct MonHealth{
    health: String,
    latency: String,
    name: String,
    skew: String,
}

pub struct MdsMap{
    by_rank: Vec<String>,
    epoch: u64,
    in_map: u64,
    max: u64,
    up: u64,
}

impl Decodable for MdsMap{
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, D::Error>{
        decoder.read_struct("root", 0, |decoder| {
          decoder.read_struct_field("mdsmap", 0, |decoder| {
             Ok(MdsMap{
              by_rank: try!(decoder.read_struct_field("by_rank", 0, |decoder| Decodable::decode(decoder))),
              epoch: try!(decoder.read_struct_field("epoch", 0, |decoder| Decodable::decode(decoder))),
              in_map: try!(decoder.read_struct_field("in", 0, |decoder| Decodable::decode(decoder))),
              max: try!(decoder.read_struct_field("max", 0, |decoder| Decodable::decode(decoder))),
              up: try!(decoder.read_struct_field("up", 0, |decoder| Decodable::decode(decoder)))
            })
          })
        })
    }
}

#[derive(Debug, RustcDecodable)]
pub struct MonMap{
    created: String,
    epoch: u64,
    fsid: String,
    modified: String,
    mons: Vec<Mon>
}

#[derive(Debug, RustcDecodable)]
pub struct PgMap{
    bytes_avail: u64,
    bytes_total: u64,
    bytes_used: u64,
    data_bytes: u64,
    num_pgs: u64,
    pgs_by_state: Vec<PgState>,
    version: u64,
}

#[derive(Debug, RustcDecodable)]
pub struct OsdMap{
    epoch: u64,
    full: bool,
    nearfull: bool,
    num_in_osds: u64,
    num_osds: u64,
    num_up_osds: u64
}

#[derive(Debug, RustcDecodable)]
pub struct PgState{
    count: u64,
    state_name: String,
}

#[derive(Debug, RustcDecodable)]
pub struct Mon{
    addr: String,
    name: String,
    rank: u64,
}
