use chrono::prelude::*;
use reqwest;
use serde::{Deserialize, Deserializer};
use serde_json;
use std::collections::{HashMap, HashSet};
use std::env;

use crate::provider::{GraphQLRequest, IssueProvider, Labels};
use crate::types::{Component, ComponentStatus, Error, Incident, IncidentUpdate};

const LABEL_COMPONENT_PREFIX: &'static str = "component/";
const LABEL_STATUS_PREFIX: &'static str = "status::";
const LABEL_STATUS_OPERATIONAL: &'static str = "status::operational";
const LABEL_STATUS_PARTIAL_OUTAGE: &'static str = "status::partial-outage";
const LABEL_STATUS_MAJOR_OUTAGE: &'static str = "status::major-outage";

#[derive(Debug, Deserialize)]
struct GQLLabel {
    name: String,
}

#[derive(Debug, Deserialize)]
struct GQLLabelNode {
    nodes: Vec<GQLLabel>,
}
#[derive(Debug, Deserialize)]
struct GQLComment {
    body: String,
    #[serde(rename = "createdAt", deserialize_with = "parse_datetime")]
    created_at: DateTime<Utc>,
}
#[derive(Debug, Deserialize)]
struct GQLCommentNode {
    nodes: Vec<GQLComment>,
}

#[derive(Debug, Deserialize)]
struct GQLIncident {
    body: String,
    #[serde(rename = "closedAt", deserialize_with = "parse_datetime_optional")]
    closed_at: Option<DateTime<Utc>>,
    comments: GQLCommentNode,
    #[serde(rename = "createdAt", deserialize_with = "parse_datetime")]
    created_at: DateTime<Utc>,
    id: String,
    labels: GQLLabelNode,
    title: String,
}

#[derive(Debug, Deserialize)]
struct GQLIncidentNode {
    nodes: Vec<GQLIncident>,
}

#[derive(Debug, Deserialize)]
struct GQLRepository {
    #[serde(rename = "openIncidents")]
    open_incidents: GQLIncidentNode,
    #[serde(rename = "closedIncidents")]
    closed_incidents: GQLIncidentNode,
}

#[derive(Debug, Deserialize)]
struct GQLData {
    repository: GQLRepository,
}

#[derive(Debug, Deserialize)]
struct GQLRoot {
    data: GQLData,
}

fn parse_datetime_optional<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    match Option::deserialize(deserializer).unwrap() {
        None => Ok(None),
        Some(s) => Ok(Some(DateTime::from(
            DateTime::parse_from_rfc3339(s).unwrap(),
        ))),
    }
}

fn parse_datetime<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = Deserialize::deserialize(deserializer).unwrap();
    Ok(DateTime::from(DateTime::parse_from_rfc3339(s).unwrap()))
}

pub struct GitHubIssueProvider {
    owner: String,
    repository: String,
    open_incidents: Vec<GQLIncident>,
    closed_incidents: Vec<GQLIncident>,
}

impl GitHubIssueProvider {
    pub fn new(owner: String, repository: String) -> Self {
        GitHubIssueProvider {
            owner,
            repository,
            open_incidents: Vec::new(),
            closed_incidents: Vec::new(),
        }
    }

    fn get_status(&self, labels: &Vec<GQLLabel>) -> ComponentStatus {
        let mut highest_status = ComponentStatus::Unknown;
        for label in labels.iter() {
            if !label.name.starts_with(LABEL_STATUS_PREFIX) {
                continue;
            }
            let this_status = match label.name.as_str() {
                LABEL_STATUS_OPERATIONAL => ComponentStatus::Operational,
                LABEL_STATUS_PARTIAL_OUTAGE => ComponentStatus::PartialOutage,
                LABEL_STATUS_MAJOR_OUTAGE => ComponentStatus::MajorOutage,
                _ => ComponentStatus::Unknown,
            };
            if this_status > highest_status {
                highest_status = this_status;
            }
        }
        highest_status
    }
}

impl IssueProvider for GitHubIssueProvider {
    fn get_open_incidents(&self) -> Vec<Incident> {
        let mut incidents = Vec::with_capacity(self.open_incidents.len());
        for open_incident in self.open_incidents.iter() {
            let mut updates = Vec::with_capacity(open_incident.comments.nodes.len() + 1);
            updates.push(IncidentUpdate::new(
                open_incident.created_at,
                open_incident.body.clone(),
            ));
            for update in open_incident.comments.nodes.iter() {
                updates.push(IncidentUpdate::new(update.created_at, update.body.clone()));
            }
            updates.sort();
            updates.reverse();
            let mut component_names = Vec::new();
            for component_name in open_incident.labels.nodes.iter() {
                if !component_name.name.starts_with(LABEL_COMPONENT_PREFIX) {
                    continue;
                }
                component_names.push(String::from(
                    component_name
                        .name
                        .strip_prefix(LABEL_COMPONENT_PREFIX)
                        .unwrap(),
                ));
            }
            let incident = Incident::new_open(
                component_names,
                open_incident.id.clone(),
                open_incident.created_at,
                self.get_status(&open_incident.labels.nodes),
                open_incident.title.clone(),
                updates,
            );
            incidents.push(incident);
        }
        incidents.sort();
        incidents.reverse();
        incidents
    }
    fn get_closed_incidents(&self) -> Vec<Incident> {
        let mut incidents = Vec::with_capacity(self.closed_incidents.len());
        for closed_incident in self.closed_incidents.iter() {
            let mut updates = Vec::with_capacity(closed_incident.comments.nodes.len() + 1);
            updates.push(IncidentUpdate::new(
                closed_incident.created_at,
                closed_incident.body.clone(),
            ));
            for update in closed_incident.comments.nodes.iter() {
                updates.push(IncidentUpdate::new(update.created_at, update.body.clone()));
            }
            updates.sort();
            updates.reverse();
            let incident = Incident::new_closed(
                closed_incident.closed_at,
                closed_incident.id.clone(),
                closed_incident.created_at,
                self.get_status(&closed_incident.labels.nodes),
                closed_incident.title.clone(),
                updates,
            );
            incidents.push(incident);
        }
        incidents.sort();
        incidents
    }

