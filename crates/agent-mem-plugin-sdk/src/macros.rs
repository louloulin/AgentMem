//! Convenience macros for plugin development

/// Create plugin metadata
#[macro_export]
macro_rules! plugin_metadata {
    (
        name: $name:expr,
        version: $version:expr,
        description: $description:expr,
        author: $author:expr,
        plugin_type: $plugin_type:expr,
        capabilities: [$($cap:expr),*]
    ) => {
        $crate::PluginMetadata {
            name: $name.to_string(),
            version: $version.to_string(),
            description: $description.to_string(),
            author: $author.to_string(),
            plugin_type: $plugin_type,
            required_capabilities: vec![$($cap),*],
            config_schema: None,
        }
    };
}

/// Create a success response
#[macro_export]
macro_rules! success_response {
    ($data:expr) => {
        $crate::PluginResponse::success($data)
    };
}

/// Create an error response
#[macro_export]
macro_rules! error_response {
    ($error:expr) => {
        $crate::PluginResponse::<()>::error($error.to_string())
    };
}

