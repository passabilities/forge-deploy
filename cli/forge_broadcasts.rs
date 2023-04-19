
use std::{fs, collections::HashMap, path::Path};

use crate::types::{DeploymentObject};

use serde::{Deserialize, Serialize};
use serde_json::{Value, from_str};
use serde_json::Value::Object;
use regex::Regex;


#[derive(Debug, Deserialize, Serialize, Clone, Default)]
#[serde(rename_all(deserialize = "camelCase", serialize = "snake_case"))]
pub struct Transaction {
    hash: String,
    transaction_type: String,
    contract_name: Option<String>,
    contract_address: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
#[serde(rename_all(deserialize = "camelCase", serialize = "snake_case"))]
pub struct FileContent {
    transactions: Vec<Transaction>,
    returns: Value
}

pub fn get_last_deployments(
    root_folder: &str,
    broadcast_folder: &str) -> HashMap<String, DeploymentObject> {
    let re = Regex::new(r"\((.+?)\)").unwrap();

    let folder_path_buf = Path::new(root_folder).join(broadcast_folder);

    let mut new_deployments: HashMap<String, DeploymentObject> = HashMap::new();

    for script_dir in fs::read_dir(folder_path_buf).unwrap() {
        match script_dir {
            Ok(script_dir) => 
                if script_dir.metadata().unwrap().is_dir() {
                    // println!("script {}", script_dir.path().display());
                    for chain_dir in fs::read_dir(script_dir.path()).unwrap() {
                        match chain_dir {
                            Ok(chain_dir) => 
                                if chain_dir.metadata().unwrap().is_dir() {
                                    // println!("chain: {}", chain_dir.path().display());
                                    let filepath_buf = chain_dir.path().join("run-latest.json");
                                    // let filepath = filepath_buf.to_str().unwrap();
                                    
                                    let data = fs::read_to_string(filepath_buf).expect("Unable to read file");
                                    let res: FileContent = from_str(&data).expect("Unable to parse");
                                    let returns = res.returns;
                                    if let Object(returns) = returns {
                                        let deployments = returns.get("newDeployments");
                                        if let Some(deployments) = deployments {
                                            if deployments["internal_type"] == "struct DeployerDeployment[]" {
                                                let value: String = deployments["value"].to_string();
                                                // println!("{}", value);
                                                let regex_result = re.captures_iter(value.as_str());
                                                
                                                for cap in regex_result {
                                                    
                                                    let parts = cap[1].split(", ");
                                                    let collection = parts.collect::<Vec<&str>>();
                                                    let name = collection[0];
                                                    let address = collection[1];
                                                    let bytecode = collection[2];
                                                    let args_data = collection[3];
                                                    let artifact_full_path = collection[4];
                                                    let deployment_context = collection[5];
                                                    let chain_id = collection[6];

                                                    let mut artifact_splitted = artifact_full_path.split(":");
                                                    let artifact_path = artifact_splitted.next().unwrap();
                                                    let contract_name = artifact_splitted.next();


                                                    // println!("{}:{}", artifact_path, contract_name.unwrap_or("unknown"));
                                                    // println!("{} address: {}, artifact_path: {}, contract_name: {}, deployment_context: {}", name, address, artifact_path, contract_name, deployment_context);
                                                    new_deployments.insert(format!("{}::{}", deployment_context, name.to_string()), DeploymentObject {
                                                        name: name.to_string(),
                                                        address: address.to_string(),
                                                        bytecode: bytecode.to_string(),
                                                        args_data: args_data.to_string(),
                                                        // TODO ?
                                                        // args: transaction.args,
                                                        // data: transaction.data,
                                                        contract_name: contract_name.map(|s| s.to_string()),
                                                        artifact_path: artifact_path.to_string(),
                                                        deployment_context: deployment_context.to_string(),
                                                        chain_id: chain_id.to_string()
                                                    });
                                                }
                                            } else {
                                                println!("not matching returns type")    
                                            }
                                            
                                        } else {
                                            println!("no deployments")
                                        }
                                    }
                                    
                                }
                            Err(_) => ()
                        }
                    }
                }
            Err(_) => ()
        }
    }
    
    return new_deployments;
}
      