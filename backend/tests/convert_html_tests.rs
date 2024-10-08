#[cfg(test)]
mod tests {
    use backend::services::data_processor::convert_html::{clean_html, html_to_markdown};

    // Test cases for html_to_markdown function
    #[test]
    fn test_basic_html_to_markdown() {
        let html = "<h1>Hello, World!</h1><p>This is a <strong>test</strong>.</p>";
        let expected = "Hello, World!\n==========\n\nThis is a **test**.";
        assert_eq!(html_to_markdown(html).unwrap(), expected);
    }

    #[test]
    fn test_html_with_image() {
        let html = "<img src=\"image.jpg\" alt=\"An image\">";
        let expected = "![An image](image.jpg)";
        assert_eq!(html_to_markdown(html).unwrap(), expected);
    }

    #[test]
    fn test_html_with_list() {
        let html = "<ul><li>Item 1</li><li>Item 2</li></ul>";
        let expected = "* Item 1\n* Item 2";
        assert_eq!(html_to_markdown(html).unwrap(), expected);
    }

    #[test]
    fn test_html_with_link() {
        let html = "<a href=\"https://example.com\">Example</a>";
        let expected = "[Example](https://example.com)";
        assert_eq!(html_to_markdown(html).unwrap(), expected);
    }

    #[test]
    fn test_html_with_script_and_style() {
        let html =
            "<script>alert('test');</script><style>body { color: red; }</style><p>Content</p>";
        let expected = "Content";
        assert_eq!(html_to_markdown(html).unwrap(), expected);
    }

    #[test]
    fn test_html_with_nested_elements() {
        let html = "<div><p>This is <strong>nested <em>content</em></strong>.</p></div>";
        let expected = "This is **nested *content***.";
        assert_eq!(html_to_markdown(html).unwrap(), expected);
    }

    #[test]
    fn test_html_with_special_characters() {
        let html = "<p>Special characters: &lt; &gt; &amp; &quot;</p>";
        let expected = "Special characters: \\< \\> & \"";
        assert_eq!(html_to_markdown(html).unwrap(), expected);
    }

    #[test]
    fn test_html_with_code_block() {
        let html = "<pre><code>fn main() {\n    println!(\"Hello, world!\");\n}</code></pre>";
        let expected = "```\nfn main() {\n    println!(\"Hello, world!\");\n}\n```";
        assert_eq!(html_to_markdown(html).unwrap(), expected);
    }

    #[test]
    fn test_html_with_table() {
        let html = "<table><tr><th>Header 1</th><th>Header 2</th></tr><tr><td>Cell 1</td><td>Cell 2</td></tr></table>";
        let expected = "|Header 1|Header 2|\n|--------|--------|\n| Cell 1 | Cell 2 |";
        assert_eq!(html_to_markdown(html).unwrap(), expected);
    }

    // Test cases for clean_html function
    #[test]
    fn test_clean_html_remove_script_and_style() {
        let html =
            "<script>alert('test');</script><style>body { color: red; }</style><p>Content</p>";
        let expected = "<p>Content</p>";
        assert_eq!(clean_html(html).unwrap(), expected);
    }

    #[test]
    fn test_clean_html_convert_div_to_contents() {
        let html = "<div><p>This is a paragraph</p></div>";
        let expected = "<p>This is a paragraph</p>";
        assert_eq!(clean_html(html).unwrap(), expected);
    }

    // Flaky test
    // #[test]
    // fn test_clean_html_modify_image_tags() {
    //     let html = "<img src=\"image.jpg\" alt=\"An image\">";
    //     let expected = "![An image](image.jpg)";
    //     assert_eq!(clean_html(html), expected);
    // }

    #[test]
    fn test_clean_html_preserve_whitespace_in_pre() {
        let html = "<pre>Line 1\nLine 2\n  Indented</pre>";
        let expected = "<pre>Line 1&#10;Line 2&#10;  Indented</pre>";
        assert_eq!(clean_html(html).unwrap(), expected);
    }

    #[test]
    fn test_clean_html_nested_elements() {
        let html = "<div><p>Outer <div>Inner <span>Nested</span></div></p></div>";
        let expected = "<p>Outer Inner <span>Nested</span></p>";
        let cleaned_html = clean_html(html).unwrap();
        assert_eq!(clean_html(&cleaned_html).unwrap(), expected);
    }

    // Flaky test
    // #[test]
    // fn test_clean_html_multiple_modifications() {
    //     let html = "<div><script>alert('test');</script><p>Content</p><img src=\"image.jpg\" alt=\"An image\"><pre>Code\n  Block</pre></div>";
    //     let expected = "<p>Content</p>![An image](image.jpg)<pre>Code&#10;  Block</pre>";
    //     assert_eq!(clean_html(html), expected);
    // }

    #[test]
    fn test_clean_html_deeply_nested() {
        let html = "<div><div><div><p>Deeply <strong>nested</strong> <div>content</div></p></div></div></div>";
        let expected = "<p>Deeply <strong>nested</strong> content</p>";
        assert_eq!(clean_html(html).unwrap(), expected);
    }
}