    fn fetch_incidents(&mut self) -> Result<(), Error> {
        let github_token = env::var("GITHUB_TOKEN").unwrap();
        let client = reqwest::blocking::Client::builder()
            .user_agent("StatusPageRS/0.1.0")
            .build()
            .unwrap();
        let mut variables = HashMap::with_capacity(2);
        variables.insert("repository", self.repository.clone());
        variables.insert("owner", self.owner.clone());
        let params = GraphQLRequest::new(
            r#"
              query($repository: String!, $owner: String!){
                repository(name: $repository, owner: $owner) {
                  openIncidents: issues(first: 100, states: OPEN) {
                    nodes {
                      body
                      closedAt
                      comments(first: 100) {
                        nodes {
                          body
                          createdAt
                        }
                      }
                      createdAt
                      id
                      labels(first: 10) {
                        nodes {
                          name
                        }
                      }
                      title
                    }
                  }
                  closedIncidents: issues(first: 100, states: CLOSED) {
                    nodes {
                      body
                      closedAt
                      comments(first: 100) {
                        nodes {
                          body
                          createdAt
                        }
                      }
                      createdAt
                      id
                      labels(first: 10) {
                        nodes {
                          name
                        }
                      }
                      title
                    }
                  }
                }
              }
            "#,
            variables,
        );
        let resp = client
            .post("https://api.github.com/graphql")
            .bearer_auth(&github_token)
            .json(&params)
            .send()
            .unwrap();
        let mut root: GQLRoot = resp.json().unwrap();
        self.open_incidents
            .append(&mut root.data.repository.open_incidents.nodes);
        self.closed_incidents
            .append(&mut root.data.repository.closed_incidents.nodes);
        Ok(())
    }

    fn fetch_labels(&self) -> Labels {
        let github_token = env::var("GITHUB_TOKEN").unwrap();
        let client = reqwest::blocking::Client::builder()
            .user_agent("StatusPageRS/0.1.0")
            .build()
            .unwrap();
        let mut variables = HashMap::with_capacity(2);
        variables.insert("repository", self.repository.clone());
        variables.insert("owner", self.owner.clone());
        let params = GraphQLRequest::new(
            r#"
              query($repository: String!, $owner: String!){
                repository(name: $repository, owner: $owner) {
                  labels(first: 100) {
                    nodes {
                      name
                    }
                  }
                }
              }
            "#,
            variables,
        );
        let resp = client
            .post("https://api.github.com/graphql")
            .bearer_auth(&github_token)
            .json(&params)
            .send()
            .unwrap();
        let root: serde_json::Value = resp.json().unwrap();
        let mut component_labels = HashSet::new();
        let mut status_labels = HashSet::new();
        if let Some(data) = root.get("data") {
            if let Some(repository) = data.get("repository") {
                if let Some(labels) = repository.get("labels") {
                    if let Some(nodes) = labels.get("nodes") {
                        if let Some(node) = nodes.as_array() {
                            for label in node.iter() {
                                let label =
                                    String::from(label.get("name").unwrap().as_str().unwrap());
                                if label.starts_with(LABEL_STATUS_PREFIX) {
                                    status_labels.insert(label);
                                } else if label.starts_with(LABEL_COMPONENT_PREFIX) {
                                    component_labels.insert(String::from(label));
                                }
                            }
                        }
                    }
                }
            }
        }
        Labels::new(component_labels, status_labels)
    }

    fn get_expected_labels(&self, components: &HashMap<String, Component>) -> Labels {
        let mut component_labels = HashSet::new();
        for comp in components.keys() {
            component_labels.insert(format!("{}{}", LABEL_COMPONENT_PREFIX, comp));
        }
        let mut status_labels = HashSet::new();
        status_labels.insert(String::from(LABEL_STATUS_OPERATIONAL));
        status_labels.insert(String::from(LABEL_STATUS_PARTIAL_OUTAGE));
        status_labels.insert(String::from(LABEL_STATUS_MAJOR_OUTAGE));
        Labels::new(component_labels, status_labels)
    }
}
