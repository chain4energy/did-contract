use bollard::container::{Config, CreateContainerOptions, RemoveContainerOptions, StopContainerOptions};
use bollard::network::{CreateNetworkOptions/*, RemoveNetworkOptions*/};
use bollard::secret::{PortBinding, PortMap};
use bollard::Docker;
use std::collections::HashMap;
use std::default::Default;
use std::env;
use std::process::Command;
use tokio::runtime::Runtime;


#[test]
fn connect_docker() {
    let docker = Docker::connect_with_local_defaults();
    assert!(docker.is_ok(), "Expected Ok, but got an Err");
    // run_chain(&docker.expect("failed to get docker"));

    let rt = Runtime::new().unwrap();

    // Run the async function and block until it completes
    let result = rt.block_on(run_chain(&docker.expect("failed to get docker")));
    assert!(result.is_ok(), "Expected Ok, but got an Err");
    // assert!(result.is_err(), "Expected Err, but got an Ok");
    // assert_eq!("Generic error: Querier contract error: Did document not found", result.err().unwrap().to_string());
}



// Function to replicate `run_chain` using the `bollard` library
async fn run_chain(docker: &Docker) -> Result<(), bollard::errors::Error> {
    // Create the network
    // let create_network_options = CreateNetworkOptions {
    //     name: "did",
    //     ..Default::default()
    // };
    // docker.create_network(create_network_options).await?;

    let uid = Command::new("id")
        .arg("-u")
        .output()
        .expect("Failed to get user ID")
        .stdout;
    let gid = Command::new("id")
        .arg("-g")
        .output()
        .expect("Failed to get group ID")
        .stdout;

    let uid_str = String::from_utf8(uid).expect("Failed to convert UID to string").trim().to_string();
    let gid_str = String::from_utf8(gid).expect("Failed to convert GID to string").trim().to_string();
    let user_str = format!("{}:{}", uid_str, gid_str); // "uid:gid"

    let mut labels = HashMap::new();
    labels.insert("com.docker.compose.project", "did-contract");

    let current_dir = env::current_dir().expect("Failed to get current directory");

    // Create and start containers
    for (name, volume, port_bindings) in [
        ("chain-node-did-1", ".e2e/node1", [("26657/tcp", "31657"), ("1317/tcp", "31317"), ("9090/tcp", "31090")]),
        ("chain-node-did-2", ".e2e/node2", [("26657/tcp", "32657"), ("1317/tcp", "32317"), ("9090/tcp", "32090")]),
        ("chain-node-did-3", ".e2e/node3", [("26657/tcp", "33657"), ("1317/tcp", "33317"), ("9090/tcp", "33090")]),
        ("chain-node-did-4", ".e2e/node4", [("26657/tcp", "34657"), ("1317/tcp", "34317"), ("9090/tcp", "34090")]),
    ] {

        let volume_absolute_path = current_dir.join(volume).to_string_lossy().into_owned();
        let bind = format!("{}:/chain4energy/.c4e-chain/", volume_absolute_path);
        let create_options = CreateContainerOptions { name, platform: Some("linux/amd64") };
        let config = create_container_config(&bind, &user_str, &port_bindings);
        docker.create_container(Some(create_options), config).await?;
        docker.start_container(name, None::<bollard::container::StartContainerOptions<String>>).await?;
        println!("Running container: {}", name);
    }

    Ok(())
}

fn create_container_config(volume: &str, user_str: &str, port_bindings: &[(&str, &str)]) -> Config<String> {
    let mut labels = HashMap::new();
    labels.insert("com.docker.compose.project".to_string(), "did-contract".to_string());

    let mut port_map: PortMap = PortMap::new();
    for (container_port, host_port) in port_bindings {
        let host_binding = vec![PortBinding {
            host_ip: Some("0.0.0.0".to_string()),
            host_port: Some(host_port.to_string()),
        }];
        port_map.insert(container_port.to_string(), Some(host_binding));
    }


    Config {
        image: Some("c4e-chain-did:v1.4.3".to_string()),
        labels: Some(labels),
        host_config: Some(bollard::service::HostConfig {
            binds: Some(vec![volume.to_string()]),
            network_mode: Some("did".to_string()),
            port_bindings: Some(port_map),
            
            ..Default::default()
        }),
        exposed_ports: Some(
            port_bindings
                .iter()
                .map(|(container_port, _)| (container_port.to_string(), HashMap::new()))
                .collect(),
        ),
        user: Some(user_str.to_string()),
        ..Default::default()
    }
}

// // Function to replicate `stop_chain` using the `bollard` library
// async fn stop_chain(docker: &Docker) -> Result<(), bollard::errors::Error> {
//     let containers = docker
//         .list_containers::<String>(None)
//         .await?
//         .into_iter()
//         .filter(|container| {
//             if let Some(labels) = &container.labels {
//                 return labels.get("com.docker.compose.project") == Some(&"did-contract".to_string());
//             }
//             false
//         })
//         .collect::<Vec<_>>();

//     // Stop and remove containers with the label
//     for container in containers {
//         if let Some(container_id) = &container.id {
//             println!("Stopping container: {}", container_id);
//             docker
//                 .stop_container(container_id, Some(StopContainerOptions { t: 10 }))
//                 .await?;
//             docker
//                 .remove_container(container_id, Some(RemoveContainerOptions { force: true, ..Default::default() }))
//                 .await?;
//         }
//     }

//     // Remove the network
//     println!("Removing network 'did'...");
//     docker.remove_network("did", Some(RemoveNetworkOptions::default())).await?;

//     Ok(())
// }


