// src-tauri/src/components/app_tracker_manager.rs
#![allow(dead_code)]

use crate::state::{AppProject, AppStatus};
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct AppTrackerManagerProps {
    pub projects: Vec<AppProject>,
    pub on_add: EventHandler<AppProject>,
    pub on_remove: EventHandler<String>,
    pub on_status_change: EventHandler<(String, AppStatus)>,
    pub on_archive: EventHandler<(String, bool)>,
}

#[allow(non_snake_case)]
pub fn AppTrackerManager(props: AppTrackerManagerProps) -> Element {
    let mut new_app_name = use_signal(String::new);
    let mut new_app_desc = use_signal(String::new);
    let mut new_app_status = use_signal(|| AppStatus::Want);
    let mut show_add_form = use_signal(|| false);

    let making_count = props.projects.iter().filter(|p| p.status == AppStatus::Making && !p.archived).count();
    let want_count = props.projects.iter().filter(|p| p.status == AppStatus::Want && !p.archived).count();
    let testing_count = props.projects.iter().filter(|p| p.status == AppStatus::Testing && !p.archived).count();
    let done_count = props.projects.iter().filter(|p| p.status == AppStatus::Done && !p.archived).count();

    rsx! {
        div {
            style: "margin-top: 16px;",
            
            // Header with add button
            div {
                style: "display: flex; align-items: center; justify-content: space-between; margin-bottom: 12px;",
                h3 {
                    style: "color: #ffcc00; letter-spacing: 0.02em; font-size: 14px; font-weight: 700; margin: 0;",
                    "APP PROJECTS"
                }
                button {
                    style: "padding: 4px 8px; background: linear-gradient(135deg, #e040fb 0%, #ffcc00 100%); color: #120e1a; border: none; border-radius: 6px; font-size: 11px; font-weight: 600; cursor: pointer;",
                    onclick: move |_| show_add_form.toggle(),
                    if show_add_form() { "✕ Close" } else { "+ Add App" }
                }
            }

            // Add new app form
            if show_add_form() {
                div {
                    style: "padding: 10px; background: rgba(224, 64, 251, 0.15); border: 1px solid #e040fb; border-radius: 8px; margin-bottom: 12px;",
                    
                    input {
                        r#type: "text",
                        placeholder: "App name",
                        value: new_app_name(),
                        oninput: move |evt| new_app_name.set(evt.value()),
                        style: "width: 100%; padding: 6px 8px; margin-bottom: 6px; border-radius: 4px; border: 1px solid #3a2d56; background: rgba(58, 45, 86, 0.5); color: #f7f2ff; font-size: 12px;"
                    }
                    
                    input {
                        r#type: "text",
                        placeholder: "Description",
                        value: new_app_desc(),
                        oninput: move |evt| new_app_desc.set(evt.value()),
                        style: "width: 100%; padding: 6px 8px; margin-bottom: 6px; border-radius: 4px; border: 1px solid #3a2d56; background: rgba(58, 45, 86, 0.5); color: #f7f2ff; font-size: 12px;"
                    }
                    
                    select {
                        value: match new_app_status() {
                            AppStatus::Making => "making",
                            AppStatus::Want => "want",
                            AppStatus::Testing => "testing",
                            AppStatus::Done => "done",
                        },
                        onchange: move |evt| {
                            let status = match evt.value().as_str() {
                                "making" => AppStatus::Making,
                                "testing" => AppStatus::Testing,
                                "done" => AppStatus::Done,
                                _ => AppStatus::Want,
                            };
                            new_app_status.set(status);
                        },
                        style: "width: 100%; padding: 6px 8px; margin-bottom: 8px; border-radius: 4px; border: 1px solid #3a2d56; background: rgba(58, 45, 86, 0.5); color: #f7f2ff; font-size: 12px;",
                        
                        option { value: "want", "Want to Make" }
                        option { value: "making", "Making" }
                        option { value: "testing", "Testing" }
                        option { value: "done", "Done" }
                    }
                    
                    button {
                        style: "width: 100%; padding: 6px; background: linear-gradient(135deg, #e040fb 0%, #ffcc00 100%); color: #120e1a; border: none; border-radius: 4px; font-weight: 600; cursor: pointer; font-size: 12px;",
                        onclick: move |_| {
                            if !new_app_name().is_empty() {
                                let project = AppProject::new(
                                    new_app_name().clone(),
                                    new_app_desc().clone(),
                                    new_app_status(),
                                );
                                props.on_add.call(project);
                                new_app_name.set(String::new());
                                new_app_desc.set(String::new());
                                new_app_status.set(AppStatus::Want);
                                show_add_form.set(false);
                            }
                        },
                        "✓ Add"
                    }
                }
            }

            // Making Apps
            if making_count > 0 {
                div {
                    style: "margin-bottom: 12px;",
                    div {
                        style: "display: flex; align-items: center; gap: 8px; margin-bottom: 8px;",
                        div { style: "width: 6px; height: 6px; border-radius: 50%; background: #e040fb;" }
                        span {
                            style: "font-size: 12px; color: #e040fb; font-weight: 600; text-transform: uppercase;",
                            "Making ({making_count})"
                        }
                    }
                    for project in props.projects.iter().filter(|p| p.status == AppStatus::Making && !p.archived) {
                        AppProjectCard { project: project.clone(), on_status_change: props.on_status_change.clone(), on_remove: props.on_remove.clone(), on_archive: props.on_archive.clone() }
                    }
                }
            }

            // Want to Make Apps
            if want_count > 0 {
                div {
                    style: "margin-bottom: 12px;",
                    div {
                        style: "display: flex; align-items: center; gap: 8px; margin-bottom: 8px;",
                        div { style: "width: 6px; height: 6px; border-radius: 50%; background: #ffcc00;" }
                        span {
                            style: "font-size: 12px; color: #ffcc00; font-weight: 600; text-transform: uppercase;",
                            "Want to Make ({want_count})"
                        }
                    }
                    for project in props.projects.iter().filter(|p| p.status == AppStatus::Want && !p.archived) {
                        AppProjectCard { project: project.clone(), on_status_change: props.on_status_change.clone(), on_remove: props.on_remove.clone(), on_archive: props.on_archive.clone() }
                    }
                }
            }

            // Testing Apps
            if testing_count > 0 {
                div {
                    style: "margin-bottom: 12px;",
                    div {
                        style: "display: flex; align-items: center; gap: 8px; margin-bottom: 8px;",
                        div { style: "width: 6px; height: 6px; border-radius: 50%; background: #7aebbe;" }
                        span {
                            style: "font-size: 12px; color: #7aebbe; font-weight: 600; text-transform: uppercase;",
                            "Testing ({testing_count})"
                        }
                    }
                    for project in props.projects.iter().filter(|p| p.status == AppStatus::Testing && !p.archived) {
                        AppProjectCard { project: project.clone(), on_status_change: props.on_status_change.clone(), on_remove: props.on_remove.clone(), on_archive: props.on_archive.clone() }
                    }
                }
            }

            // Done Apps
            if done_count > 0 {
                div {
                    style: "margin-bottom: 12px;",
                    div {
                        style: "display: flex; align-items: center; gap: 8px; margin-bottom: 8px;",
                        div { style: "width: 6px; height: 6px; border-radius: 50%; background: #4ecca3;" }
                        span {
                            style: "font-size: 12px; color: #4ecca3; font-weight: 600; text-transform: uppercase;",
                            "Done ({done_count})"
                        }
                    }
                    for project in props.projects.iter().filter(|p| p.status == AppStatus::Done && !p.archived) {
                        AppProjectCard { project: project.clone(), on_status_change: props.on_status_change.clone(), on_remove: props.on_remove.clone(), on_archive: props.on_archive.clone() }
                    }
                }
            }

            if making_count == 0 && want_count == 0 && testing_count == 0 && done_count == 0 {
                div {
                    style: "padding: 12px; text-align: center; color: #a99ec3; font-size: 12px; background: rgba(58, 45, 86, 0.2); border-radius: 8px; border: 1px solid #3a2d56;",
                    "No apps yet. Click '+ Add App' →"
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct AppProjectCardProps {
    project: AppProject,
    on_status_change: EventHandler<(String, AppStatus)>,
    on_remove: EventHandler<String>,
    on_archive: EventHandler<(String, bool)>,
}

#[allow(non_snake_case)]
fn AppProjectCard(props: AppProjectCardProps) -> Element {
    let current_status = props.project.status.clone();
    let project_id = props.project.id.clone();
    let project_id_1 = project_id.clone();
    let project_id_2 = project_id.clone();
    let project_id_3 = project_id.clone();
    
    let (other_status_1, label1) = match current_status {
        AppStatus::Making => (AppStatus::Want, "→ Want"),
        AppStatus::Want => (AppStatus::Making, "→ Make"),
        AppStatus::Testing => (AppStatus::Making, "→ Make"),
        AppStatus::Done => (AppStatus::Testing, "← Back"),
    };
    
    let (other_status_2, label2) = match current_status {
        AppStatus::Making => (AppStatus::Testing, "→ Test"),
        AppStatus::Want => (AppStatus::Testing, "→ Test"),
        AppStatus::Testing => (AppStatus::Done, "✓ Done"),
        AppStatus::Done => (AppStatus::Want, "← Back"),
    };

    let color_string = current_status.color().to_string();

    rsx! {
        div {
            style: "padding: 8px 10px; background: rgba(224, 64, 251, 0.1); border: 1px solid {color_string}; border-radius: 8px; margin-bottom: 6px;",
            div {
                style: "display: flex; justify-content: space-between; align-items: start;",
                div {
                    style: "flex: 1;",
                    div { style: "color: #f7f2ff; font-size: 12px; font-weight: 600;", "{props.project.name}" }
                    div { style: "color: #cbd5ff; font-size: 11px; margin-top: 2px;", "{props.project.description}" }
                    if current_status != AppStatus::Want {
                        div { style: "color: #a99ec3; font-size: 10px; margin-top: 4px;", "v{props.project.version}" }
                    }
                }
                div {
                    style: "display: flex; gap: 4px;",
                    button {
                        style: "padding: 3px 6px; background: rgba(224, 64, 251, 0.2); color: #e040fb; border: 1px solid #e040fb; border-radius: 4px; font-size: 10px; cursor: pointer;",
                        onclick: move |_| props.on_status_change.call((project_id.clone(), other_status_1.clone())),
                        "{label1}"
                    }
                    button {
                        style: "padding: 3px 6px; background: rgba(122, 238, 190, 0.2); color: #7aebbe; border: 1px solid #7aebbe; border-radius: 4px; font-size: 10px; cursor: pointer;",
                        onclick: move |_| props.on_status_change.call((project_id_1.clone(), other_status_2.clone())),
                        "{label2}"
                    }
                    button {
                        style: "padding: 3px 6px; background: rgba(76, 204, 163, 0.2); color: #4ecca3; border: 1px solid #4ecca3; border-radius: 4px; font-size: 10px; cursor: pointer;",
                        onclick: move |_| props.on_archive.call((project_id_2.clone(), true)),
                        "📦"
                    }
                    button {
                        style: "padding: 3px 6px; background: rgba(255, 107, 107, 0.2); color: #ff6b6b; border: 1px solid #ff6b6b; border-radius: 4px; font-size: 10px; cursor: pointer;",
                        onclick: move |_| props.on_remove.call(project_id_3.clone()),
                        "✕"
                    }
                }
            }
        }
    }
}
