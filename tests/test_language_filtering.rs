#[test]
fn test_language_filtering() {
    use jmdict::{self, Enum};
    
    // Use entry 1001820 (お金 - money) which has translations in all languages
    let test_entry_num = 1001820;
    
    let entry = jmdict::entries()
        .find(|e| e.number == test_entry_num)
        .expect("Entry 1001820 (お金) should exist in all configurations");
    
    println!("Testing entry {}", entry.number);
    
    let mut gloss_languages = std::collections::HashSet::new();
    let mut gloss_count = 0;
    
    for sense in entry.senses() {
        for gloss in sense.glosses() {
            gloss_count += 1;
            gloss_languages.insert(gloss.language.code());
            println!("  Found gloss: {} (lang: {})", gloss.text, gloss.language.code());
        }
    }
    
    println!("\nTotal glosses: {}", gloss_count);
    println!("Languages found: {:?}", gloss_languages);
    
    // Verify that we only see glosses in enabled languages
    if cfg!(feature = "translations-eng") && !cfg!(feature = "translations-ger") {
        // With default features (only English), we should only see English glosses
        assert!(gloss_languages.len() == 1, "Expected only 1 language, found {}", gloss_languages.len());
        assert!(gloss_languages.contains("eng"), "Expected English glosses");
    }
    
    // General assertions that apply to all configurations
    assert!(gloss_count > 0, "Should have at least one gloss");
    assert!(gloss_count <= 20, "Expected reasonable number of glosses, found {}", gloss_count);
}