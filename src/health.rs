extern crate rustc_serialize;
extern crate time;
extern crate uuid;

use rustc_serialize::{json, Decoder, Decodable};

#[test]
fn test_autodecode(){
    let test_str = "{\"health\":{\"health\":{\"health_services\":[{\"mons\":[{\"name\":\"chris-local-machine-1\",\"kb_total\":232205304,\"kb_used\":81823684,\"kb_avail\":138563228,\"avail_percent\":59,\"last_updated\":\"2015-10-07 12:19:51.281273\",\"store_stats\":{\"bytes_total\":5408347,\"bytes_sst\":0,\"bytes_log\":4166001,\"bytes_misc\":1242346,\"last_updated\":\"0.000000\"},\"health\":\"HEALTH_OK\"},{\"name\":\"chris-local-machine-2\",\"kb_total\":232205304,\"kb_used\":79803236,\"kb_avail\":140583676,\"avail_percent\":60,\"last_updated\":\"2015-10-07 12:19:23.247120\",\"store_stats\":{\"bytes_total\":6844874,\"bytes_sst\":0,\"bytes_log\":5602535,\"bytes_misc\":1242339,\"last_updated\":\"0.000000\"},\"health\":\"HEALTH_OK\"},{\"name\":\"chris-local-machine-3\",\"kb_total\":232205304,\"kb_used\":78650196,\"kb_avail\":141736716,\"avail_percent\":61,\"last_updated\":\"2015-10-07 12:19:07.182466\",\"store_stats\":{\"bytes_total\":6531182,\"bytes_sst\":0,\"bytes_log\":5288894,\"bytes_misc\":1242288,\"last_updated\":\"0.000000\"},\"health\":\"HEALTH_OK\"}]}]},\"summary\":[],\"timechecks\":{\"epoch\":6,\"round\":38,\"round_status\":\"finished\",\"mons\":[{\"name\":\"chris-local-machine-1\",\"skew\":\"0.000000\",\"latency\":\"0.000000\",\"health\":\"HEALTH_OK\"},{\"name\":\"chris-local-machine-2\",\"skew\":\"0.000000\",\"latency\":\"0.000977\",\"health\":\"HEALTH_OK\"},{\"name\":\"chris-local-machine-3\",\"skew\":\"0.000000\",\"latency\":\"0.000818\",\"health\":\"HEALTH_OK\"}]},\"overall_status\":\"HEALTH_OK\",\"detail\":[]},\"fsid\":\"1bb15abc-4158-11e5-b499-00151737cf98\",\"election_epoch\":6,\"quorum\":[0,1,2],\"quorum_names\":[\"chris-local-machine-1\",\"chris-local-machine-2\",\"chris-local-machine-3\"],\"monmap\":{\"epoch\":2,\"fsid\":\"1bb15abc-4158-11e5-b499-00151737cf98\",\"modified\":\"2015-10-07 10:45:23.255204\",\"created\":\"0.000000\",\"mons\":[{\"rank\":0,\"name\":\"chris-local-machine-1\",\"addr\":\"10.0.2.22:6789/0\"},{\"rank\":1,\"name\":\"chris-local-machine-2\",\"addr\":\"10.0.2.78:6789/0\"},{\"rank\":2,\"name\":\"chris-local-machine-3\",\"addr\":\"10.0.2.141:6789/0\"}]},\"osdmap\":{\"osdmap\":{\"epoch\":9,\"num_osds\":3,\"num_up_osds\":3,\"num_in_osds\":3,\"full\":false,\"nearfull\":false}},\"pgmap\":{\"pgs_by_state\":[{\"state_name\":\"active+clean\",\"count\":192}],\"version\":487,\"num_pgs\":192,\"data_bytes\":4970896648,\"bytes_used\":252251439104,\"bytes_avail\":424777154560,\"bytes_total\":713334693888,\"write_bytes_sec\":26793300,\"op_per_sec\":8},\"mdsmap\":{\"epoch\":1,\"up\":0,\"in\":0,\"max\":1,\"by_rank\":[]}}";
    let decoded: CephHealth = json::decode(test_str).unwrap();
    println!("Decoded: {:?}", decoded);
}

#[derive(Debug, RustcDecodable)]
pub struct CephHealth  {
    pub election_epoch: u64,
    pub fsid: uuid::Uuid,
    pub health: Health,
    pub quorum: Vec<u64>,
    pub quorum_names: Vec<String>,
    pub pgmap: PgMap,
    pub monmap: MonMap,
    pub osdmap: OsdMap,
}

fn get_time()->f64{
    let now = time::now();
    let milliseconds_since_epoch = now.to_timespec().sec * 1000;
    return milliseconds_since_epoch as f64;
}

impl CephHealth{
    pub fn decode(json_data: &str)->Result<Self, json::DecoderError>{
        let decode: CephHealth = try!(json::decode(json_data));
        return Ok(decode);
    }

