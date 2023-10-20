use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

use ics100::msg::DetailsResponse;
use ics100::msg::ExecuteMsg;
use ics100::msg::InstantiateMsg;
use ics100::msg::ListResponse;
use ics100::msg::QueryMsg;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(ListResponse), &out_dir);
    export_schema(&schema_for!(DetailsResponse), &out_dir);
}
