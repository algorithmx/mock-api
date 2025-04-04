use std::collections::HashMap;
use std::sync::RwLock;
use lazy_static::lazy_static;
use crate::schema::ProjectConfig;
use crate::helpers;
use std::fs::read_to_string;
use std::sync::Arc;

lazy_static! {
    static ref PROJECT_CACHE: RwLock<HashMap<String, Arc<ProjectConfig>>> = 
        RwLock::new(HashMap::new());
}

fn get_cached_config(project_name: &str) -> Option<Arc<ProjectConfig>> {
    let cache = PROJECT_CACHE.read().unwrap();
    cache.get(project_name).cloned()
}


pub fn cache_config(project_name: String, config: ProjectConfig) {
    let mut cache = PROJECT_CACHE.write().unwrap();
    cache.insert(project_name, Arc::new(config));
}

// pub fn invalidate_cache(project_name: &str) {
//     let mut cache = PROJECT_CACHE.write().unwrap();
//     cache.remove(project_name);
// }

fn load_file_to_cache(project_name: &str) -> Result<Arc<ProjectConfig>, String> {
    let config_path = helpers::get_project_config_file_path(project_name);
    if !config_path.exists() {
        return Err("Project does not exist.".to_string());
    }
    let content = read_to_string(config_path)
        .map_err(|e| format!("Invalid project configuration file: {}", e))?;

    let mut config: ProjectConfig = serde_json::from_str(&content)
        .map_err(|e| format!("Invalid project configuration format: {}", e))?;

    // Build condition map for each endpoint
    for endpoint in config.endpoints.values_mut() {
        if endpoint.condition_map.is_empty() {
            endpoint.build_condition_map();
        }
    }

    // Cache the config and return the Arc 
    let config_arc = Arc::new(config);
    cache_config(project_name.to_string(), (*config_arc).clone());
    Ok(config_arc)
} 

pub fn get_or_else_load_cached_config(project_name: &str) -> Result<Arc<ProjectConfig>, String> {
    match get_cached_config(project_name) {
        Some(config) => {
            // Create a mutable clone of the config
            let mut config_clone = (*config).clone();
            // Build condition maps for endpoints
            for endpoint in config_clone.endpoints.values_mut() {
                if endpoint.condition_map.is_empty() {
                    endpoint.build_condition_map();
                }
            }
            // Cache the updated config and return it
            let updated_config = Arc::new(config_clone);
            cache_config(project_name.to_string(), (*updated_config).clone());
            Ok(updated_config)
        },
        None => {
            match load_file_to_cache(project_name) {
                Ok(config) => Ok(config),
                Err(e) => Err(e),
            }
        }
    }
}