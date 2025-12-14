#![allow(dead_code)]

use crate::state::AppProject;
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct ProjectArchiveSettingsProps {
    pub archived_projects: Vec<AppProject>,
    pub on_restore: EventHandler<(String, bool)>, // (project_id, unarchive)
    pub on_delete: EventHandler<String>,
}

#[allow(non_snake_case)]
pub fn ProjectArchiveSettings(props: ProjectArchiveSettingsProps) -> Element {
    let archived_count = props.archived_projects.len();

    rsx! {
        div {
            style: "padding: 12px; background: rgba(224, 64, 251, 0.1); border: 1px solid #e040fb; border-radius: 8px;",
            
            div {
                style: "display: flex; align-items: center; gap: 8px; margin-bottom: 12px;",
                div { style: "width: 6px; height: 6px; border-radius: 50%; background: #4ecca3;" }
                span {
                    style: "font-size: 13px; color: #4ecca3; font-weight: 600; text-transform: uppercase;",
                    "📦 Archived Projects ({archived_count})"
                }
            }

            if archived_count == 0 {
                div {
                    style: "padding: 8px; text-align: center; color: #a99ec3; font-size: 11px; background: rgba(58, 45, 86, 0.2); border-radius: 6px; border: 1px dashed #3a2d56;",
                    "No archived projects. Archive projects by clicking 📦 on them to keep things organized."
                }
            } else {
                div {
                    style: "max-height: 300px; overflow-y: auto;",
                    for project in props.archived_projects.iter() {
                        ArchivedProjectItem {
                            project: project.clone(),
                            on_restore: props.on_restore.clone(),
                            on_delete: props.on_delete.clone()
                        }
                    }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct ArchivedProjectItemProps {
    project: AppProject,
    on_restore: EventHandler<(String, bool)>,
    on_delete: EventHandler<String>,
}

#[allow(non_snake_case)]
fn ArchivedProjectItem(props: ArchivedProjectItemProps) -> Element {
    let project_id = props.project.id.clone();
    let project_id_2 = project_id.clone();

    rsx! {
        div {
            style: "padding: 8px; background: rgba(76, 204, 163, 0.05); border: 1px solid rgba(76, 204, 163, 0.3); border-radius: 6px; margin-bottom: 6px;",
            div {
                style: "display: flex; justify-content: space-between; align-items: start;",
                div {
                    style: "flex: 1;",
                    div { style: "color: #f7f2ff; font-size: 11px; font-weight: 600;", "{props.project.name}" }
                    div { style: "color: #cbd5ff; font-size: 10px; margin-top: 1px;", "{props.project.description}" }
                    div { style: "color: #a99ec3; font-size: 9px; margin-top: 3px;", "Status: {props.project.status.label()}" }
                }
                div {
                    style: "display: flex; gap: 3px;",
                    button {
                        style: "padding: 2px 4px; background: rgba(122, 238, 190, 0.2); color: #7aebbe; border: 1px solid #7aebbe; border-radius: 3px; font-size: 9px; cursor: pointer;",
                        onclick: move |_| props.on_restore.call((project_id.clone(), false)),
                        "↺ Restore"
                    }
                    button {
                        style: "padding: 2px 4px; background: rgba(255, 107, 107, 0.2); color: #ff6b6b; border: 1px solid #ff6b6b; border-radius: 3px; font-size: 9px; cursor: pointer;",
                        onclick: move |_| props.on_delete.call(project_id_2.clone()),
                        "🗑️"
                    }
                }
            }
        }
    }
}
