use chrono::prelude::*;
use chrono::serde::ts_seconds::serialize as to_seconds;
use chrono::serde::ts_seconds_option::serialize as to_seconds_optional;
use serde::Serialize;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug, Serialize, PartialEq, PartialOrd)]
pub enum ComponentStatus {
    Unknown,
    Operational,
    PartialOutage,
    MajorOutage,
}

#[derive(Debug, Serialize)]
pub struct Component {
    name: String,
    status: ComponentStatus,
}
impl Component {
    pub fn new(name: String) -> Self {
        Component {
            name,
            status: ComponentStatus::Operational,
        }
    }
    pub fn bump_status(&mut self, status: ComponentStatus) {
        if status > self.status {
            self.status = status
        }
    }
}

#[derive(Debug, Serialize)]
pub enum IncidentStatus {
    Open,
    Closed,
}

#[derive(Debug, Serialize)]
pub struct IncidentUpdate {
    #[serde(serialize_with = "to_seconds")]
    datetime: DateTime<Utc>,
    text: String,
}

impl IncidentUpdate {
    pub fn new(datetime: DateTime<Utc>, text: String) -> Self {
        IncidentUpdate { datetime, text }
    }
}

impl Ord for IncidentUpdate {
    fn cmp(&self, other: &Self) -> Ordering {
        self.datetime.cmp(&other.datetime)
    }
}
impl PartialOrd for IncidentUpdate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl PartialEq for IncidentUpdate {
    fn eq(&self, other: &Self) -> bool {
        self.datetime == other.datetime
    }
}
impl Eq for IncidentUpdate {}

#[derive(Debug, Serialize)]
pub struct Incident {
    #[serde(serialize_with = "to_seconds_optional")]
    closed: Option<DateTime<Utc>>,
    component_names: Option<Vec<String>>,
    #[serde(serialize_with = "to_seconds")]
    opened: DateTime<Utc>,
    severity: ComponentStatus,
    status: IncidentStatus,
    title: String,
    updates: Vec<IncidentUpdate>,
}

impl Incident {
    pub fn new_open(
        component_names: Vec<String>,
        opened: DateTime<Utc>,
        severity: ComponentStatus,
        title: String,
        updates: Vec<IncidentUpdate>,
    ) -> Self {
        Incident {
            closed: None,
            component_names: Some(component_names),
            opened,
            severity,
            status: IncidentStatus::Open,
            title,
            updates,
        }
    }
    pub fn new_closed(
        closed: Option<DateTime<Utc>>,
        opened: DateTime<Utc>,
        severity: ComponentStatus,
        title: String,
        updates: Vec<IncidentUpdate>,
    ) -> Self {
        Incident {
            closed,
            component_names: None,
            opened,
            severity,
            status: IncidentStatus::Closed,
            title,
            updates,
        }
    }

    pub fn update_components(&self, components: &mut HashMap<String, Component>) {
        match &self.component_names {
            Some(names) => {
                for name in names.iter() {
                    match components.get_mut(name) {
                        Some(component) => component.bump_status(self.severity),
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}

impl Ord for Incident {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.severity == other.severity {
            self.opened.cmp(&other.opened).reverse()
        } else if self.severity < other.severity {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    }
}
impl PartialOrd for Incident {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl PartialEq for Incident {
    fn eq(&self, other: &Self) -> bool {
        self.severity == other.severity && self.opened == other.opened
    }
}
impl Eq for Incident {}

#[derive(Debug)]
pub struct Labels {
    pub components: HashSet<String>,
    pub statuses: HashSet<String>,
}
impl Labels {
    pub fn new(components: HashSet<String>, statuses: HashSet<String>) -> Self {
        Labels {
            components,
            statuses,
        }
    }
}

#[derive(Debug)]
pub enum Error {}

pub trait IssueProvider {
    fn get_open_incidents(&self) -> Vec<Incident>;
    fn get_closed_incidents(&self) -> Vec<Incident>;
    fn fetch_incidents(&mut self) -> Result<(), Error>;
    fn fetch_labels(&self) -> Labels;
    fn get_expected_labels(&self, components: &HashMap<String, Component>) -> Labels;
}

#[derive(Debug, Serialize)]
pub struct GraphQLRequest {
    query: &'static str,
    variables: HashMap<&'static str, String>,
}

impl GraphQLRequest {
    pub fn new(query: &'static str, variables: HashMap<&'static str, String>) -> Self {
        GraphQLRequest { query, variables }
    }
}
