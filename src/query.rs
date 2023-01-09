use std::collections::{BTreeMap, HashMap};
use crate::cfg_file::{ConfigFile, QueryRef, ServerRef};
use crate::cli::Query;
use std::time::{SystemTime, UNIX_EPOCH};
use anyhow::bail;
use reqwest::Client;
use serde_json::{Result, Value};
use crate::prometheus::PromResponse;
use handlebars::Handlebars;

pub async fn execute_query(config: &ConfigFile, args: &Query) -> anyhow::Result<()> {
    let servername= match &args.server {
        Some(s) => s.to_string(),
        _ => "default".to_string()
    };
    let server: Option<ServerRef> = config.servers.clone().into_iter().filter(|s| s.name.eq(&servername)).next();
    if server.is_none(){
        anyhow::bail!("Can't find a default server to query, please specify default server in config.toml or specify server via -s");
    }

    let query: Option<QueryRef> = config.queries.clone().into_iter().filter(|q| q.name.eq(&args.query.clone().unwrap())).next();

    let query_string = match query {
        Some(e) => e.query,
        None => {
            bail!("There was no query by that name.")
        }
    };
    let query_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let full_url: String = format!("{}/api/v1/query",server.unwrap().url);

    let mut handlebars = Handlebars::new();

    let mut data = BTreeMap::new();
    for e in args.eval.clone() {
        let val: Vec<String> = e.split("=").map(str::to_string).collect();
        data.insert(val[0].clone(), val[1].clone());
    }
    let interp_string = handlebars.render_template(&query_string,&data)?;
    let rs = reqwest::Client::new()
        .post(&full_url)
        .form(&[
            ("query",interp_string),
            ("time", query_time.to_string())
        ])
        .send()
        .await?;
    let result = rs.text().await?;
    //let rsjson: Value = serde_json::from_str(&result)?;
    let rsjson: PromResponse = serde_json::from_str::<PromResponse>(&result)?;
    debug!("{:#?}", rsjson);
    for r in rsjson.data.unwrap().result.iter() {
        for v in r.value.iter() {
            info!("{:#?}", v);
         //info!("{}: {}", d[0], d[1]);
        }
    }
    Ok(())
}