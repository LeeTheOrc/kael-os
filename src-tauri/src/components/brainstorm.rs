use crate::services::brainstorm::{BrainstormIdea, fetch_brainstorm_ideas, request_new_ideas, toggle_star_idea, cache_ideas_locally, load_cached_ideas};
use crate::auth::AuthService;
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct BrainstormProps {
    pub auth_service: Signal<AuthService>,
}

#[allow(non_snake_case)]
pub fn BrainstormPanel(props: BrainstormProps) -> Element {
    let mut ideas = use_signal(Vec::<BrainstormIdea>::new);
    let mut selected_category = use_signal(|| "all".to_string());
    let mut loading = use_signal(|| false);
    let mut error_msg = use_signal(|| String::new());
    let mut generating = use_signal(|| false);
    let mut custom_prompt = use_signal(String::new);

    // Load cached ideas on mount
    use_effect(move || {
        spawn(async move {
            // Try loading from local cache first
            if let Ok(cached) = load_cached_ideas() {
                if !cached.is_empty() {
                    ideas.set(cached);
                    return;
                }
            }
            
            // Fetch from Firebase if no cache
            let auth = props.auth_service.read();
            if let Some(user) = auth.get_user() {
                loading.set(true);
                match fetch_brainstorm_ideas(&user).await {
                    Ok(fetched) => {
                        ideas.set(fetched.clone());
                        let _ = cache_ideas_locally(&fetched);
                        error_msg.set(String::new());
                    }
                    Err(e) => {
                        error_msg.set(format!("Failed to load ideas: {}", e));
                    }
                }
                loading.set(false);
            }
        });
    });

    let filtered_ideas = use_memo(move || {
        let all_ideas = ideas.read();
        let category = selected_category.read();
        
        if category.as_str() == "all" {
            all_ideas.clone()
        } else if category.as_str() == "starred" {
            all_ideas.iter().filter(|i| i.starred).cloned().collect()
        } else {
            all_ideas.iter().filter(|i| i.category == *category).cloned().collect()
        }
    });

    let new_ideas_count = use_memo(move || {
        ideas.read().iter().filter(|i| !i.starred && i.on_demand == false).count()
    });

    let mut on_refresh = move || {
        let auth = props.auth_service.read();
        if let Some(user) = auth.get_user() {
            loading.set(true);
            spawn(async move {
                match fetch_brainstorm_ideas(&user).await {
                    Ok(fetched) => {
                        ideas.set(fetched.clone());
                        let _ = cache_ideas_locally(&fetched);
                        error_msg.set(String::new());
                    }
                    Err(e) => {
                        error_msg.set(format!("Failed to refresh: {}", e));
                    }
                }
                loading.set(false);
            });
        }
    };

    let mut on_generate_new = move |category: String| {
        let auth = props.auth_service.read();
        if let Some(user) = auth.get_user() {
            generating.set(true);
            let prompt = if category == "custom" {
                Some(custom_prompt.read().clone())
            } else {
                None
            };
            
            spawn(async move {
                match request_new_ideas(&user, &category, prompt).await {
                    Ok(new_idea) => {
                        let mut current = ideas.read().clone();
                        current.insert(0, new_idea);
                        ideas.set(current.clone());
                        let _ = cache_ideas_locally(&current);
                        error_msg.set(String::new());
                        custom_prompt.set(String::new());
                    }
                    Err(e) => {
                        error_msg.set(format!("Failed to generate: {}", e));
                    }
                }
                generating.set(false);
            });
        }
    };

    let on_toggle_star = move |idea_id: String, starred: bool| {
        let auth = props.auth_service.read();
        if let Some(user) = auth.get_user() {
            spawn(async move {
                match toggle_star_idea(&user, &idea_id, starred).await {
                    Ok(_) => {
                        let mut current = ideas.read().clone();
                        if let Some(idea) = current.iter_mut().find(|i| i.id == idea_id) {
                            idea.starred = starred;
                        }
                        ideas.set(current.clone());
                        let _ = cache_ideas_locally(&current);
                    }
                    Err(e) => {
                        error_msg.set(format!("Failed to star: {}", e));
                    }
                }
            });
        }
    };

    rsx! {
        div {
            class: "flex-1 flex flex-col p-4",
            style: "display: flex; flex-direction: column; height: 100%; overflow: hidden;",
            
            // Header with title and new ideas badge
            div {
                style: "display: flex; align-items: center; justify-content: space-between; margin-bottom: 16px;",
                div { style: "display: flex; align-items: center; gap: 12px;",
                    h2 { style: "font-size: 24px; font-weight: bold; color: #f7f2ff; margin: 0;", "💡 Ideas" }
                    if new_ideas_count() > 0 {
                        span {
                            class: "chip",
                            style: "background: linear-gradient(135deg, #e040fb 0%, #ffcc00 100%); color: #120e1a; padding: 4px 10px; border-radius: 12px; font-size: 12px; font-weight: bold;",
                            "{new_ideas_count()} new"
                        }
                    }
                }
                div { style: "display: flex; gap: 8px;",
                    button {
                        style: "padding: 8px 12px; border-radius: 8px; border: 1px solid #3a2d56; background: linear-gradient(135deg, #1f1631 0%, #181024 100%); color: #a99ec3; cursor: pointer;",
                        disabled: loading(),
                        onclick: move |_| on_refresh(),
                        if loading() { "⏳ Loading..." } else { "🔄 Refresh" }
                    }
                }
            }

            // Error message
            if !error_msg().is_empty() {
                div {
                    style: "padding: 12px; border-radius: 8px; background: rgba(255,100,100,0.1); border: 1px solid rgba(255,100,100,0.3); color: #ff9999; margin-bottom: 16px;",
                    "{error_msg()}"
                }
            }

            // Category tabs
            div {
                style: "display: flex; gap: 8px; margin-bottom: 16px; flex-wrap: wrap;",
                for category in ["all", "features", "ui", "optimization", "integration", "starred"] {
                    button {
                        style: if selected_category() == category {
                            "padding: 8px 16px; border-radius: 8px; background: linear-gradient(135deg, #e040fb 0%, #ffcc00 100%); color: #120e1a; border: none; cursor: pointer; font-weight: bold;"
                        } else {
                            "padding: 8px 16px; border-radius: 8px; background: rgba(58,42,80,0.2); color: #a99ec3; border: 1px solid #3a2d56; cursor: pointer;"
                        },
                        onclick: move |_| selected_category.set(category.to_string()),
                        match category {
                            "all" => "All",
                            "features" => "✨ Features",
                            "ui" => "🎨 UI/UX",
                            "optimization" => "⚡ Optimize",
                            "integration" => "🔗 Integrate",
                            "starred" => "⭐ Starred",
                            _ => category
                        }
                    }
                }
            }

            // Generate new ideas section
            div {
                style: "padding: 16px; border-radius: 12px; border: 1px solid #3a2d56; background: linear-gradient(135deg, #1f1631 0%, #181024 100%); margin-bottom: 16px;",
                h3 { style: "margin: 0 0 12px 0; color: #f7f2ff; font-size: 16px;", "🎯 Generate Fresh Ideas" }
                div { style: "display: flex; gap: 8px; flex-wrap: wrap;",
                    for cat in ["features", "ui", "optimization", "integration"] {
                        button {
                            style: "padding: 6px 12px; border-radius: 6px; border: 1px solid #7aebbe; background: linear-gradient(135deg, #7aebbe 0%, #5af0c8 100%); color: #120e1a; cursor: pointer; font-size: 13px;",
                            disabled: generating(),
                            onclick: {
                                let category = cat.to_string();
                                move |_| on_generate_new(category.clone())
                            },
                            if generating() { "⏳" } else { "+" }
                            " {cat}"
                        }
                    }
                }
                
                // Custom prompt
                div { style: "margin-top: 12px;",
                    input {
                        style: "width: 100%; padding: 8px; border-radius: 6px; background: #0f0b1a; border: 1px solid #3a2d56; color: #f7f2ff;",
                        placeholder: "Custom prompt (e.g., 'How to improve performance?')",
                        value: "{custom_prompt()}",
                        oninput: move |e| custom_prompt.set(e.value()),
                    }
                    button {
                        style: "margin-top: 8px; padding: 6px 12px; border-radius: 6px; border: 1px solid #7aebbe; background: linear-gradient(135deg, #7aebbe 0%, #5af0c8 100%); color: #120e1a; cursor: pointer; font-size: 13px;",
                        disabled: generating() || custom_prompt().trim().is_empty(),
                        onclick: move |_| on_generate_new("custom".to_string()),
                        "Generate from Custom Prompt"
                    }
                }
            }

            // Ideas list (scrollable)
            div {
                style: "flex: 1; overflow-y: auto; display: flex; flex-direction: column; gap: 12px;",
                if filtered_ideas.read().is_empty() {
                    div {
                        style: "padding: 40px; text-align: center; color: #a99ec3;",
                        if loading() {
                            "⏳ Loading ideas..."
                        } else {
                            "No ideas yet. Click Refresh to fetch from cloud or Generate to create new ones!"
                        }
                    }
                }
                for idea in filtered_ideas.read().iter() {
                    div {
                        key: "{idea.id}",
                        style: "padding: 16px; border-radius: 12px; border: 1px solid #3a2d56; background: linear-gradient(135deg, #1c162b 0%, #120e1a 100%);",
                        
                        // Header with category and star
                        div { style: "display: flex; align-items: center; justify-content: space-between; margin-bottom: 8px;",
                            span {
                                class: "chip",
                                style: "background: rgba(122,236,190,0.15); color: #7aebbe; padding: 4px 8px; border-radius: 6px; font-size: 11px; text-transform: uppercase;",
                                "{idea.category}"
                            }
                            button {
                                style: "background: none; border: none; font-size: 20px; cursor: pointer; padding: 4px;",
                                onclick: {
                                    let id = idea.id.clone();
                                    let starred = !idea.starred;
                                    move |_| on_toggle_star(id.clone(), starred)
                                },
                                if idea.starred { "⭐" } else { "☆" }
                            }
                        }
                        
                        // Ideas content
                        div {
                            style: "color: #f7f2ff; line-height: 1.6; white-space: pre-wrap; word-wrap: break-word;",
                            "{idea.ideas}"
                        }
                        
                        // Footer with timestamp
                        if let Some(timestamp) = &idea.generated_at {
                            div {
                                style: "margin-top: 12px; padding-top: 12px; border-top: 1px solid #3a2d56; font-size: 12px; color: #a99ec3;",
                                "Generated: {timestamp}"
                                if idea.on_demand {
                                    span { style: "margin-left: 8px; color: #ffcc00;", "• On-demand" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
