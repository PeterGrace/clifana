use serde::Deserialize;

//{"status":"success","data":{"resultType":"vector","result":[]}}

#[derive(Deserialize, Debug)]
pub struct Points {
    pub metric: serde_json::Value,
    pub value: Vec<serde_json::Value>
}

#[derive(Deserialize, Debug)]
pub struct Datum {
    #[serde(rename="resultType")]
    pub result_type: String,
    pub result: Vec<Points>
}
#[derive(Deserialize, Debug)]
pub struct PromResponse {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Datum>
}