#[derive(Debug, Clone)]
pub(crate) struct Options {
    pub port: u16,
}

impl Options {
    pub fn from_claps(matches: &clap::ArgMatches<'_>) -> Options {
        Options {
            // All options are either required or have a default value, so unwrap should be safe
            port: matches
                .value_of("port")
                .map(|e| e.parse())
                .unwrap()
                .expect("Invalid port number"),
        }
    }
}
