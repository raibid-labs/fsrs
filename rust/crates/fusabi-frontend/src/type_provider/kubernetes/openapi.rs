//! OpenAPI parsing utilities for Kubernetes provider

use crate::type_provider::error::{ProviderError, ProviderResult};
use std::collections::HashMap;

/// Kubernetes API group/version info
#[derive(Debug, Clone)]
pub struct ApiGroupVersion {
    pub group: String,
    pub version: String,
}

impl ApiGroupVersion {
    pub fn parse(full_name: &str) -> Option<Self> {
        // Parse names like "io.k8s.api.core.v1.Pod"
        let parts: Vec<&str> = full_name.split('.').collect();

        // Find version marker (v1, v2, etc.)
        for (i, part) in parts.iter().enumerate() {
            if part.starts_with('v') && part.len() >= 2 && part.chars().nth(1).map(|c| c.is_numeric()).unwrap_or(false) {
                if i > 0 {
                    return Some(Self {
                        group: parts[..i].join("."),
                        version: part.to_string(),
                    });
                }
            }
        }

        None
    }

    pub fn module_name(&self) -> String {
        let group_name = self.group.split('.').last().unwrap_or("core");
        let group_pascal = capitalize(group_name);
        let version_upper = self.version.to_uppercase();
        format!("{}.{}", group_pascal, version_upper)
    }
}

/// Known Kubernetes resource kinds
#[derive(Debug, Clone, PartialEq)]
pub enum K8sResourceKind {
    // Core
    Pod,
    Service,
    ConfigMap,
    Secret,
    Namespace,
    Node,
    PersistentVolume,
    PersistentVolumeClaim,
    ServiceAccount,
    Endpoints,
    Event,

    // Apps
    Deployment,
    StatefulSet,
    DaemonSet,
    ReplicaSet,

    // Batch
    Job,
    CronJob,

    // Networking
    Ingress,
    NetworkPolicy,

    // Storage
    StorageClass,

    // RBAC
    Role,
    RoleBinding,
    ClusterRole,
    ClusterRoleBinding,

    // Autoscaling
    HorizontalPodAutoscaler,

    // Other
    Custom(String),
}

impl K8sResourceKind {
    pub fn from_type_name(name: &str) -> Self {
        match name {
            "Pod" => Self::Pod,
            "Service" => Self::Service,
            "ConfigMap" => Self::ConfigMap,
            "Secret" => Self::Secret,
            "Namespace" => Self::Namespace,
            "Node" => Self::Node,
            "PersistentVolume" => Self::PersistentVolume,
            "PersistentVolumeClaim" => Self::PersistentVolumeClaim,
            "ServiceAccount" => Self::ServiceAccount,
            "Endpoints" => Self::Endpoints,
            "Event" => Self::Event,
            "Deployment" => Self::Deployment,
            "StatefulSet" => Self::StatefulSet,
            "DaemonSet" => Self::DaemonSet,
            "ReplicaSet" => Self::ReplicaSet,
            "Job" => Self::Job,
            "CronJob" => Self::CronJob,
            "Ingress" => Self::Ingress,
            "NetworkPolicy" => Self::NetworkPolicy,
            "StorageClass" => Self::StorageClass,
            "Role" => Self::Role,
            "RoleBinding" => Self::RoleBinding,
            "ClusterRole" => Self::ClusterRole,
            "ClusterRoleBinding" => Self::ClusterRoleBinding,
            "HorizontalPodAutoscaler" => Self::HorizontalPodAutoscaler,
            other => Self::Custom(other.to_string()),
        }
    }

    pub fn api_group(&self) -> &'static str {
        match self {
            Self::Pod | Self::Service | Self::ConfigMap | Self::Secret |
            Self::Namespace | Self::Node | Self::PersistentVolume |
            Self::PersistentVolumeClaim | Self::ServiceAccount |
            Self::Endpoints | Self::Event => "",

            Self::Deployment | Self::StatefulSet | Self::DaemonSet |
            Self::ReplicaSet => "apps",

            Self::Job | Self::CronJob => "batch",

            Self::Ingress | Self::NetworkPolicy => "networking.k8s.io",

            Self::StorageClass => "storage.k8s.io",

            Self::Role | Self::RoleBinding | Self::ClusterRole |
            Self::ClusterRoleBinding => "rbac.authorization.k8s.io",

            Self::HorizontalPodAutoscaler => "autoscaling",

            Self::Custom(_) => "",
        }
    }
}

/// Extract definitions from OpenAPI spec
pub fn extract_definitions(
    openapi: &serde_json::Value
) -> ProviderResult<HashMap<String, serde_json::Value>> {
    let definitions = openapi.get("definitions")
        .or_else(|| openapi.get("components").and_then(|c| c.get("schemas")))
        .and_then(|d| d.as_object())
        .ok_or_else(|| ProviderError::ParseError("No definitions in OpenAPI spec".to_string()))?;

    Ok(definitions.iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect())
}

/// Check if a type name is a core Kubernetes resource
pub fn is_core_resource(type_name: &str) -> bool {
    matches!(K8sResourceKind::from_type_name(type_name),
        K8sResourceKind::Pod | K8sResourceKind::Service | K8sResourceKind::Deployment |
        K8sResourceKind::ConfigMap | K8sResourceKind::Secret | K8sResourceKind::Namespace |
        K8sResourceKind::StatefulSet | K8sResourceKind::DaemonSet | K8sResourceKind::Job |
        K8sResourceKind::CronJob | K8sResourceKind::Ingress
    )
}

/// Helper to capitalize first letter
fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(chars).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_api_group_version() {
        let gv = ApiGroupVersion::parse("io.k8s.api.core.v1.Pod").unwrap();
        assert_eq!(gv.group, "io.k8s.api.core");
        assert_eq!(gv.version, "v1");
    }

    #[test]
    fn test_module_name() {
        let gv = ApiGroupVersion {
            group: "io.k8s.api.core".to_string(),
            version: "v1".to_string()
        };
        assert_eq!(gv.module_name(), "Core.V1");
    }

    #[test]
    fn test_resource_kind() {
        assert_eq!(K8sResourceKind::from_type_name("Pod"), K8sResourceKind::Pod);
        assert_eq!(K8sResourceKind::from_type_name("Deployment"), K8sResourceKind::Deployment);
        assert!(matches!(K8sResourceKind::from_type_name("CustomResource"), K8sResourceKind::Custom(_)));
    }
}
