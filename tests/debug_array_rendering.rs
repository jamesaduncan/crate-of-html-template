//! Debug test for array rendering

use html_template::*;

#[test]
fn debug_array_extraction_and_rendering() {
    // Source with array properties
    let source_html = r#"
        <div itemscope itemtype="https://schema.org/Recipe">
            <h1 itemprop="name">Chocolate Cake</h1>
            <ul>
                <li itemprop="recipeIngredient">2 cups flour</li>
                <li itemprop="recipeIngredient">1.5 cups sugar</li>
                <li itemprop="recipeIngredient">3/4 cup cocoa powder</li>
            </ul>
        </div>
    "#;
    
    // Extract microdata to see what we get
    let microdata_items = html_template::microdata::extract_microdata_from_html(source_html).unwrap();
    println!("Extracted microdata: {:#?}", microdata_items);
    
    // Template with array handling
    let template_html = r#"
        <template>
            <div class="recipe">
                <h2 itemprop="name"></h2>
                <ul>
                    <li itemprop="recipeIngredient[]"></li>
                </ul>
            </div>
        </template>
    "#;
    
    let template = HtmlTemplate::from_str(template_html, Some("div.recipe")).unwrap();
    let results = template.render_from_html(source_html).unwrap();
    
    println!("Rendered output: {:#?}", results);
}