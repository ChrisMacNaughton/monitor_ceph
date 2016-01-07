extern crate time;
extern crate uuid;
extern crate hyper;

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
        let _ = client.post(url)
            .body(body)
            .header(ContentType::json())
            .send();
    }
}
