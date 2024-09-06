use backend::data_processing::converter::html_to_markdown;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_html_to_markdown() {
        let html = "<h1>Hello, World!</h1><p>This is a <strong>test</strong>.</p>";
        let expected = "# Hello, World!\n\nThis is a **test**.";
        assert_eq!(html_to_markdown(html), expected);
    }

    #[test]
    fn test_html_with_image() {
        let html = "<img src=\"image.jpg\" alt=\"An image\">";
        let expected = "![An image](image.jpg)";
        assert_eq!(html_to_markdown(html), expected);
    }

    #[test]
    fn test_html_with_list() {
        let html = "<ul><li>Item 1</li><li>Item 2</li></ul>";
        let expected = "- Item 1\n- Item 2";
        assert_eq!(html_to_markdown(html), expected);
    }

    #[test]
    fn test_html_with_link() {
        let html = "<a href=\"https://example.com\">Example</a>";
        let expected = "[Example](https://example.com)";
        assert_eq!(html_to_markdown(html), expected);
    }

    #[test]
    fn test_html_with_script_and_style() {
        let html =
            "<script>alert('test');</script><style>body { color: red; }</style><p>Content</p>";
        let expected = "Content";
        assert_eq!(html_to_markdown(html), expected);
    }

    #[test]
    fn test_html_with_nested_elements() {
        let html = "<div><p>This is <strong>nested <em>content</em></strong>.</p></div>";
        let expected = "This is **nested *content***.";
        assert_eq!(html_to_markdown(html), expected);
    }

    #[test]
    fn test_html_with_special_characters() {
        let html = "<p>Special characters: &lt; &gt; &amp; &quot;</p>";
        let expected = "Special characters: < > & \"";
        assert_eq!(html_to_markdown(html), expected);
    }

    #[test]
    fn test_html_with_code_block() {
        let html = "<pre><code>fn main() {\n    println!(\"Hello, world!\");\n}</code></pre>";
        let expected = "```\nfn main() {\n    println!(\"Hello, world!\");\n}\n```";
        assert_eq!(html_to_markdown(html), expected);
    }

    #[test]
    fn test_html_with_table() {
        let html = "<table><tr><th>Header 1</th><th>Header 2</th></tr><tr><td>Cell 1</td><td>Cell 2</td></tr></table>";
        let expected = "| Header 1 | Header 2 |\n| --- | --- |\n| Cell 1 | Cell 2 |";
        assert_eq!(html_to_markdown(html), expected);
    }
}
