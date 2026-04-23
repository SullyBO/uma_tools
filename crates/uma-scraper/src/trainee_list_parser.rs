use log::info;
use scraper::{Html, Selector};
use uma_core::ids::UmaId;

#[derive(Debug)]
pub struct TraineeIndexEntry {
    pub id: UmaId,
    pub name: String,
}

impl TraineeIndexEntry {
    pub fn gametora_url(&self) -> String {
        let slug = self.name.to_lowercase().replace(' ', "-");
        format!(
            "https://gametora.com/umamusume/characters/{}-{}",
            self.id.0, slug
        )
    }
}

/// Parses the trainee list page from the umamusume wiki
pub fn parse_trainee_index(html: &str) -> Vec<TraineeIndexEntry> {
    let document = Html::parse_document(html);
    let row_sel = Selector::parse("table.wikitable tbody tr").unwrap();
    let td_sel = Selector::parse("td").unwrap();
    let img_sel = Selector::parse("img").unwrap();
    let a_sel = Selector::parse("a").unwrap();

    let mut entries: Vec<TraineeIndexEntry> = Vec::new();
    let mut skipped_jp = 0usize;
    let mut skipped_parse = 0usize;

    for row in document.select(&row_sel) {
        let cells: Vec<_> = row.select(&td_sel).collect();
        if cells.len() < 5 {
            skipped_parse += 1;
            continue;
        }

        // Skip JP-only trainees
        if cells[4].value().attr("data-sort-value").is_none() {
            skipped_jp += 1;
            continue;
        }

        // Extract ID from image filename
        let id = match cells[0]
            .select(&img_sel)
            .next()
            .and_then(|img| img.value().attr("src"))
            .and_then(|src| src.split("Icon_").nth(1))
            .and_then(|s| s.split('.').next())
            .and_then(|s| s.parse::<u32>().ok())
        {
            Some(id) => UmaId(id),
            None => {
                skipped_parse += 1;
                continue;
            }
        };

        let name = match cells[2]
            .select(&a_sel)
            .next()
            .map(|a| a.text().collect::<String>().trim().to_string())
        {
            Some(n) if !n.is_empty() => n,
            _ => {
                skipped_parse += 1;
                continue;
            }
        };

        entries.push(TraineeIndexEntry { id, name });
    }

    info!(
        "Trainee index parsing complete: {} entries parsed, {} skipped (JP-only: {}, parse failures: {}) out of {} rows",
        entries.len(),
        skipped_jp + skipped_parse,
        skipped_jp,
        skipped_parse,
        entries.len() + skipped_jp + skipped_parse
    );

    entries
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_row(icon_id: &str, name: &str, en_date: Option<&str>) -> String {
        let en_cell = match en_date {
            Some(date) => format!(r#"<td data-sort-value="1">{date}</td>"#),
            None => "<td></td>".to_string(),
        };

        format!(
            r#"<tr>
                <td><img src="/w/thumb.php?f=Game_Playable_Icon_{icon_id}.png&width=72"></td>
                <td><b>Some Title</b></td>
                <td><a href="/wiki/Character">{name}</a></td>
                <td data-sort-value="1">2021-03-02</td>
                {en_cell}
            </tr>"#
        )
    }

    fn wrap_table(rows: &str) -> String {
        format!(r#"<table class="wikitable"><tbody>{rows}</tbody></table>"#)
    }

    #[test]
    fn parses_valid_en_row() {
        let html = wrap_table(&make_row("100101", "Special Week", Some("2025-06-26")));
        let entries = parse_trainee_index(&html);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, UmaId(100101));
        assert_eq!(entries[0].name, "Special Week");
        assert_eq!(
            entries[0].gametora_url(),
            "https://gametora.com/umamusume/characters/100101-special-week"
        );
    }

    #[test]
    fn filters_jp_only_row() {
        let html = wrap_table(&make_row("100101", "Special Week", None));
        let entries = parse_trainee_index(&html);
        assert!(entries.is_empty());
    }

    #[test]
    fn filters_jp_only_keeps_en() {
        let rows = format!(
            "{}{}",
            make_row("100101", "Special Week", Some("2025-06-26")),
            make_row("100201", "Silence Suzuka", None),
        );
        let html = wrap_table(&rows);
        let entries = parse_trainee_index(&html);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, UmaId(100101));
    }

    #[test]
    fn skips_row_with_missing_image() {
        let html = wrap_table(&format!(
            r#"<tr>
                <td></td>
                <td><b>Some Title</b></td>
                <td><a href="/wiki/Character">Special Week</a></td>
                <td data-sort-value="1">2021-03-02</td>
                <td data-sort-value="1">2025-06-26</td>
            </tr>"#
        ));
        let entries = parse_trainee_index(&html);
        assert!(entries.is_empty());
    }

    #[test]
    fn skips_row_with_missing_name() {
        let html = wrap_table(&format!(
            r#"<tr>
                <td><img src="/w/thumb.php?f=Game_Playable_Icon_100101.png&width=72"></td>
                <td><b>Some Title</b></td>
                <td></td>
                <td data-sort-value="1">2021-03-02</td>
                <td data-sort-value="1">2025-06-26</td>
            </tr>"#
        ));
        let entries = parse_trainee_index(&html);
        assert!(entries.is_empty());
    }
}
