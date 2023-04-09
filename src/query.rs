use std::collections::{BTreeMap, HashMap};
use crate::cfg_file::{ConfigFile, QueryRef, ServerRef};
use crate::cli::Query;
use std::time::{SystemTime, UNIX_EPOCH};
use anyhow::bail;
use reqwest::blocking::Client;
use crate::prometheus::{PromResponse};
use handlebars::Handlebars;

pub fn execute_query(config: &ConfigFile, args: &Query) -> anyhow::Result<()> {
    let servername = match &args.server {
        Some(s) => s.to_string(),
        _ => "default".to_string()
    };
    let server: Option<&ServerRef> = config.servers.iter().find(|s| s.name == servername);
    if server.is_none() {
        anyhow::bail!("Can't find a default server to query, please specify default server in config.toml or specify server via -s");
    }

    let query: Option<&QueryRef> = config.queries.iter().find(|q| Some(&q.name) == args.query.as_ref());

    let query_string = match query {
        Some(e) => &e.query,
        None => {
            bail!("There was no query by that name.")
        }
    };
    let query_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let full_url: String = format!("{}/api/v1/query", server.unwrap().url);

    let handlebars = Handlebars::new();

    let mut data = BTreeMap::new();
    for e in args.eval.clone() {
        let val: Vec<String> = e.split("=").map(str::to_string).collect();
        data.insert(val[0].clone(), val[1].clone());
    }
    let interp_string = handlebars.render_template(&query_string, &data)?;
    let rs = reqwest::blocking::Client::new()
        .post(&full_url)
        .form(&[
            ("query", interp_string),
            ("time", query_time.to_string())
        ])
        .send()?;
    let result = match rs.text() {
        Ok(s) => s,
        Err(e) => bail!("no text from response: {}", e)
    };
    let rsjson: PromResponse = serde_json::from_str::<PromResponse>(&result)?;
    rsjson.data.unwrap().result.iter().for_each(|datum| {
        //info!("{}: {}", datum.value[0], datum.value[1]);
    });
    Ok(())
}

pub fn execute_query_range(config: &ConfigFile, args: &Query) -> anyhow::Result<()> {
    let servername = match &args.server {
        Some(s) => s.to_string(),
        _ => "default".to_string()
    };
    let server: Option<&ServerRef> = config.servers.iter().find(|s| s.name == servername);
    if server.is_none() {
        anyhow::bail!("Can't find a default server to query, please specify default server in config.toml or specify server via -s");
    }

    let query: Option<&QueryRef> = config.queries.iter().find(|q| Some(&q.name) == args.query.as_ref());

    let query_string = match query {
        Some(e) => &e.query,
        None => {
            bail!("There was no query by that name.")
        }
    };
    let query_start = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()-3600;
    let query_end = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let full_url: String = format!("{}/api/v1/query_range", server.unwrap().url);

    let handlebars = Handlebars::new();

    let mut data = BTreeMap::new();
    for e in args.eval.clone() {
        let val: Vec<String> = e.split("=").map(str::to_string).collect();
        data.insert(val[0].clone(), val[1].clone());
    }
    let interp_string = handlebars.render_template(&query_string, &data)?;
    let rs = reqwest::blocking::Client::new()
        .post(&full_url)
        .form(&[
            ("query", interp_string),
            ("start", query_start.to_string()),
            ("end", query_end.to_string())
        ])
        .send()?;
    let result = match rs.text() {
        Ok(s) => s,
        Err(e) => bail!("no text from response: {}", e)
    };
    let rsjson: PromResponse = serde_json::from_str::<PromResponse>(&result)?;
    rsjson.data.unwrap().result.iter().for_each(|datum| {
       // info!("{}: {}", datum.value[0], datum.value[1]);
    });
    Ok(())
}