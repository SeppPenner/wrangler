#![allow(clippy::redundant_closure)]

#[macro_use]
extern crate text_io;

use std::env;
use std::str::FromStr;

use clap::{App, AppSettings, Arg, SubCommand};
use commands::HTTPMethod;

use log::info;

mod commands;
mod http;
mod install;
mod installer;
mod settings;
mod terminal;

use crate::settings::project::ProjectType;
use exitfailure::ExitFailure;
use terminal::emoji;
use terminal::message;

fn main() -> Result<(), ExitFailure> {
    env_logger::init();
    if let Ok(me) = env::current_exe() {
        // If we're actually running as the installer then execute our
        // self-installation, otherwise just continue as usual.
        if me
            .file_stem()
            .and_then(|s| s.to_str())
            .expect("executable should have a filename")
            .starts_with("wrangler-init")
        {
            installer::install();
        }
    }
    Ok(run()?)
}

fn run() -> Result<(), failure::Error> {
    let matches = App::new(format!("{}{} wrangler", emoji::WORKER, emoji::SPARKLES))
        .version(env!("CARGO_PKG_VERSION"))
        .author("ashley g williams <ashley666ashley@gmail.com>")
        .setting(AppSettings::ArgRequiredElseHelp)
        .setting(AppSettings::DeriveDisplayOrder)
        .subcommand(
            SubCommand::with_name("kv")
                .about(&*format!(
                    "{} Interact with your Workers KV Store",
                    emoji::KV
                ))
                .subcommand(
                    SubCommand::with_name("create")
                        .arg(
                            Arg::with_name("title")
                        )
                )
                .subcommand(
                    SubCommand::with_name("delete")
                        .arg(
                            Arg::with_name("id")
                        )
                )
                .subcommand(
                    SubCommand::with_name("rename")
                        .arg(
                            Arg::with_name("id")
                        )
                        .arg(
                            Arg::with_name("title")
                        )
                )
                .subcommand(
                    SubCommand::with_name("list")
                )
                .subcommand(
                    SubCommand::with_name("write")
                        .arg(
                            Arg::with_name("expiration")
                            .short("e")
                            .long("expiration")
                            .takes_value(true)
                            .value_name("SECONDS")
                            .help("the time, measured in number of seconds since the UNIX epoch, at which the entries should expire"),
                        )
                        .arg(
                            Arg::with_name("time-to-live")
                            .short("t")
                            .long("ttl")
                            .value_name("SECONDS")
                            .takes_value(true)
                            .help("the number of seconds for which the entries should be visible before they expire. At least 60"),
                        )
                        .subcommand(
                            SubCommand::with_name("bulk")
                                .about("upload multiple key-value pairs at once")
                                .arg(
                                    Arg::with_name("id")
                                        .help("the id of your Workers KV namespace")
                                        .index(1),
                                )
                                .arg(
                                    Arg::with_name("filename")
                                    .help("the json file of key-value pairs to upload, in form [{\"key\":..., \"value\":...}\"...]")
                                    .index(2),
                                )
                                .arg(
                                    Arg::with_name("base64")
                                    .long("base64")
                                    .takes_value(false)
                                    .help("the server should base64 decode the value before storing it. Useful for writing values that wouldn't otherwise be valid JSON strings, such as images."),
                                )
                        )
                )
        )
        .subcommand(
            SubCommand::with_name("generate")
                .about(&*format!(
                    "{} Generate a new worker project",
                    emoji::DANCERS
                ))
                .arg(
                    Arg::with_name("name")
                        .help("the name of your worker! defaults to 'worker'")
                        .index(1),
                )
                .arg(
                    Arg::with_name("template")
                        .help("a link to a github template! defaults to cloudflare/worker-template")
                        .index(2),
                )
                .arg(
                    Arg::with_name("type")
                        .short("t")
                        .long("type")
                        .takes_value(true)
                        .help("the type of project you want generated"),
                ),
        )
        .subcommand(
            SubCommand::with_name("init")
                .about(&*format!(
                    "{} Create a wrangler.toml for an existing project",
                    emoji::INBOX
                ))
                .arg(
                    Arg::with_name("name")
                        .help("the name of your worker! defaults to 'worker'")
                        .index(1),
                )
                .arg(
                    Arg::with_name("type")
                        .short("t")
                        .long("type")
                        .takes_value(true)
                        .help("the type of project you want generated"),
                ),
        )
        .subcommand(
            SubCommand::with_name("build")
                .about(&*format!(
                    "{} Build your worker",
                    emoji::CRAB
                )
            ),
        )
        .subcommand(
            SubCommand::with_name("preview")
                .about(&*format!(
                    "{} Preview your code temporarily on cloudflareworkers.com",
                    emoji::MICROSCOPE
                ))
                .arg(
                    Arg::with_name("method")
                        .help("Type of request to preview your worker with (get, post)")
                        .index(1),
                )
                .arg(
                    Arg::with_name("body")
                        .help("Body string to post to your preview worker request")
                        .index(2),
                ),
        )
        .subcommand(
            SubCommand::with_name("publish").about(&*format!(
                "{} Publish your worker to the orange cloud",
                emoji::UP
            ))
            .arg(
                Arg::with_name("release")
                    .long("release")
                    .takes_value(false)
                    .help("should this be published to a workers.dev subdomain or a domain name you have registered"),
             ),
        )
        .subcommand(
            SubCommand::with_name("config")
                .about(&*format!(
                    "{} Setup wrangler with your Cloudflare account",
                    emoji::SLEUTH
                )),
        )
        .subcommand(
            SubCommand::with_name("subdomain")
                .about(&*format!(
                    "{} Configure your workers.dev subdomain",
                    emoji::WORKER
                ))
                .arg(
                    Arg::with_name("name")
                        .help("the subdomain on workers.dev you'd like to reserve")
                        .index(1)
                        .required(true),
                ),
        )
        .subcommand(SubCommand::with_name("whoami").about(&*format!(
            "{} Retrieve your user info and test your auth config",
            emoji::SLEUTH
        )))
        .get_matches();

    if let Some(_matches) = matches.subcommand_matches("config") {
        println!("Enter email: ");
        let mut email: String = read!("{}\n");
        email.truncate(email.trim_end().len());
        println!("Enter api key: ");
        let mut api_key: String = read!("{}\n");
        api_key.truncate(api_key.trim_end().len());

        commands::global_config(email, api_key)?;
    } else if let Some(matches) = matches.subcommand_matches("generate") {
        let name = matches.value_of("name").unwrap_or("worker");
        let project_type = match matches.value_of("type") {
            Some(s) => Some(ProjectType::from_str(&s.to_lowercase())?),
            None => None,
        };

        let default_template = "https://github.com/cloudflare/worker-template";
        let template = matches.value_of("template").unwrap_or(match project_type {
            Some(ref pt) => match pt {
                ProjectType::Rust => "https://github.com/cloudflare/rustwasm-worker-template",
                _ => default_template,
            },
            _ => default_template,
        });

        info!(
            "Generate command called with template {}, and name {}",
            template, name
        );
        commands::generate(name, template, project_type)?;
    } else if let Some(matches) = matches.subcommand_matches("init") {
        let name = matches.value_of("name");
        let project_type = match matches.value_of("type") {
            Some(s) => Some(settings::project::ProjectType::from_str(&s.to_lowercase())?),
            None => None,
        };
        commands::init(name, project_type)?;
    } else if matches.subcommand_matches("build").is_some() {
        info!("Getting project settings");
        let project = settings::project::Project::new()?;
        commands::build(&project)?;
    } else if let Some(matches) = matches.subcommand_matches("preview") {
        info!("Getting project settings");
        let project = settings::project::Project::new()?;

        // the preview command can be called with or without a Global User having been config'd
        // so we convert this Result into an Option
        let user = settings::global_user::GlobalUser::new().ok();

        let method = HTTPMethod::from_str(matches.value_of("method").unwrap_or("get"))?;

        let body = match matches.value_of("body") {
            Some(s) => Some(s.to_string()),
            None => None,
        };

        commands::preview(project, user, method, body)?;
    } else if matches.subcommand_matches("whoami").is_some() {
        info!("Getting User settings");
        let user = settings::global_user::GlobalUser::new()?;

        commands::whoami(&user);
    } else if let Some(matches) = matches.subcommand_matches("publish") {
        info!("Getting project settings");
        let project = settings::project::Project::new()?;

        info!("Getting User settings");
        let user = settings::global_user::GlobalUser::new()?;

        info!("{}", matches.occurrences_of("release"));
        let release = match matches.occurrences_of("release") {
            1 => true,
            _ => false,
        };

        commands::publish(&user, &project, release)?;
    } else if let Some(matches) = matches.subcommand_matches("subdomain") {
        info!("Getting project settings");
        let project = settings::project::Project::new()?;

        info!("Getting User settings");
        let user = settings::global_user::GlobalUser::new()?;

        let name = matches
            .value_of("name")
            .expect("The subdomain name you are requesting must be provided.");

        commands::subdomain(name, &user, &project)?;
    } else if let Some(kv_matches) = matches.subcommand_matches("kv") {
        match kv_matches.subcommand() {
            ("create", Some(create_matches)) => {
                let title = create_matches.value_of("title").unwrap();
                commands::kv::create_namespace(title)?;
            }
            ("delete", Some(delete_matches)) => {
                let id = delete_matches.value_of("id").unwrap();
                commands::kv::delete_namespace(id)?;
            }
            ("rename", Some(rename_matches)) => {
                let id = rename_matches.value_of("id").unwrap();
                let title = rename_matches.value_of("title").unwrap();
                commands::kv::rename_namespace(id, title)?;
            }
            ("list", Some(_list_matches)) => {
                commands::kv::list_namespaces()?;
            }
            ("write", Some(write_matches)) => {
                match write_matches.subcommand() {
                    ("bulk", Some(bulk_write_matches)) => {
                        let id = bulk_write_matches.value_of("id").unwrap();
                        let filename = bulk_write_matches.value_of("filename").unwrap();
                        let expiration = bulk_write_matches.value_of("expiration");
                        let ttl = bulk_write_matches.value_of("time-to-live");
                        let base64 = match bulk_write_matches.occurrences_of("release") {
                            1 => true,
                            _ => false,
                        };
                        commands::kv::write_bulk(id, filename, expiration, ttl, base64)?;
                    }
                    ("", None) => {
                        println!("hi!")
                    },
                    _ => unreachable!(),
                }
            }
            ("", None) => message::warn("kv expects a subcommand"),
            _ => unreachable!(),
        }
    }
    Ok(())
}
