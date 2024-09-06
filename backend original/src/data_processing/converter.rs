use scraper::{Html, Selector};

pub fn html_to_markdown(html: &str) -> String {
    let cleaned_html = clean_html(html);
    html2md::parse_html(&cleaned_html)
}

fn clean_html(html: &str) -> String {
    let fragment = Html::parse_fragment(html);

    // Create selectors for elements we want to remove or modify
    let script_style_selector = Selector::parse("script, style").unwrap();
    let div_selector = Selector::parse("div").unwrap();
    let img_selector = Selector::parse("img").unwrap();
    let pre_selector = Selector::parse("pre").unwrap();

    // Remove script and style tags
    let mut cleaned_html = fragment
        .root_element()
        .select(&script_style_selector)
        .fold(fragment.root_element().html(), |acc, element| {
            acc.replace(&element.html(), "")
        });

    // Convert div tags to their contents
    let document = Html::parse_document(&cleaned_html);
    for div in document.select(&div_selector) {
        cleaned_html = cleaned_html.replace(&div.html(), &div.inner_html());
    }

    // Modify image tags to include alt text
    let document = Html::parse_document(&cleaned_html);
    for img in document.select(&img_selector) {
        let src = img.value().attr("src").unwrap_or("");
        let alt = img.value().attr("alt").unwrap_or("");
        let new_img = format!("![{}]({})", alt, src);
        cleaned_html = cleaned_html.replace(&img.html(), &new_img);
    }

    // Preserve whitespace in pre tags
    let document = Html::parse_document(&cleaned_html);
    for pre in document.select(&pre_selector) {
        let content = pre.inner_html();
        let preserved = content.replace("\n", "&#10;");
        cleaned_html = cleaned_html.replace(&pre.html(), &format!("<pre>{}</pre>", preserved));
    }

    cleaned_html
}
