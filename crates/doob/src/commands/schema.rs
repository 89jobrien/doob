use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CliManifest {
    pub name: String,
    pub version: String,
    pub commands: Vec<CommandSchema>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandSchema {
    /// Flat command name, e.g. "todo list", "todo add", "note add"
    pub name: String,
    pub description: String,
    pub params: Vec<ParamSchema>,
    /// true if the command supports --json output
    pub json_output: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ParamSchema {
    pub name: String,
    pub flag: String,
    pub required: bool,
    pub description: String,
    #[serde(rename = "type")]
    pub ty: String,
}

pub fn build_manifest() -> CliManifest {
    CliManifest {
        name: "doob".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        commands: vec![
            CommandSchema {
                name: "todo add".to_string(),
                description: "Add one or more todos".to_string(),
                json_output: true,
                params: vec![
                    ParamSchema {
                        name: "content".to_string(),
                        flag: "--content".to_string(),
                        required: true,
                        description: "Task description(s)".to_string(),
                        ty: "array".to_string(),
                    },
                    ParamSchema {
                        name: "priority".to_string(),
                        flag: "--priority".to_string(),
                        required: false,
                        description: "Priority level".to_string(),
                        ty: "string".to_string(),
                    },
                    ParamSchema {
                        name: "project".to_string(),
                        flag: "--project".to_string(),
                        required: false,
                        description: "Project name".to_string(),
                        ty: "string".to_string(),
                    },
                    ParamSchema {
                        name: "tags".to_string(),
                        flag: "--tags".to_string(),
                        required: false,
                        description: "Comma-separated tags".to_string(),
                        ty: "string".to_string(),
                    },
                    ParamSchema {
                        name: "blocks".to_string(),
                        flag: "--blocks".to_string(),
                        required: false,
                        description: "UUIDs this todo blocks".to_string(),
                        ty: "string".to_string(),
                    },
                    ParamSchema {
                        name: "blocked_by".to_string(),
                        flag: "--blocked-by".to_string(),
                        required: false,
                        description: "UUIDs that block this todo".to_string(),
                        ty: "string".to_string(),
                    },
                ],
            },
            CommandSchema {
                name: "todo list".to_string(),
                description: "List todos".to_string(),
                json_output: true,
                params: vec![
                    ParamSchema {
                        name: "status".to_string(),
                        flag: "--status".to_string(),
                        required: false,
                        description: "Filter by status".to_string(),
                        ty: "string".to_string(),
                    },
                    ParamSchema {
                        name: "project".to_string(),
                        flag: "--project".to_string(),
                        required: false,
                        description: "Filter by project".to_string(),
                        ty: "string".to_string(),
                    },
                    ParamSchema {
                        name: "limit".to_string(),
                        flag: "--limit".to_string(),
                        required: false,
                        description: "Max results".to_string(),
                        ty: "integer".to_string(),
                    },
                ],
            },
            CommandSchema {
                name: "todo complete".to_string(),
                description: "Complete one or more todos".to_string(),
                json_output: true,
                params: vec![ParamSchema {
                    name: "ids".to_string(),
                    flag: "--ids".to_string(),
                    required: true,
                    description: "Todo ID(s)".to_string(),
                    ty: "array".to_string(),
                }],
            },
            CommandSchema {
                name: "todo undo".to_string(),
                description: "Undo completion — mark todos as pending".to_string(),
                json_output: true,
                params: vec![ParamSchema {
                    name: "ids".to_string(),
                    flag: "--ids".to_string(),
                    required: true,
                    description: "Todo ID(s)".to_string(),
                    ty: "array".to_string(),
                }],
            },
            CommandSchema {
                name: "todo remove".to_string(),
                description: "Remove todos".to_string(),
                json_output: true,
                params: vec![ParamSchema {
                    name: "ids".to_string(),
                    flag: "--ids".to_string(),
                    required: true,
                    description: "Todo ID(s)".to_string(),
                    ty: "array".to_string(),
                }],
            },
            CommandSchema {
                name: "todo due".to_string(),
                description: "Set or clear due date for a todo".to_string(),
                json_output: true,
                params: vec![
                    ParamSchema {
                        name: "id".to_string(),
                        flag: "--id".to_string(),
                        required: true,
                        description: "Todo ID".to_string(),
                        ty: "string".to_string(),
                    },
                    ParamSchema {
                        name: "date".to_string(),
                        flag: "--date".to_string(),
                        required: false,
                        description: "Due date (YYYY-MM-DD or 'clear')".to_string(),
                        ty: "string".to_string(),
                    },
                ],
            },
            CommandSchema {
                name: "todo deps".to_string(),
                description: "Show dependency chain for a todo".to_string(),
                json_output: true,
                params: vec![ParamSchema {
                    name: "id".to_string(),
                    flag: "--id".to_string(),
                    required: true,
                    description: "Todo UUID".to_string(),
                    ty: "string".to_string(),
                }],
            },
            CommandSchema {
                name: "note add".to_string(),
                description: "Add one or more notes".to_string(),
                json_output: true,
                params: vec![
                    ParamSchema {
                        name: "content".to_string(),
                        flag: "--content".to_string(),
                        required: true,
                        description: "Note content".to_string(),
                        ty: "array".to_string(),
                    },
                    ParamSchema {
                        name: "project".to_string(),
                        flag: "--project".to_string(),
                        required: false,
                        description: "Project name".to_string(),
                        ty: "string".to_string(),
                    },
                    ParamSchema {
                        name: "tags".to_string(),
                        flag: "--tags".to_string(),
                        required: false,
                        description: "Comma-separated tags".to_string(),
                        ty: "string".to_string(),
                    },
                ],
            },
            CommandSchema {
                name: "note list".to_string(),
                description: "List notes".to_string(),
                json_output: true,
                params: vec![ParamSchema {
                    name: "project".to_string(),
                    flag: "--project".to_string(),
                    required: false,
                    description: "Filter by project".to_string(),
                    ty: "string".to_string(),
                }],
            },
            CommandSchema {
                name: "note remove".to_string(),
                description: "Remove notes".to_string(),
                json_output: true,
                params: vec![ParamSchema {
                    name: "ids".to_string(),
                    flag: "--ids".to_string(),
                    required: true,
                    description: "Note ID(s)".to_string(),
                    ty: "array".to_string(),
                }],
            },
            CommandSchema {
                name: "search".to_string(),
                description: "Full-text search across todos and notes".to_string(),
                json_output: true,
                params: vec![
                    ParamSchema {
                        name: "query".to_string(),
                        flag: "--query".to_string(),
                        required: true,
                        description: "Search query".to_string(),
                        ty: "string".to_string(),
                    },
                    ParamSchema {
                        name: "type".to_string(),
                        flag: "--type".to_string(),
                        required: false,
                        description: "Filter by type: todo, note, or all".to_string(),
                        ty: "string".to_string(),
                    },
                    ParamSchema {
                        name: "project".to_string(),
                        flag: "--project".to_string(),
                        required: false,
                        description: "Filter by project".to_string(),
                        ty: "string".to_string(),
                    },
                ],
            },
            CommandSchema {
                name: "stats".to_string(),
                description: "Analytics and statistics".to_string(),
                json_output: true,
                params: vec![
                    ParamSchema {
                        name: "project".to_string(),
                        flag: "--project".to_string(),
                        required: false,
                        description: "Filter by project".to_string(),
                        ty: "string".to_string(),
                    },
                    ParamSchema {
                        name: "window".to_string(),
                        flag: "--window".to_string(),
                        required: false,
                        description: "Time window in days".to_string(),
                        ty: "integer".to_string(),
                    },
                ],
            },
            CommandSchema {
                name: "handoff list".to_string(),
                description: "List handoff items".to_string(),
                json_output: true,
                params: vec![
                    ParamSchema {
                        name: "project".to_string(),
                        flag: "--project".to_string(),
                        required: false,
                        description: "Filter by project".to_string(),
                        ty: "string".to_string(),
                    },
                    ParamSchema {
                        name: "status".to_string(),
                        flag: "--status".to_string(),
                        required: false,
                        description: "Filter by status".to_string(),
                        ty: "string".to_string(),
                    },
                ],
            },
            CommandSchema {
                name: "handoff sync".to_string(),
                description: "Bidirectional sync with HANDOFF.yaml".to_string(),
                json_output: true,
                params: vec![ParamSchema {
                    name: "file".to_string(),
                    flag: "--file".to_string(),
                    required: true,
                    description: "Path to HANDOFF.yaml".to_string(),
                    ty: "string".to_string(),
                }],
            },
            CommandSchema {
                name: "archive list".to_string(),
                description: "List archived todos".to_string(),
                json_output: true,
                params: vec![ParamSchema {
                    name: "project".to_string(),
                    flag: "--project".to_string(),
                    required: false,
                    description: "Filter by project".to_string(),
                    ty: "string".to_string(),
                }],
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_serializes_to_json() {
        let manifest = build_manifest();
        let json = serde_json::to_string(&manifest).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["name"], "doob");
        assert!(v["commands"].as_array().unwrap().len() > 5);
    }

    #[test]
    fn todo_list_command_present() {
        let manifest = build_manifest();
        let cmd = manifest
            .commands
            .iter()
            .find(|c| c.name == "todo list")
            .unwrap();
        assert!(cmd.params.iter().any(|p| p.name == "status"));
        assert!(cmd.params.iter().any(|p| p.name == "project"));
    }

    #[test]
    fn schema_command_outputs_valid_json() {
        let manifest = build_manifest();
        let json = serde_json::to_string_pretty(&manifest).unwrap();
        let back: CliManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "doob");
        assert!(!back.version.is_empty());
    }
}
