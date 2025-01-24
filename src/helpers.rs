use std::path::PathBuf;
use std::str::FromStr;
use std::env;
use crate::web_server::types::Request;


/// Returns the value of an environment variable.
pub fn get_env_var<T: FromStr>(key: &str, default: T) -> T {
  env::var(key)
      .ok()
      .and_then(|val| val.parse().ok())
      .unwrap_or(default)
}

/// Returns the path to a project's config file.
pub fn get_project_config_file_path(project_name: &str) -> PathBuf {
  let database_root_folder = get_env_var("MOCK_SERVER_DB_ROOT", "./database".to_string());
  PathBuf::from(format!("{}/projects/{}.json", database_root_folder, project_name))
}

/// Returns the path to a project's config file from a request.
pub fn config_file_path_from_request(request: &Request) -> PathBuf {
  let project_name = request.params.get("name").unwrap();
  get_project_config_file_path(project_name)
}
