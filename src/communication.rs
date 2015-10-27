extern crate time;
extern crate output_args;
use std::io::prelude::*;
use std::net::TcpStream;
use influent::create_client;
use influent::client::Client;
use influent::client::Credentials;
use influent::measurement::Measurement;
use output_args::*;

pub fn send_to_influx(args: &output_args::Args, measurement: Measurement) {
    let influx = &args.influx.clone().unwrap();
    let credentials = Credentials {
        username: influx.user.as_ref(),
        password: influx.password.as_ref(),
        database: "ceph",
    };
    let host = format!("http://{}:{}", influx.host, influx.port);
    let hosts = vec![host.as_ref()];
    let client = create_client(credentials, hosts);

    let res = client.write_one(measurement, None);

    debug!("{:?}", res);
}

pub fn send_to_carbon(args: &output_args::Args, carbon_data: String) -> Result<(), String> {
    let carbon = &args.carbon.clone().unwrap();

    let carbon_host = &carbon.host;
    let carbon_port = &carbon.port;
    let carbon_url = format!("{}:{}", carbon_host, carbon_port);
    let carbon_string: &str = carbon_url.as_ref();
    let mut stream = try!(TcpStream::connect(carbon_string).map_err(|e| e.to_string()));
    let bytes_written = try!(stream.write(&carbon_data.into_bytes()[..])
                                   .map_err(|e| e.to_string()));
    debug!("Wrote: {} bytes to graphite", &bytes_written);
    Ok(())
}