use eyre::Result;
use inquire::{
    formatter::MultiOptionFormatter,
    ui::{Color, RenderConfig, StyleSheet, Styled},
    MultiSelect,
};
use oma_console::WRITER;

use crate::{
    mgr::{Branches, DistroConfig, Mirror, MirrorManager},
    utils::{create_status, distro_and_custom_mirrors, refresh, root},
    APT_CONFIG, BRANCHES_PATH, STATUS_FILE,
};

pub fn execute() -> Result<()> {
    root()?;
    let status = create_status(STATUS_FILE)?;
    let mut mm = MirrorManager::new(status);

    let mm_info = distro_and_custom_mirrors()?;
    let mirrors = mm_info.list_mirrors();

    let mut default = vec![];

    let formatter: MultiOptionFormatter<Mirror> = &|a| format!("Activating {} mirrors", a.len());
    let render_config = RenderConfig {
        selected_checkbox: Styled::new("✔").with_fg(Color::LightGreen),
        help_message: StyleSheet::empty().with_fg(Color::LightBlue),
        unselected_checkbox: Styled::new(" "),
        highlighted_option_prefix: Styled::new(""),
        selected_option: Some(StyleSheet::new().with_fg(Color::DarkCyan)),
        scroll_down_prefix: Styled::new("▼"),
        scroll_up_prefix: Styled::new("▲"),
        ..Default::default()
    };

    // 空行（最多两行）+ tips (最多两行) + prompt（最多两行）
    let page_size = match WRITER.get_height() {
        0 => panic!("Terminal height must be greater than 0"),
        1..=6 => 1,
        x @ 7..=25 => x - 6,
        26.. => 20,
    };

    let enabled_mirrors = mm
        .list_enabled_mirrors()
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<_>>();

    for (i, x) in mirrors.iter().enumerate() {
        if enabled_mirrors.contains(&x.inner().0.to_string()) {
            default.push(i);
        }
    }

    let ans = MultiSelect::new("Select to open or close mirror", mirrors)
        .with_help_message(
            "Press [Space]/[Enter] to toggle selection, [Esc] to apply changes, [Ctrl-c] to abort.",
        )
        .with_formatter(formatter)
        .with_default(&default)
        .with_page_size(page_size as usize)
        .with_render_config(render_config)
        .prompt()
        .ok();

    if ans.is_none() {
        return Ok(());
    }

    let ans = ans.unwrap();
    let ans = ans.iter().map(|x| x.inner().0).collect::<Vec<_>>();

    let mut remove_mirrors = vec![];

    for i in &enabled_mirrors {
        if !ans.contains(&i.as_str()) {
            remove_mirrors.push(i.to_owned());
        }
    }

    let mut add_mirrors = vec![];

    for i in ans {
        if !enabled_mirrors.contains(&i.to_string()) {
            add_mirrors.push(i);
        }
    }

    if !add_mirrors.is_empty() {
        mm.add_mirrors(&mm_info, &add_mirrors)?;
    }

    if !remove_mirrors.is_empty() {
        mm.remove_mirrors(&remove_mirrors)?;
    }

    let branches = Branches::from_path(BRANCHES_PATH)?;
    mm.apply_config(&branches, APT_CONFIG)?;

    if !add_mirrors.is_empty() || !remove_mirrors.is_empty() {
        refresh()?;
    }

    Ok(())
}
