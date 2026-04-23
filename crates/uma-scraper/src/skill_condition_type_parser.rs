use scraper::{ElementRef, Html, Selector};

#[derive(Debug)]
pub struct ConditionType {
    pub cond_key: String,
    pub description: String,
    pub example: Option<String>,
}

/// Parses the skill conditions page from `https://gametora.com/umamusume/skill-condition-viewer`
/// Need to use the html asset as opposed to scraping automatically
/// So we actually parse it from /uma-scraper/assets/skill_condition_viewer.html
pub fn parse_skill_condition_types(html: &str) -> Vec<ConditionType> {
    let document = Html::parse_document(html);
    let cond_sel = Selector::parse("div[class*='conditionviewer_cond__']").unwrap();
    let name_sel = Selector::parse("div[class*='conditionviewer_cond_name__'] b").unwrap();
    let note_sel = Selector::parse("div[class*='conditionviewer_cond_optional__']").unwrap();
    let example_sel = Selector::parse("div[class*='conditionviewer_cond_example__']").unwrap();

    let mut entries = Vec::new();
    let mut skipped = 0usize;

    for cond in document.select(&cond_sel) {
        let cond_key = match cond.select(&name_sel).next() {
            Some(el) => el.text().collect::<String>().trim().to_string(),
            None => {
                skipped += 1;
                continue;
            }
        };

        let divs: Vec<_> = cond
            .children()
            .filter_map(|n| scraper::ElementRef::wrap(n))
            .filter(|el| el.value().name() == "div")
            .collect();

        let description = parse_description(cond, &divs, &note_sel);
        let example = cond
            .select(&example_sel)
            .next()
            .map(|el| parse_example(el, &divs));

        entries.push(ConditionType {
            cond_key,
            description,
            example,
        });
    }

    log::info!(
        "Condition type parsing complete: {} entries parsed, {} skipped",
        entries.len(),
        skipped
    );

    entries
}

fn parse_description(cond: ElementRef, divs: &[ElementRef], note_sel: &Selector) -> String {
    let description_parts: Vec<String> = divs
        .iter()
        .filter(|el| {
            let class = el.value().attr("class").unwrap_or("");
            !class.contains("cond_name")
                && !class.contains("cond_example")
                && !class.contains("cond_optional")
        })
        .map(|el| el.text().collect::<String>().trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let note = cond.select(note_sel).next().map(|el| {
        el.text()
            .collect::<String>()
            .replace("Note:", "")
            .trim()
            .to_string()
    });

    match note {
        Some(n) => format!("{} {}", description_parts.join(" "), n),
        None => description_parts.join(" "),
    }
}

fn parse_example(example_el: ElementRef, divs: &[ElementRef]) -> String {
    let example_text = example_el
        .text()
        .collect::<String>()
        .replace("Example:", "")
        .trim()
        .to_string();

    let meaning = divs
        .iter()
        .skip_while(|d| d.html() != example_el.html())
        .nth(1)
        .map(|d| {
            d.text()
                .collect::<String>()
                .replace("Meaning:", "")
                .trim()
                .to_string()
        })
        .unwrap_or_default();

    format!("{} — {}", example_text, meaning)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wrap(inner: &str) -> String {
        format!(
            r#"<html><body><div class="conditionviewer_cond__LnQzc">{}</div></body></html>"#,
            inner
        )
    }

    #[test]
    fn test_parses_entry_without_note() {
        let html = wrap(
            r#"
            <div class="conditionviewer_cond_name__WOrIu"><b>accumulatetime</b></div>
            <div>The number of seconds since the race has started.</div>
            <div class="conditionviewer_cond_example__NjOJr">
                <span class="conditionviewer_label__pteXY">Example:</span> accumulatetime&gt;=5
            </div>
            <div>
                <span class="conditionviewer_label__pteXY">Meaning:</span> The race has been ongoing for at least 5 seconds.
            </div>
        "#,
        );

        let results = parse_skill_condition_types(&html);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].cond_key, "accumulatetime");
        assert!(
            results[0]
                .description
                .contains("The number of seconds since the race has started")
        );
        assert!(!results[0].description.contains("Note:"));
        let example = results[0].example.as_ref().unwrap();
        assert!(example.contains("accumulatetime>=5"));
        assert!(example.contains("at least 5 seconds"));
    }

    #[test]
    fn test_parses_entry_with_note() {
        let html = wrap(
            r#"
            <div class="conditionviewer_cond_name__WOrIu"><b>all_corner_random</b></div>
            <div>Picks a random point during any corner.</div>
            <div class="conditionviewer_cond_optional__pY5QJ">
                <span class="conditionviewer_label__pteXY">Note:</span> To be more precise, this condition randomly picks four points.
            </div>
            <div class="conditionviewer_cond_example__NjOJr">
                <span class="conditionviewer_label__pteXY">Example:</span> all_corner_random==1
            </div>
            <div>
                <span class="conditionviewer_label__pteXY">Meaning:</span> A random point on any corner is selected.
            </div>
        "#,
        );

        let results = parse_skill_condition_types(&html);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].cond_key, "all_corner_random");
        assert!(
            results[0]
                .description
                .contains("Picks a random point during any corner")
        );
        assert!(
            results[0]
                .description
                .contains("randomly picks four points")
        );
        let example = results[0].example.as_ref().unwrap();
        assert!(example.contains("all_corner_random==1"));
        assert!(example.contains("random point on any corner"));
    }

    #[test]
    fn test_skips_entry_missing_key() {
        let html = wrap(
            r#"
            <div class="conditionviewer_cond_name__WOrIu"></div>
            <div>Some description.</div>
            <div class="conditionviewer_cond_example__NjOJr">
                <span class="conditionviewer_label__pteXY">Example:</span> something==1
            </div>
            <div>
                <span class="conditionviewer_label__pteXY">Meaning:</span> Something.
            </div>
        "#,
        );

        let results = parse_skill_condition_types(&html);
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_parses_multiple_entries() {
        let html = format!(
            r#"<html><body>
            <div class="conditionviewer_cond__LnQzc">
                <div class="conditionviewer_cond_name__WOrIu"><b>accumulatetime</b></div>
                <div>The number of seconds since the race has started.</div>
                <div class="conditionviewer_cond_example__NjOJr"><span class="conditionviewer_label__pteXY">Example:</span> accumulatetime&gt;=5</div>
                <div><span class="conditionviewer_label__pteXY">Meaning:</span> The race has been ongoing for at least 5 seconds.</div>
            </div>
            <div class="conditionviewer_cond__LnQzc">
                <div class="conditionviewer_cond_name__WOrIu"><b>activate_count_all</b></div>
                <div>The number of skills you have activated in the race.</div>
                <div class="conditionviewer_cond_example__NjOJr"><span class="conditionviewer_label__pteXY">Example:</span> activate_count_all&gt;=7</div>
                <div><span class="conditionviewer_label__pteXY">Meaning:</span> You have activated at least 7 skills so far.</div>
            </div>
            </body></html>"#
        );

        let results = parse_skill_condition_types(&html);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].cond_key, "accumulatetime");
        assert_eq!(results[1].cond_key, "activate_count_all");
    }
}
