// src-tauri/src/components/icons.rs
use dioxus::prelude::*;

#[allow(non_snake_case)]
pub fn SendIcon(props: SvgIconProps) -> Element {
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            fill: "none",
            view_box: "0 0 24 24",
            width: "16",
            height: "16",
            stroke_width: "1.5",
            stroke: "currentColor",
            class: "{props.class}",
            path {
                vector_effect: "non-scaling-stroke",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                d: "M6 12L3.269 3.126A59.768 59.768 0 0121.485 12 59.77 59.77 0 013.27 20.876L5.999 12zm0 0h7.5"
            }
        }
    }
}

#[allow(non_snake_case)]
pub fn CogIcon(props: SvgIconProps) -> Element {
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            fill: "none",
            view_box: "0 0 24 24",
            width: "16",
            height: "16",
            stroke_width: "1.5",
            stroke: "currentColor",
            class: "{props.class}",
            path {
                vector_effect: "non-scaling-stroke",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                d: "M4.5 12a7.5 7.5 0 0015 0m-15 0a7.5 7.5 0 1115 0m-15 0H3m18 0h-1.5m-15 0a7.5 7.5 0 1115 0m-15 0H3m15-12.75H3m15 12.75h-1.5"
            }
        }
    }
}

#[allow(non_snake_case)]
pub fn SparkIcon(props: SvgIconProps) -> Element {
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            fill: "none",
            view_box: "0 0 24 24",
            width: "16",
            height: "16",
            stroke_width: "1.6",
            stroke: "currentColor",
            class: "{props.class}",
            path {
                vector_effect: "non-scaling-stroke",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                d: "M12 2l2.5 5.5L20 10l-5.5 2.5L12 18l-2.5-5.5L4 10l5.5-2.5L12 2z"
            }
        }
    }
}

#[allow(non_snake_case)]
pub fn PanelIcon(props: SvgIconProps) -> Element {
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            fill: "none",
            view_box: "0 0 24 24",
            width: "16",
            height: "16",
            stroke_width: "1.5",
            stroke: "currentColor",
            class: "{props.class}",
            path { vector_effect: "non-scaling-stroke", stroke_linecap: "round", stroke_linejoin: "round", d: "M4 5h16v14H4zM4 10h16" }
        }
    }
}
#[allow(non_snake_case)]
pub fn KaelAvatar() -> Element {
    rsx! {
        div {
            class: "flex items-center justify-center font-bold",
            style: "width: 44px; height: 44px; border-radius: 12px; background: radial-gradient(circle at 30% 30%, #e040fb 0%, #120e1a 55%, #0f0c1a 100%); color: #f7f2ff; font-size: 16px; display: flex; align-items: center; justify-content: center; border: 1px solid #3a2d56; box-shadow: 0 4px 12px #00000077, inset 3px 0 0 #ffcc00; overflow: hidden;",
            KaelSigilIcon { class: "w-7 h-7 text-[#f7f2ff]" }
        }
    }
}

#[allow(non_snake_case)]
pub fn KaelSigilIcon(props: SvgIconProps) -> Element {
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            fill: "none",
            view_box: "0 0 24 24",
            width: "16",
            height: "16",
            stroke_width: "1.8",
            stroke: "currentColor",
            class: "{props.class}",
            path { vector_effect: "non-scaling-stroke", stroke_linecap: "round", stroke_linejoin: "round", d: "M4 20V4M4 12h8M12 12l6-8M12 12l6 8" }
        }
    }
}

#[allow(non_snake_case)]
pub fn ArchitectAvatar() -> Element {
    rsx! {
        div {
            class: "flex items-center justify-center font-bold",
            style: "width: 44px; height: 44px; border-radius: 12px; background: linear-gradient(160deg, #60a5fa 0%, #3b82f6 70%, #1d4ed8 100%); color: #0f172a; font-size: 16px; display: flex; align-items: center; justify-content: center; border: 1px solid #93c5fd55; box-shadow: 0 4px 10px #00000055, inset 4px 0 0 #7aebbe; overflow: hidden;",
            span { style: "font-weight: 800;", "A" }
        }
    }
}
#[derive(Props, Clone, PartialEq)]
pub struct SvgIconProps {
    #[props(default = "w-6 h-6".to_string())]
    class: String,
}