    pub fn to_json(&self)->String{
        let ops_per_sec = match self.pgmap.op_per_sec{
            Some(ops) => ops,
            None => 0,
        };
        let write_bytes_sec = match self.pgmap.write_bytes_sec{
            Some(write_bytes_sec) => write_bytes_sec,
            None => 0,
        };
        let read_bytes_sec = match self.pgmap.read_bytes_sec{
            Some(read_bytes_sec) => read_bytes_sec,
            None => 0,
        };

        format!("{{\"fsid\":\"{}\",\"ops_per_sec\": \"{}\",\"write_bytes_sec\": \"{}\", \"read_bytes_sec\": \"{}\", \"data\":\"{}\", \
            \"bytes_used\":{}, \"bytes_avail\":{}, \"bytes_total\":\"{}\", \"postDate\": {}}}",
            self.fsid.to_hyphenated_string(),
            ops_per_sec,
            write_bytes_sec,
            read_bytes_sec,
            self.pgmap.data_bytes,
            self.pgmap.bytes_used,
            self.pgmap.bytes_avail,
            self.pgmap.bytes_total,
            get_time())
    }

//     fn to_carbon_string(&self, root_key: &String) -> String {
//         format!( r#"{root_key}.{} {} {timestamp}
// {root_key}.{} {} {timestamp}
// {root_key}.{} {} {timestamp}
// {root_key}.{} {} {timestamp}
// {root_key}.{} {} {timestamp}
// {root_key}.{} {} {timestamp}
// {root_key}.{} {} {timestamp}
// {root_key}.{} {} {timestamp}
// "#, "osds", self.num_osds, "ops", self.ops, "write_bytes", self.write_bytes_sec,
// "read_bytes", self.read_bytes_sec, "data", self.data, "used", self.bytes_used,
// "avail", self.bytes_avail, "total", self.bytes_total, root_key = root_key.clone(), timestamp = get_time())
//     }
}

#[derive(Debug, RustcDecodable)]
pub struct Health{
    pub detail: Vec<String>,
    pub health: SubHealth,
    pub overall_status: String,
    pub summary: Vec<SummaryDetail>,
    pub timechecks: TimeCheck,
}

#[derive(Debug, RustcDecodable)]
pub struct SummaryDetail{
     pub severity: String,
     pub summary: String,
}

#[derive(Debug, RustcDecodable)]
pub struct SubHealth{
    pub health_services: Vec<HealthService>,
}

#[derive(Debug, RustcDecodable)]
pub struct HealthService{
    pub mons: Vec<MonHealthDetails>
}

#[derive(Debug, RustcDecodable)]
pub struct TimeCheck{
    pub epoch: u64,
    pub mons: Vec<MonHealth>,
    pub round: u64,
    pub round_status: String,
}

#[derive(Debug, RustcDecodable)]
pub struct MonHealthDetails{
    pub avail_percent: u64,
    pub health: String,
    pub kb_avail: u64,
    pub kb_total: u64,
    pub kb_used: u64,
    pub last_updated: String,
    pub name: String,
    pub store_stats: MonStoreStat,
}

#[derive(Debug, RustcDecodable)]
pub struct MonStoreStat{
    pub bytes_log: u64,
    pub bytes_misc: u64,
    pub bytes_sst: u64,
    pub bytes_total: u64,
    pub last_updated: String,
}

#[derive(Debug, RustcDecodable)]
pub struct MonHealth{
    pub health: String,
    pub latency: String,
    pub name: String,
    pub skew: String,
}

pub struct MdsMap{
    pub epoch: u64,
    pub by_rank: Vec<String>,
    pub in_map: u64,
    pub max: u64,
    pub up: u64,
}

impl Decodable for MdsMap{
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, D::Error>{
        decoder.read_struct("root", 0, |decoder| {
          decoder.read_struct_field("mdsmap", 0, |decoder| {
             Ok(MdsMap{
              epoch: try!(decoder.read_struct_field("epoch", 0, |decoder| Decodable::decode(decoder))),
              up: try!(decoder.read_struct_field("up", 0, |decoder| Decodable::decode(decoder))),
              in_map: try!(decoder.read_struct_field("in", 0, |decoder| Decodable::decode(decoder))),
              max: try!(decoder.read_struct_field("max", 0, |decoder| Decodable::decode(decoder))),
              by_rank: try!(decoder.read_struct_field("by_rank", 0, |decoder| Decodable::decode(decoder))),
            })
          })
        })
    }
}

#[derive(Debug, RustcDecodable)]
pub struct MonMap{
    pub epoch: u64,
    pub fsid: String,
    pub modified: String,
    pub created: String,
    pub mons: Vec<Mon>
}

#[derive(Debug, RustcDecodable)]
pub struct PgMap{
    pub bytes_avail: u64,
    pub bytes_total: u64,
    pub bytes_used: u64,
    pub read_bytes_sec: Option<u64>,
    pub write_bytes_sec: Option<u64>,
    pub op_per_sec: Option<u64>,
    pub data_bytes: u64,
    pub num_pgs: u64,
    pub pgs_by_state: Vec<PgState>,
    pub version: u64,
}

#[derive(Debug, RustcDecodable)]
pub struct OsdMap{
    osdmap: SubOsdMap,
}

#[derive(Debug, RustcDecodable)]
pub struct SubOsdMap{
    pub epoch: u64,
    pub num_osds: u64,
    pub num_up_osds: u64,
    pub num_in_osds: u64,
    pub full: bool,
    pub nearfull: bool,
}

#[derive(Debug, RustcDecodable)]
pub struct PgState{
    pub count: u64,
    pub state_name: String,
}

#[derive(Debug, RustcDecodable)]
pub struct Mon{
    pub rank: u64,
    pub name: String,
    pub addr: String,
}
