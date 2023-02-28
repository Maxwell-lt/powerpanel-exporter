use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "powerpanek exporter")]
pub(crate) struct Options {
    /// Sets the port the exporter uses
    #[structopt(short, long, default_value = "9102")]
    pub port: u16,
}
