use serde::Serialize;
use std::collections::{HashMap, HashSet};

use crate::types::{Component, Error, Incident};

pub trait IssueProvider {
    fn get_open_incidents(&self) -> Vec<Incident>;
    fn get_closed_incidents(&self) -> Vec<Incident>;
    fn fetch_incidents(&mut self) -> Result<(), Error>;
    fn fetch_labels(&self) -> Labels;
    fn get_expected_labels(&self, components: &HashMap<String, Component>) -> Labels;
}

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
