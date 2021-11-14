use slug::slugify;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tera::{Context, Tera};
use toml;

mod config;
mod github;
mod provider;
mod types;
mod utils;
mod writer;

use config::{Backend, Config};
use github::GitHubIssueProvider;
use provider::IssueProvider;
use types::{Component, Incident};
use writer::render;

fn main() {
    let config_string =
        fs::read_to_string("config.toml").expect("Something went wrong reading the config file.");

    let config: Config = toml::from_str(&config_string).unwrap();

    let mut issue_provider = match config.backend {
        Backend::GITHUB => match config.github {
            Some(gh) => GitHubIssueProvider::new(gh.owner, gh.repository),
            None => return,
        },
    };

    let mut components = HashMap::with_capacity(config.components.len());
    for comp in config.components.iter() {
        components.insert(slugify(comp), Component::new(comp.to_string()));
    }

    let existing_labels = issue_provider.fetch_labels();
    let expected_labels = issue_provider.get_expected_labels(&components);
    let missing_component_labels = expected_labels
        .components
        .difference(&existing_labels.components)
        .collect::<Vec<&String>>();
    let missing_status_labels = expected_labels
        .statuses
        .difference(&existing_labels.statuses)
        .collect::<Vec<&String>>();
    let obsolet_component_labels = existing_labels
        .components
        .difference(&expected_labels.components)
        .collect::<Vec<&String>>();
    if missing_component_labels.len() > 0 {
        eprintln!("Missing component labels:");
        for label in missing_component_labels {
            eprintln!("- {}", label);
        }
    }
    if obsolet_component_labels.len() > 0 {
        eprintln!("Obsolet component labels:");
        for label in obsolet_component_labels {
            eprintln!("- {}", label);
        }
    }
    if missing_status_labels.len() > 0 {
        eprintln!("Missing status labels:");
        for label in missing_status_labels {
            eprintln!("- {}", label);
        }
    }

    let tera = match Tera::new(config.template_dir.to_str().unwrap()) {
        Ok(t) => t,
        Err(e) => {
            println!("Parsing error(s): {}", e);
            ::std::process::exit(1);
        }
    };

    issue_provider.fetch_incidents().unwrap();

    let open_incidents = issue_provider.get_open_incidents();
    for incident in open_incidents.iter() {
        incident.update_components(&mut components);
    }
    let closed_incidents = issue_provider.get_closed_incidents();

    render_index(
        &tera,
        &components,
        &open_incidents,
        &closed_incidents,
        config.output_dir.join("index.html"),
    );
    for incident in open_incidents.iter() {
        render_incident(
            &tera,
            &incident,
            config.output_dir.join(format!("{}.html", incident.id)),
        );
    }
    for incident in closed_incidents.iter() {
        render_incident(
            &tera,
            &incident,
            config.output_dir.join(format!("{}.html", incident.id)),
        );
    }
}

fn render_index(
    tera: &Tera,
    components: &HashMap<String, Component>,
    open: &Vec<Incident>,
    closed: &Vec<Incident>,
    output: PathBuf,
) {
    let mut ctx = Context::new();
    ctx.insert("components", &components);
    ctx.insert("open_incidents", &open);
    ctx.insert("closed_incidents", &closed);
    render(&tera, &ctx, "index.html", output);
}

fn render_incident(tera: &Tera, incident: &Incident, output: PathBuf) {
    let mut ctx = Context::new();
    ctx.insert("incident", &incident);
    render(&tera, &ctx, "detail.html", output);
}
