const DOCKER_SOCKET: &str = "/var/run/docker.sock";
const TIMEOUT: u64 = 5000;

use axum::{
    Router,
    extract::{Json, Path},
    http::StatusCode,
    routing::{get, put},
};
use bollard::{
    Docker,
    query_parameters::{
        InspectContainerOptions, ListContainersOptions, StartContainerOptions, StopContainerOptions,
    },
    secret::{ContainerInspectResponse, ContainerSummary},
};
use serde::Deserialize;
use tracing::instrument;

// Router function to define all container routes
pub fn router() -> Router {
    Router::new()
        .route("/", get(list_containers))
        .route("/{id}", get(get_container))
        .route("/{id}", put(update_container))
}

#[derive(Debug, Deserialize)]
pub struct UpdateContainerRequest {
    pub running: bool,
}

// Containers controller handlers

// List all containers
#[instrument]
pub async fn list_containers() -> Result<Json<Vec<ContainerSummary>>, StatusCode> {
    let docker = Docker::connect_with_unix(DOCKER_SOCKET, TIMEOUT, bollard::API_DEFAULT_VERSION)
        .map_err(|e| {
            eprintln!("Failed to connect to Docker socket: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let list_containers_options = Some(ListContainersOptions {
        all: true,
        ..Default::default()
    });

    let containers = docker
        .list_containers(list_containers_options)
        .await
        .map_err(|e| {
            eprintln!("Failed to list containers: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(containers))
}

// Get a specific container by ID
#[instrument]
pub async fn get_container(
    Path(id): Path<String>,
) -> Result<Json<ContainerInspectResponse>, StatusCode> {
    let docker = Docker::connect_with_unix(DOCKER_SOCKET, TIMEOUT, bollard::API_DEFAULT_VERSION)
        .map_err(|e| {
            eprintln!("Failed to connect to Docker socket: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let container = docker
        .inspect_container(id.as_str(), None::<InspectContainerOptions>)
        .await
        .map_err(|e| {
            eprintln!("Failed to inspect container {}: {}", id, e);
            StatusCode::NOT_FOUND
        })?;

    Ok(Json(container))
}

// Update a container
#[instrument]
pub async fn update_container(
    Path(id): Path<String>,
    Json(payload): Json<UpdateContainerRequest>,
) -> Result<Json<ContainerInspectResponse>, StatusCode> {
    let docker = Docker::connect_with_unix(DOCKER_SOCKET, TIMEOUT, bollard::API_DEFAULT_VERSION)
        .map_err(|e| {
            eprintln!("Failed to connect to Docker socket: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if payload.running {
        docker
            .start_container(&id, None::<StartContainerOptions>)
            .await
            .map_err(|e| {
                eprintln!("Failed to start container {}: {}", id, e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
    } else {
        docker
            .stop_container(&id, None::<StopContainerOptions>)
            .await
            .map_err(|e| {
                eprintln!("Failed to stop container {}: {}", id, e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
    }

    let updated_container = docker
        .inspect_container(&id, None::<InspectContainerOptions>)
        .await
        .map_err(|e| {
            eprintln!("Failed to inspect updated container {}: {}", id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(updated_container))
}
