use crate::parser::{parse, Line, Option as QuestOption, Stage};
use log::{debug, info};
use std::convert::AsRef;

fn image(option: &QuestOption) -> String {
    match option {
        QuestOption::Event => "&#x2B1C".to_string(),
        QuestOption::Boss => "&#x1F3C1".to_string(),
        QuestOption::Miniboss => "&#x1F47E".to_string(),
        QuestOption::Other(s) => format!("<{}>", s),
    }
}

fn image_tag(option: &QuestOption) -> String {
    format!(r#"<span class="quest-image">{}</span>"#, image(option))
}

fn html_stage(stage: Stage) -> String {
    let images = stage
        .options
        .options
        .iter()
        .map(image_tag)
        .collect::<Vec<String>>()
        .join(" ");
    format!(
        r#"
<div class="stage">
    {}
    <span class="quest-action">{}</span>
</div>"#,
        images, stage.action
    )
}

fn here_stage() -> String {
    r#"
<div class="stage-now">
    YOU ARE HERE
</div>"#
        .to_string()
}

pub fn generate<S: AsRef<str>>(quest_string: S) -> anyhow::Result<String> {
    let quest = parse(quest_string.as_ref().to_string())?;
    let mut output: Vec<String> = vec![];
    for line in quest.lines {
        let html_code = match line {
            Line::Now => here_stage(),
            Line::Stage(stage) => html_stage(stage),
        };
        output.push(html_code);
    }
    let res = output.join("\n");
    debug!("Generated: {}", &res);
    info!("Generated: {} bytes", res.len());
    Ok(res)
}

#[cfg(test)]
mod tests {

    use super::*;
    use test_log::test;

    static SAMPLE1: &str = r"first
[event] second
 ---
  third
[event,boss] third";

    #[test]
    fn test_simple() {
        let out = generate(SAMPLE1);
        assert!(out.is_ok());
    }

    #[test]
    fn test_one_option() {
        let out = generate(SAMPLE1).unwrap();
        assert_eq!(out.matches(r#"<div class="stage">"#).count(), 4);
        assert!(out.find(r#"<span class="quest-image">&#x2B1C</span> <span class="quest-image">&#x1F3C1</span>"#).is_some());
    }
}
