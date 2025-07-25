use html_template::HtmlTemplate;
use serde_json::json;

fn main() {
    // Test 1: div with itemprop
    let div_html = r#"
        <template>
            <article>
                <div itemprop="content"></div>
            </article>
        </template>
    "#;
    
    // Test 1b: div with itemprop AND class
    let div_class_html = r#"
        <template>
            <article>
                <div itemprop="content" class="content"></div>
            </article>
        </template>
    "#;
    
    // Test 2: span with itemprop  
    let span_html = r#"
        <template>
            <article>
                <span itemprop="content"></span>
            </article>
        </template>
    "#;
    
    // Test 3: p with itemprop
    let p_html = r#"
        <template>
            <article>
                <p itemprop="content"></p>
            </article>
        </template>
    "#;
    
    let data = json!({"content": "Test content here"});
    
    println!("=== Testing DIV ===");
    let div_template = HtmlTemplate::from_str(div_html, Some("article")).unwrap();
    let div_result = div_template.render(&data).unwrap();
    println!("DIV result: {}", div_result);
    println!("DIV works: {}", div_result.contains("Test content here"));
    
    println!("\n=== Testing SPAN ===");
    let span_template = HtmlTemplate::from_str(span_html, Some("article")).unwrap();
    let span_result = span_template.render(&data).unwrap();
    println!("SPAN result: {}", span_result);
    println!("SPAN works: {}", span_result.contains("Test content here"));
    
    println!("\n=== Testing P ===");
    let p_template = HtmlTemplate::from_str(p_html, Some("article")).unwrap();
    let p_result = p_template.render(&data).unwrap();
    println!("P result: {}", p_result);
    println!("P works: {}", p_result.contains("Test content here"));
    
    println!("\n=== Testing DIV WITH CLASS ===");
    let div_class_template = HtmlTemplate::from_str(div_class_html, Some("article")).unwrap();
    let div_class_result = div_class_template.render(&data).unwrap();
    println!("DIV+CLASS result: {}", div_class_result);
    println!("DIV+CLASS works: {}", div_class_result.contains("Test content here"));
}