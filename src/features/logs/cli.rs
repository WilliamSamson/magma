use std::io;

use clap::{Arg, ArgAction, Command};

use super::log_level::LogLevel;

pub(crate) struct ParsedArgs {
    pub(crate) input_path: Option<String>,
    pub(crate) startup_filter: StartupFilter,
}

pub(crate) struct StartupFilter {
    pub(crate) query: Option<String>,
    pub(crate) levels: Vec<LogLevel>,
}

pub(crate) fn parse_args() -> io::Result<ParsedArgs> {
    let matches = Command::new("magma")
        .about("Terminal workspace for structured logs")
        .version(env!("CARGO_PKG_VERSION"))
        .arg(Arg::new("input").value_name("INPUT"))
        .arg(
            Arg::new("filter")
                .long("filter")
                .value_name("RULE")
                .action(ArgAction::Append),
        )
        .get_matches();

    let filters = matches
        .get_many::<String>("filter")
        .map(|values| values.cloned().collect::<Vec<_>>())
        .unwrap_or_default();

    Ok(ParsedArgs {
        input_path: matches.get_one::<String>("input").cloned(),
        startup_filter: parse_filters(&filters)?,
    })
}

fn parse_filters(filters: &[String]) -> io::Result<StartupFilter> {
    let mut startup = StartupFilter {
        query: None,
        levels: Vec::new(),
    };

    for filter in filters {
        let Some((key, value)) = filter.split_once('=') else {
            return invalid_filter(filter, "expected key=value");
        };

        match key.trim() {
            "level" => {
                let level = LogLevel::from_text(value.trim());
                if level == LogLevel::Unknown {
                    return invalid_filter(filter, "unknown level");
                }
                startup.levels.push(level);
            }
            "query" | "search" => startup.query = Some(value.trim().to_string()),
            _ => return invalid_filter(filter, "supported keys are level, query, search"),
        }
    }

    Ok(startup)
}

fn invalid_filter(filter: &str, message: &str) -> io::Result<StartupFilter> {
    Err(io::Error::new(
        io::ErrorKind::InvalidInput,
        format!("invalid --filter '{filter}': {message}"),
    ))
}
