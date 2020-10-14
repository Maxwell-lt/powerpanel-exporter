use clap::{crate_authors, crate_name, crate_version, App, Arg};
use prometheus_exporter_base::{render_prometheus, MetricType, PrometheusMetric};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::process::Command;
use std::str::from_utf8;
mod exporter_error;
use exporter_error::ExporterError;
mod options;
use options::Options;
use regex::Regex;
#[macro_use]
extern crate lazy_static;

struct UPSData {
    utility_voltage: u32,
    output_voltage: u32,
    battery_capacity: u8,
    remaining_runtime: u32,
    load_watts: u32,
    load_percent: u8,
}

fn get_command_output() -> Result<String, ExporterError> {
    Ok(from_utf8(&Command::new("pwrstat").arg("-status").output()?.stdout)?.to_owned())
}

fn parse_status_message(message: &str) -> Result<UPSData, ExporterError> {
    lazy_static! {
        static ref RE: Regex =
            Regex::new("Utility Voltage[. ]+(?P<util_volts>\\d+)\\sV\\n\\s+Output Voltage[. ]+(?P<out_volts>\\d+)\\sV\\n\\s+Battery Capacity[. ]+(?P<batt_cap>\\d+) %\\n\\s+Remaining Runtime[. ]+(?P<runtime>\\d+) min.\\n\\s+Load[. ]+(?P<load_watts>\\d+) Watt\\((?P<load_percent>\\d+)")
            .unwrap();
    }

    let captures: regex::Captures = RE.captures(message).unwrap();
    Ok(UPSData {
        utility_voltage: captures.name("util_volts").unwrap().as_str().parse()?,
        output_voltage: captures.name("out_volts").unwrap().as_str().parse()?,
        battery_capacity: captures.name("batt_cap").unwrap().as_str().parse()?,
        remaining_runtime: captures.name("runtime").unwrap().as_str().parse()?,
        load_watts: captures.name("load_watts").unwrap().as_str().parse()?,
        load_percent: captures.name("load_percent").unwrap().as_str().parse()?,
    })
}

#[tokio::main]
async fn main() {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .required(false)
                .help("Sets the port the exporter uses")
                .default_value("9102")
                .takes_value(true),
        )
        .get_matches();

    let options = Options::from_claps(&matches);

    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), options.port);
    render_prometheus(addr, options, |_request, _options| async move {
        let ups_data: UPSData = parse_status_message(&get_command_output()?)?;

        let input_voltage =
            PrometheusMetric::new("ups_input_voltage", MetricType::Gauge, "Utility voltage");
        let output_voltage =
            PrometheusMetric::new("ups_output_voltage", MetricType::Gauge, "Output voltage");
        let battery_capacity = PrometheusMetric::new(
            "ups_battery_capacity",
            MetricType::Gauge,
            "Battery capacity",
        );
        let remaining_runtime = PrometheusMetric::new(
            "ups_remaining_runtime",
            MetricType::Gauge,
            "Remaining runtime",
        );
        let load_watts =
            PrometheusMetric::new("ups_load_watts", MetricType::Gauge, "Load in watts");
        let load_percent =
            PrometheusMetric::new("ups_load_percent", MetricType::Gauge, "Load percentage");

        let mut output = input_voltage.render_header();
        output.push_str(&input_voltage.render_sample(None, ups_data.utility_voltage, None));

        output.push_str(&output_voltage.render_header());
        output.push_str(&output_voltage.render_sample(None, ups_data.output_voltage, None));

        output.push_str(&battery_capacity.render_header());
        output.push_str(&battery_capacity.render_sample(None, ups_data.battery_capacity, None));

        output.push_str(&remaining_runtime.render_header());
        output.push_str(&remaining_runtime.render_sample(None, ups_data.remaining_runtime, None));

        output.push_str(&load_watts.render_header());
        output.push_str(&load_watts.render_sample(None, ups_data.load_watts, None));

        output.push_str(&load_percent.render_header());
        output.push_str(&load_percent.render_sample(None, ups_data.load_percent, None));

        Ok(output)
    })
    .await;
}
