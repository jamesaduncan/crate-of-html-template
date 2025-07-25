use html_template::HtmlTemplate;
use serde_json::json;

fn main() {
    let html = r#"
        <template>
            <div>
                <h2>Products</h2>
                <div itemprop="products[]" class="product">
                    <h3 itemprop="name"></h3>
                    <p>Price: $${price}</p>
                    <div data-constraint="inStock">
                        <button>Add to Cart</button>
                    </div>
                    <div data-constraint="!inStock">
                        <p>Out of Stock</p>
                    </div>
                    <div data-constraint="price < 50">
                        <span class="badge">Budget Friendly!</span>
                    </div>
                </div>
            </div>
        </template>
    "#;

    let template = HtmlTemplate::from_str(html, None).unwrap();

    let data = json!({
        "products": [
            {
                "name": "Laptop",
                "price": 999,
                "inStock": true
            },
            {
                "name": "Mouse", 
                "price": 25,
                "inStock": true
            },
            {
                "name": "Keyboard",
                "price": 75,
                "inStock": false
            }
        ]
    });

    let result = template.render(&data).unwrap();
    
    println!("Result:\n{}", result);
    
    println!("\n=== Checking Laptop (price: 999, inStock: true) ===");
    println!("Contains 'Laptop': {}", result.contains("Laptop"));
    println!("Contains 'Price: $999': {}", result.contains("Price: $999"));
    println!("Contains 'Add to Cart': {}", result.contains("Add to Cart"));
    println!("Contains 'Budget Friendly!': {} (should be false)", result.contains("Budget Friendly!"));
    
    println!("\n=== Checking Mouse (price: 25, inStock: true) ===");
    println!("Contains 'Mouse': {}", result.contains("Mouse"));
    println!("Contains 'Price: $25': {}", result.contains("Price: $25"));
    println!("Contains 'Budget Friendly!': {} (should be true)", result.contains("Budget Friendly!"));
    
    println!("\n=== Checking Keyboard (price: 75, inStock: false) ===");
    println!("Contains 'Keyboard': {}", result.contains("Keyboard"));
    println!("Contains 'Out of Stock': {} (should be true)", result.contains("Out of Stock"));
}