/*
 * vSMTP mail transfer agent
 * Copyright (C) 2022 viridIT SAS
 *
 * This program is free software: you can redistribute it and/or modify it under
 * the terms of the GNU General Public License as published by the Free Software
 * Foundation, either version 3 of the License, or any later version.
 *
 * This program is distributed in the hope that it will be useful, but WITHOUT
 * ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
 * FOR A PARTICULAR PURPOSE.  See the GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along with
 * this program. If not, see https://www.gnu.org/licenses/.
 *
*/

///
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Timeout(pub std::time::Duration);

impl std::str::FromStr for Timeout {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(
            humantime::parse_duration(s).map_err(anyhow::Error::new)?,
        ))
    }
}

///
#[derive(Debug, clap::Parser, PartialEq, Eq)]
#[clap(about, author)]
pub struct Args {
    /// Print the version and exit.
    #[clap(short, long, action)]
    pub version: bool,

    // NOTE: Can't use `PathBuf`, `default_value_t` needs `std::fmt::Display`.
    /// Path of the vSMTP configuration file. (vSL format)
    #[arg(default_value_t = Args::default_config_location())]
    #[clap(short, long, action)]
    pub config: String,

    /// Absolute path of a dotenv file.
    #[clap(short, long, action)]
    pub env: Option<String>,

    /// Commands.
    #[clap(subcommand)]
    pub command: Option<Commands>,

    /// Do not run the program as a daemon.
    #[clap(short, long, action)]
    pub no_daemon: bool,

    /// Output to stdout.
    #[clap(long, action)]
    pub stdout: bool,

    /// Make the server stop after a delay. (human readable format)
    #[clap(short, long, action)]
    pub timeout: Option<Timeout>,
}

impl Args {
    fn default_config_location() -> String {
        String::from("/etc/vsmtp/vsmtp.vsl")
    }
}

///
#[derive(Debug, clap::Subcommand, PartialEq, Eq)]
pub enum Commands {
    /// Show the loaded config (as serialized json format)
    ConfigShow,
    /// Show the difference between the loaded config and the default one
    ConfigDiff,
}

#[cfg(test)]
mod tests {

    use super::*;

    #[allow(clippy::too_many_lines)]
    #[test]
    fn parse_arg() {
        assert!(<Args as clap::Parser>::try_parse_from([""]).is_ok());

        assert_eq!(
            Args {
                version: false,
                command: None,
                config: "path".to_string(),
                env: None,
                no_daemon: false,
                stdout: false,
                timeout: None
            },
            <Args as clap::Parser>::try_parse_from(["", "-c", "path"]).unwrap()
        );

        assert_eq!(
            Args {
                version: false,
                command: None,
                config: Args::default_config_location(),
                env: Some("env".to_string()),
                no_daemon: false,
                stdout: false,
                timeout: None
            },
            <Args as clap::Parser>::try_parse_from(["", "--env", "env"]).unwrap()
        );

        assert_eq!(
            Args {
                version: false,
                command: Some(Commands::ConfigShow),
                config: "path".to_string(),
                env: None,
                no_daemon: false,
                stdout: false,
                timeout: None
            },
            <Args as clap::Parser>::try_parse_from(["", "-c", "path", "config-show"]).unwrap()
        );

        assert_eq!(
            Args {
                version: false,
                command: Some(Commands::ConfigDiff),
                config: "path".to_string(),
                env: None,
                no_daemon: false,
                stdout: false,
                timeout: None
            },
            <Args as clap::Parser>::try_parse_from(["", "-c", "path", "config-diff"]).unwrap()
        );

        assert_eq!(
            Args {
                version: true,
                command: None,
                config: Args::default_config_location(),
                env: None,
                no_daemon: false,
                stdout: false,
                timeout: None
            },
            <Args as clap::Parser>::try_parse_from(["", "--version"]).unwrap()
        );

        assert_eq!(
            Args {
                version: false,
                command: None,
                config: "path".to_string(),
                env: None,
                no_daemon: true,
                stdout: false,
                timeout: None
            },
            <Args as clap::Parser>::try_parse_from(["", "-c", "path", "--no-daemon"]).unwrap()
        );

        assert_eq!(
            Args {
                version: false,
                command: None,
                config: "path".to_string(),
                env: None,
                no_daemon: true,
                stdout: true,
                timeout: Some(Timeout(std::time::Duration::from_secs(1)))
            },
            <Args as clap::Parser>::try_parse_from([
                "",
                "-c",
                "path",
                "--no-daemon",
                "--stdout",
                "--timeout",
                "1s"
            ])
            .unwrap()
        );

        assert_eq!(
            Args {
                version: false,
                command: None,
                config: "path".to_string(),
                env: Some("env".to_string()),
                no_daemon: true,
                stdout: true,
                timeout: Some(Timeout(std::time::Duration::from_secs(1)))
            },
            <Args as clap::Parser>::try_parse_from([
                "",
                "-c",
                "path",
                "--env",
                "env",
                "--no-daemon",
                "--stdout",
                "--timeout",
                "1s"
            ])
            .unwrap()
        );
    }
}
