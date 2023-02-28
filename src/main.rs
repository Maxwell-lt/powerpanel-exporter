use std::process::Command;
use std::str::from_utf8;
mod options;
use options::Options;
use regex::Regex;
use anyhow::Result;
use rocket::Config;
use structopt::StructOpt;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate rocket;

struct UPSData {
    utility_voltage: u32,
    output_voltage: u32,
    battery_capacity: u8,
    remaining_runtime: u32,
    load_watts: u32,
    load_percent: u8,
}

fn get_command_output() -> Result<String> {
    Ok(from_utf8(&Command::new("pwrstat").arg("-status").output()?.stdout)?.to_owned())
}

fn parse_status_message(message: &str) -> Result<UPSData> {
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

#[get("/metrics")]
fn metrics() -> Result<String, rocket::response::Debug<anyhow::Error>> {
    let ups_data: UPSData = parse_status_message(&get_command_output()?)?;
    let mut output = String::new();
    
    output.push_str("# HELP ups_input_voltage Utility voltage\n");
    output.push_str("# TYPE ups_input_voltage gauge\n");
    output.push_str(&format!("ups_input_voltage {}\n", ups_data.utility_voltage));

    output.push_str("# HELP ups_output_voltage Output voltage\n");
    output.push_str("# TYPE ups_output_voltage gauge\n");
    output.push_str(&format!("ups_output_voltage {}\n", ups_data.output_voltage));

    output.push_str("# HELP ups_battery_capacity Battery capacity\n");
    output.push_str("# TYPE ups_battery_capacity gauge\n");
    output.push_str(&format!("ups_battery_capacity {}\n", ups_data.battery_capacity));
    
    output.push_str("# HELP ups_remaining_runtime Remaining runtime\n");
    output.push_str("# TYPE ups_remaining_runtime gauge\n");
    output.push_str(&format!("ups_remaining_runtime {}\n", ups_data.remaining_runtime));

    output.push_str("# HELP ups_load_watts Load in watts\n");
    output.push_str("# TYPE ups_load_watts gauge\n");
    output.push_str(&format!("ups_load_watts {}\n", ups_data.load_watts));

    output.push_str("# HELP ups_load_percent Load percentage\n");
    output.push_str("# TYPE ups_load_percent gauge\n");
    output.push_str(&format!("ups_load_percent {}\n", ups_data.load_percent));

    Ok(output)
}

#[launch]
fn rocket() -> _ {
    let parameters: Options = Options::from_args();

    let config = Config::figment()
        .merge(("port", parameters.port));

    rocket::custom(config).mount("/", routes![metrics])
}
