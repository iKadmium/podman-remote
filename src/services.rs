use axum::{
    Router,
    extract::{Json, Path},
    http::StatusCode,
    routing::{get, put},
};
use serde::{Deserialize, Serialize};
use systemd_zbus::{ManagerProxy, Mode};
use tracing::instrument;
use zbus::Connection;

pub fn router() -> Router {
    Router::new()
        .route("/", get(list_services))
        .route("/{name}", get(get_service))
        .route("/{name}", put(update_service))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ServiceCommand {
    Start,
    Stop,
    Restart,
    Enable,
    Disable,
}

#[derive(Debug, Deserialize)]
pub struct UpdateServiceRequest {
    pub command: ServiceCommand,
}

#[derive(Debug, Serialize)]
pub struct ServiceInfo {
    pub name: String,
    pub active_state: String,
    pub sub_state: String,
    pub load_state: String,
}

async fn connect_to_user_bus() -> Result<Connection, StatusCode> {
    // Verify DBUS_SESSION_BUS_ADDRESS is set
    if std::env::var("DBUS_SESSION_BUS_ADDRESS").is_err() {
        tracing::error!(
            "DBUS_SESSION_BUS_ADDRESS environment variable is not set. Please set it (e.g., unix:path=/run/user/1000/bus)"
        );
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Connection::session().await.map_err(|e| {
        tracing::error!("Failed to connect to user DBus: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

fn normalize_service_name(name: &str) -> String {
    if name.ends_with(".service") {
        name.to_string()
    } else {
        format!("{}.service", name)
    }
}

#[instrument]
pub async fn list_services() -> Result<Json<Vec<ServiceInfo>>, StatusCode> {
    let connection = connect_to_user_bus().await?;

    let manager = ManagerProxy::new(&connection).await.map_err(|e| {
        tracing::error!("Failed to create ManagerProxy: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let units = manager.list_units().await.map_err(|e| {
        tracing::error!("Failed to list units: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let services: Vec<ServiceInfo> = units
        .into_iter()
        .filter(|unit| unit.name.ends_with(".service"))
        .map(|unit| ServiceInfo {
            name: unit.name,
            active_state: format!("{:?}", unit.active),
            sub_state: format!("{:?}", unit.sub_state),
            load_state: format!("{:?}", unit.load),
        })
        .collect();

    Ok(Json(services))
}

#[instrument]
pub async fn get_service(Path(name): Path<String>) -> Result<Json<ServiceInfo>, StatusCode> {
    let service_name = normalize_service_name(&name);
    let connection = connect_to_user_bus().await?;

    let manager = ManagerProxy::new(&connection).await.map_err(|e| {
        tracing::error!("Failed to create ManagerProxy: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let units = manager.list_units().await.map_err(|e| {
        tracing::error!("Failed to list units: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let service = units
        .into_iter()
        .find(|unit| unit.name == service_name)
        .map(|unit| ServiceInfo {
            name: unit.name,
            active_state: format!("{:?}", unit.active),
            sub_state: format!("{:?}", unit.sub_state),
            load_state: format!("{:?}", unit.load),
        })
        .ok_or_else(|| {
            tracing::warn!("Service {} not found", service_name);
            StatusCode::NOT_FOUND
        })?;

    Ok(Json(service))
}

#[instrument]
pub async fn update_service(
    Path(name): Path<String>,
    Json(payload): Json<UpdateServiceRequest>,
) -> Result<Json<ServiceInfo>, StatusCode> {
    let service_name = normalize_service_name(&name);
    let connection = connect_to_user_bus().await?;

    let manager = ManagerProxy::new(&connection).await.map_err(|e| {
        tracing::error!("Failed to create ManagerProxy: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    match payload.command {
        ServiceCommand::Start => {
            manager
                .start_unit(&service_name, Mode::Replace)
                .await
                .map_err(|e| {
                    tracing::error!("Failed to start service {}: {}", service_name, e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;
        }
        ServiceCommand::Stop => {
            manager
                .stop_unit(&service_name, Mode::Replace)
                .await
                .map_err(|e| {
                    tracing::error!("Failed to stop service {}: {}", service_name, e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;
        }
        ServiceCommand::Restart => {
            manager
                .restart_unit(&service_name, Mode::Replace)
                .await
                .map_err(|e| {
                    tracing::error!("Failed to restart service {}: {}", service_name, e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;
        }
        ServiceCommand::Enable => {
            manager
                .enable_unit_files(&[service_name.as_str()], false, true)
                .await
                .map_err(|e| {
                    tracing::error!("Failed to enable service {}: {}", service_name, e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;
        }
        ServiceCommand::Disable => {
            manager
                .disable_unit_files(&[service_name.as_str()], false)
                .await
                .map_err(|e| {
                    tracing::error!("Failed to disable service {}: {}", service_name, e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;
        }
    }

    get_service(Path(name)).await
}
