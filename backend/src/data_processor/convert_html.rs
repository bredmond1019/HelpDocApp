use scraper::{Html, Selector};

pub fn html_to_markdown(html: &str) -> String {
    let cleaned_html = clean_html(html);
    html2md::parse_html(&cleaned_html)
}

pub fn clean_html(html: &str) -> String {
    let mut cleaned_html = html.to_string();
    let mut previous_html;

    // Create selectors for elements we want to remove or modify
    let script_style_selector = Selector::parse("script, style").unwrap();
    let div_selector = Selector::parse("div").unwrap();
    let img_selector = Selector::parse("img").unwrap();
    let pre_selector = Selector::parse("pre").unwrap();

    loop {
        previous_html = cleaned_html.clone();

        let fragment = Html::parse_fragment(&cleaned_html);

        // Remove script and style tags
        for element in fragment.select(&script_style_selector) {
            cleaned_html = cleaned_html.replace(&element.html(), "");
        }

        // Convert div tags to their contents
        for div in fragment.select(&div_selector) {
            cleaned_html = cleaned_html.replace(&div.html(), &div.inner_html());
        }

        // Modify image tags to include alt text
        for img in fragment.select(&img_selector) {
            let src = img.value().attr("src").unwrap_or("");
            let alt = img.value().attr("alt").unwrap_or("");
            let new_img = format!("![{}]({})", alt, src);
            cleaned_html = cleaned_html.replace(&img.html(), &new_img);
        }

        // Preserve whitespace in pre tags
        for pre in fragment.select(&pre_selector) {
            let content = pre.inner_html();
            let preserved = content.replace("\n", "&#10;");
            cleaned_html = cleaned_html.replace(&pre.html(), &format!("<pre>{}</pre>", preserved));
        }

        // If no changes were made in this iteration, break the loop
        if cleaned_html == previous_html {
            break;
        }
    }

    cleaned_html.trim().to_string()
}
