use std::borrow::Cow;
use std::collections::{BTreeMap, VecDeque};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use crate::consts::*;
use tui_menu::{MenuItem, MenuState};
use crate::cfg_file::{ConfigFile, QueryRef, ServerRef};
use crate::cli::Query;
use anyhow::bail;
use handlebars::Handlebars;
use reqwest::Error as reqwestError;
use crate::prometheus::PromResponse;
use prometheus_http_query::{Error, Selector, RuleType};
use prometheus_http_query::Client as phqc;

pub struct AppData {
    pub config: ConfigFile,
    pub query: String,
    pub data: Vec<(f64, f64)>,
    pub tick_interval_msecs: u64,
    pub log_buffer: Arc<Mutex<VecDeque<String>>>,
    pub menu: MenuState<Cow<'static, str>>,
    pub last_request: Instant,
    pub last_refresh: Instant,
}

impl AppData {
    pub fn new(config_path: Option<PathBuf>) -> AppData {
        AppData {
            config: ConfigFile::new(config_path).unwrap(),
            query: "".to_string(),
            data: vec![],
            tick_interval_msecs: DEFAULT_TICK_INTERVAL_MSECS,
            log_buffer: Arc::new(Mutex::new(VecDeque::with_capacity(DEFAULT_LOG_RING_BUFFER_SIZE))),
            menu: make_menu(),
            last_request: Instant::now() - Duration::from_secs(20),
            last_refresh: Instant::now() - Duration::from_secs(20),
        }
    }

    pub async fn on_tick(&mut self) {
        let now = Instant::now();
        if self.last_refresh.elapsed() >= Duration::from_secs(MINIMUM_SERVER_WAIT_SECS) {
            debug!("in query loop");
            self.execute_promhttp_query_range("default".to_string(), "cpu".to_string()).await;
            self.last_refresh = now;
        }
    }

    pub async fn execute_promhttp_query_range(&mut self, servername: String, queryname: String) {
        let server: Option<&ServerRef> = self.config.servers.iter().find(|s| s.name == servername);
        if server.is_none() {
            warn!("Can't find a default server to query, please specify default server in config.toml or specify server via -s");
            return;
        }
        let query: Option<&QueryRef> = self.config.queries.iter().find(|q| q.name == queryname);

        let query_string = match query {
            Some(e) => &e.query,
            None => {
                warn!("There was no query by that name.");
                return;
            }
        };
        let query_start = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() - 3600;
        let query_end = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        let handlebars = Handlebars::new();

        let mut data = BTreeMap::new();
        //for e in self.args.eval.clone() {
        //    let val: Vec<String> = e.split("=").map(str::to_string).collect();
        // TODO: un-debug this
        data.insert("podex".to_string(), ".+".to_string());
        //}
        let interp_string = handlebars.render_template(&query_string, &data).unwrap();
        let client: phqc = {
            let c= reqwest::Client::builder()
                .build().unwrap();
            phqc::from(c, &server.unwrap().url).unwrap()
        };
        let response = match client.query_range(interp_string, query_start as i64, query_end as i64, 60 as f64).get().await {
            Ok(e) => e,
            Err(e) => {
                warn!("promhttplib is angy: {}", e);
                return;
            }
        };
        response.data().as_matrix().iter().for_each(|rvvec| {
            // rv is a vec of RangeVectors
            rvvec.iter().for_each(|rv| {
                rv.samples().iter().for_each(|sample| {
                    self.data.push((sample.timestamp(), sample.value()));
                })

            })
        });
    }

    pub fn execute_query_range(&mut self, servername: String, queryname: String) -> anyhow::Result<Vec<(f64, f64)>> {
        let server: Option<&ServerRef> = self.config.servers.iter().find(|s| s.name == servername);
        if server.is_none() {
            anyhow::bail!("Can't find a default server to query, please specify default server in config.toml or specify server via -s");
        }

        let query: Option<&QueryRef> = self.config.queries.iter().find(|q| q.name == queryname);

        let query_string = match query {
            Some(e) => &e.query,
            None => {
                bail!("There was no query by that name.")
            }
        };
        let query_start = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() - 3600;
        let query_end = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let full_url: String = format!("{}/api/v1/query_range", server.unwrap().url);

        let handlebars = Handlebars::new();

        let mut data = BTreeMap::new();
        //for e in self.args.eval.clone() {
        //    let val: Vec<String> = e.split("=").map(str::to_string).collect();
        // TODO: un-debug this
        data.insert("podex".to_string(), ".+".to_string());
        //}
        let interp_string = handlebars.render_template(&query_string, &data)?;
        let rs = reqwest::blocking::Client::new()
            .post(&full_url)
            .form(&[
                ("query", interp_string),
                ("step", "1m".to_string()),
                ("start", query_start.to_string()),
                ("end", query_end.to_string())
            ])
            .send()?;
        let result = match rs.text() {
            Ok(s) => {
                debug!("{:#?}", s);
                s
            }
            Err(e) => bail!("no text from response: {}", e)
        };
        let rsjson: PromResponse = match serde_json::from_str::<PromResponse>(&result) {
            Ok(s) => s,
            Err(e) => {
                warn!("unable to parse promresponse from json: {}", e);
                bail!("unable to parse promresponse from json: {}", e);
            }
        };
        let rsdata = match rsjson.data {
            Some(data) => data,
            None => {
                warn!("Didn't get any data back from query?");
                bail!("Didn't get any data back from query?");
            }
        };
        let retval: Vec<(f64, f64)> = Vec::new();
        debug!("rsdata contains {} results", rsdata.result.len());
        rsdata.result.iter().for_each(|datum| {
            debug!("datum contains {} points", datum.values.len());
            datum.values.iter().for_each(|value| {
                debug!("value[0] is {:#?}, value[1] is {:#?}", value[0], value[1]);
                //self.data.push((value[0].as_f64().unwrap(), value[1].as_f64().unwrap()));
            });
        });
        Ok(retval)
    }
}

fn make_menu() -> MenuState<Cow<'static, str>> {
    MenuState::new(vec![
        MenuItem::group(
            "File",
            vec![MenuItem::item("Exit", "exit".into())],
        ),
        MenuItem::group(
            "Help",
            vec![MenuItem::item("About", "about".into())]),
    ])
}