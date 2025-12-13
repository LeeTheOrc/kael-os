// src-tauri/src/components/icons.rs
// ... existing icons ...

#[allow(non_snake_case)]
pub fn CogIcon(props: SvgIconProps) -> Element {
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            fill: "none",
            view_box: "0 0 24 24",
            stroke_width: "1.5",
            stroke: "currentColor",
            class: "{props.class}",
            path {
                stroke_linecap: "round",
                stroke_linejoin: "round",
                d: "M4.5 12a7.5 7.5 0 0015 0m-15 0a7.5 7.5 0 1115 0m-15 0H3m18 0h-1.5m-15 0a7.5 7.5 0 1115 0m-15 0H3m15-12.75H3m15 12.75h-1.5"
            }
        }
    }
}
