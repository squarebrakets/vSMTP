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
use crate::Args;
use tracing_subscriber::fmt::writer::{MakeWriterExt, OptionalWriter};
use tracing_subscriber::{filter, fmt, prelude::*, EnvFilter};
use vsmtp_common::collection;
use vsmtp_common::re::anyhow::{self, Context};
use vsmtp_common::re::{either, log};
use vsmtp_config::field::{FieldServerLogSystem, SyslogFormat, SyslogSocket};
use vsmtp_config::Config;

struct SyslogWriter {
    logger: either::Either<
        syslog::Logger<syslog::LoggerBackend, syslog::Formatter3164>,
        syslog::Logger<syslog::LoggerBackend, syslog::Formatter5424>,
    >,
}

impl std::io::Write for SyslogWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self.logger {
            either::Either::Left(ref mut logger) => syslog::LogFormat::format(
                &logger.formatter,
                &mut logger.backend,
                syslog::Severity::LOG_WARNING,
                std::str::from_utf8(buf).unwrap_or("utf-8 error").to_owned(),
            ),
            either::Either::Right(ref mut logger) => syslog::LogFormat::format(
                &logger.formatter,
                &mut logger.backend,
                syslog::Severity::LOG_WARNING,
                (
                    0,
                    collection! {},
                    std::str::from_utf8(buf).unwrap_or("utf-8 error").to_owned(),
                ),
            ),
        }
        .map(|_| buf.len())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.description()))
    }

    fn flush(&mut self) -> std::io::Result<()> {
        either::for_both!(&mut self.logger, logger => logger.backend.flush())
    }
}

struct MakeSyslogWriter {
    config: (SyslogFormat, SyslogSocket),
}

impl<'a> fmt::MakeWriter<'a> for MakeSyslogWriter {
    // NOTE: if the syslog failed to initialize, is it written to stdout ?
    type Writer = fmt::writer::OptionalWriter<SyslogWriter>;

    fn make_writer(&self) -> Self::Writer {
        fn get_3164() -> syslog::Formatter3164 {
            syslog::Formatter3164 {
                facility: syslog::Facility::LOG_MAIL,
                hostname: None,
                ..Default::default()
            }
        }

        fn get_5424() -> syslog::Formatter5424 {
            syslog::Formatter5424 {
                facility: syslog::Facility::LOG_MAIL,
                hostname: None,
                ..Default::default()
            }
        }

        let result = match (self.config.0, &self.config.1) {
            (SyslogFormat::Rfc3164, SyslogSocket::Udp { local, server }) => {
                syslog::udp(get_3164(), local, server).map(|logger| {
                    OptionalWriter::some(SyslogWriter {
                        logger: either::Left(logger),
                    })
                })
            }
            (SyslogFormat::Rfc3164, SyslogSocket::Tcp { server }) => {
                syslog::tcp(get_3164(), server).map(|logger| {
                    OptionalWriter::some(SyslogWriter {
                        logger: either::Left(logger),
                    })
                })
            }
            (SyslogFormat::Rfc3164, SyslogSocket::Unix { path }) => {
                syslog::unix_custom(get_3164(), path).map(|logger| {
                    OptionalWriter::some(SyslogWriter {
                        logger: either::Left(logger),
                    })
                })
            }
            (SyslogFormat::Rfc5424, SyslogSocket::Udp { local, server }) => {
                syslog::udp(get_5424(), local, server).map(|logger| {
                    OptionalWriter::some(SyslogWriter {
                        logger: either::Right(logger),
                    })
                })
            }
            (SyslogFormat::Rfc5424, SyslogSocket::Tcp { server }) => {
                syslog::tcp(get_5424(), server).map(|logger| {
                    OptionalWriter::some(SyslogWriter {
                        logger: either::Right(logger),
                    })
                })
            }
            (SyslogFormat::Rfc5424, SyslogSocket::Unix { path }) => {
                syslog::unix_custom(get_5424(), path).map(|logger| {
                    OptionalWriter::some(SyslogWriter {
                        logger: either::Right(logger),
                    })
                })
            }
        };

        match result {
            Ok(logger) => logger,
            Err(e) => {
                eprintln!("{}", e);
                OptionalWriter::none()
            }
        }
    }
}

