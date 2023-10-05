use crate::parser::{parse, Line, Option as QuestOption, Options, Stage};
use log::{debug, info};
use std::convert::AsRef;

fn encode(option: &QuestOption) -> Option<String> {
    match option {
        QuestOption::Event => Some("&#x2139;&#xFE0F".to_string()),
        QuestOption::Boss => Some("&#x1F3C1".to_string()),
        QuestOption::Miniboss => Some("&#x1F47E".to_string()),
        QuestOption::Fail => Some("&#x1F4A9".to_string()),
        QuestOption::Research => Some("&#x1F9D1;&#x200D;&#x1F52C;".to_string()),
        QuestOption::Other(_) => None,
    }
}

fn image_tag(option: &QuestOption) -> String {
    if let Some(emoji) = encode(option) {
        format!(r#"<span class="quest-image">{}</span>"#, emoji)
    } else if let QuestOption::Other(plain) = option {
        format!(r#"<em class="quest-other">{}</em>"#, plain)
    } else {
        panic!("Uncovered option {:?}", option);
    }
}

fn html_options(options: &Options) -> String {
    let concatenated = options
        .options
        .iter()
        .map(image_tag)
        .collect::<Vec<String>>()
        .join("&nbsp;");
    if concatenated.is_empty() {
        concatenated
    } else {
        format!(r#"<div class="quest-options">{}</div>"#, concatenated)
    }
}

fn html_stage(stage: Stage, is_future: bool) -> String {
    let options_class = if is_future { "stage future" } else { "stage" };
    let action_class = if stage
        .options
        .options
        .iter()
        .any(|opt| matches!(opt, QuestOption::Miniboss | QuestOption::Boss))
    {
        "quest-action quest-important"
    } else {
        "quest-action"
    };
    format!(
        r#"
<div class="{}">
    {}
    <span class="{}">{}</span>
</div>"#,
        options_class,
        html_options(&stage.options),
        action_class,
        stage.action
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
    let mut is_future: bool = false;
    for line in quest.lines {
        is_future = is_future || matches!(&line, Line::Now);
        let html_code = match line {
            Line::Now => here_stage(),
            Line::Stage(stage) => html_stage(stage, is_future),
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
        assert_eq!(out.matches(r#"<div class="stage">"#).count(), 2);
        assert_eq!(out.matches(r#"<div class="stage future">"#).count(), 2);
        assert!(out.find(r#"<span class="quest-image">&#x2139;&#xFE0F</span>&nbsp;<span class="quest-image">&#x1F3C1</span>"#).is_some());
    }

    #[test]
    fn test_other_option() {
        let out = generate("[event,unknown] fifth").unwrap();
        assert_eq!(out.matches(r#"<div class="stage">"#).count(), 1);
        assert!(out.find(r#"<span class="quest-image">&#x2139;&#xFE0F</span>&nbsp;<em class="quest-other">unknown</em>"#).is_some());
    }
}