#[cfg(debug_assertions)]
macro_rules! get_fmt {
    () => {
        fmt::layer()
            .pretty()
            .with_file(true)
            .with_line_number(true)
            .with_thread_ids(true)
            .with_target(true)
            .with_ansi(false)
    };
}

#[cfg(not(debug_assertions))]
macro_rules! get_fmt {
    () => {
        fmt::layer()
            .compact()
            .with_thread_ids(false)
            .with_target(false)
            .with_ansi(false)
    };
}

/// Initialize the tracing subsystem.
///
/// # Errors
///
/// * The logs path in the configuration file are invalid.
/// * Failed to initialize the tracing subsystem.
pub fn initialize(args: &Args, config: &Config) -> anyhow::Result<()> {
    std::fs::create_dir_all(config.server.logs.filepath.clone())
        .context("Cannot create `server.logs` directory")?;

    let writer_backend = tracing_appender::rolling::daily(&config.server.logs.filepath, "vsmtp");
    let writer_backend = writer_backend.with_filter(|metadata| {
        metadata.target() != "vsmtp_rule_engine::api::logging::logging_rhai"
    });

    std::fs::create_dir_all(config.app.logs.filepath.clone())
        .context("Cannot create `app.logs` directory")?;

    let writer_app = tracing_appender::rolling::daily(&config.app.logs.filepath, "app");
    let writer_app = writer_app.with_filter(|metadata| {
        metadata.target() == "vsmtp_rule_engine::api::logging::logging_rhai"
    });

    let subscriber = tracing_subscriber::registry().with(
        EnvFilter::builder().try_from_env().unwrap_or_else(|_| {
            let mut e = EnvFilter::default();
            for i in &config.server.logs.level {
                e = e.add_directive(i.clone());
            }
            e
        }),
    );

    #[cfg(feature = "tokio_console")]
    let subscriber = subscriber.with(console_subscriber::spawn());

    let subscriber = subscriber
        .with(get_fmt!().with_writer(writer_backend))
        .with(get_fmt!().with_writer(writer_app));

    if let Some(system_log_config) = &config.server.logs.system {
        match &system_log_config {
            FieldServerLogSystem::Syslogd {
                level,
                format,
                socket,
            } => {
                let subscriber = subscriber.with(
                    get_fmt!()
                        .with_writer(
                            MakeSyslogWriter {
                                config: (*format, socket.clone()),
                            }
                            .with_max_level(match level {
                                log::LevelFilter::Off => unimplemented!(),
                                log::LevelFilter::Error => tracing::Level::ERROR,
                                log::LevelFilter::Warn => tracing::Level::WARN,
                                log::LevelFilter::Info => tracing::Level::INFO,
                                log::LevelFilter::Debug => tracing::Level::DEBUG,
                                log::LevelFilter::Trace => tracing::Level::TRACE,
                            }),
                        )
                        .without_time(),
                );

                if args.no_daemon {
                    subscriber
                        .with(get_fmt!().with_writer(std::io::stdout).with_ansi(true))
                        .try_init()
                } else {
                    subscriber.try_init()
                }
            }
            FieldServerLogSystem::Journald { level } => {
                let min_level = match level {
                    log::LevelFilter::Off => unimplemented!(),
                    log::LevelFilter::Error => tracing::Level::ERROR,
                    log::LevelFilter::Warn => tracing::Level::WARN,
                    log::LevelFilter::Info => tracing::Level::INFO,
                    log::LevelFilter::Debug => tracing::Level::DEBUG,
                    log::LevelFilter::Trace => tracing::Level::TRACE,
                };

                let subscriber = subscriber.with(
                    tracing_journald::layer()
                        .map_err(|e| anyhow::anyhow!("{e}"))?
                        .with_filter(filter::filter_fn(move |i| *i.level() <= min_level)),
                );

                if args.no_daemon {
                    subscriber
                        .with(get_fmt!().with_writer(std::io::stdout).with_ansi(true))
                        .try_init()
                } else {
                    subscriber.try_init()
                }
            }
        }
    } else if args.no_daemon {
        subscriber
            .with(get_fmt!().with_writer(std::io::stdout).with_ansi(true))
            .try_init()
    } else {
        subscriber.try_init()
    }
    .map_err(|e| anyhow::anyhow!("{e}"))
}