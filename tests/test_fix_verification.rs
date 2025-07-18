#[test]
fn test_language_grouping_fix() {
    use jmdict::{self, Enum};
    use std::collections::HashSet;
    
    // Test that we have only 1 sense regardless of language features
    // Use entry 1001820 (お金 - money) which has translations in all languages
    let test_entry_num = 1001820;
    
    let entry = jmdict::entries()
        .find(|e| e.number == test_entry_num)
        .expect("Entry 1001820 (お金) should exist in all configurations");
    
    let sense_count = entry.senses().count();
    let mut languages = HashSet::new();
    let mut gloss_count = 0;
    
    for sense in entry.senses() {
        for gloss in sense.glosses() {
            gloss_count += 1;
            languages.insert(gloss.language.code());
        }
    }
    
    println!("Entry {} test:", test_entry_num);
    println!("  Senses: {}", sense_count);
    println!("  Total glosses: {}", gloss_count);
    println!("  Languages: {:?}", languages);
    
    // The fix ensures we have 1 sense, not multiple senses for different languages
    assert_eq!(sense_count, 1, "Should have exactly 1 sense after language grouping fix");
    assert!(gloss_count > 0, "Should have at least one gloss");
    
    // Verify that only enabled languages are present
    for lang in &languages {
        match *lang {
            "eng" => assert!(cfg!(feature = "translations-eng"), "English should only appear if feature is enabled"),
            "dut" => assert!(cfg!(feature = "translations-dut"), "Dutch should only appear if feature is enabled"),
            "fre" => assert!(cfg!(feature = "translations-fre"), "French should only appear if feature is enabled"),
            "ger" => assert!(cfg!(feature = "translations-ger"), "German should only appear if feature is enabled"),
            "hun" => assert!(cfg!(feature = "translations-hun"), "Hungarian should only appear if feature is enabled"),
            "rus" => assert!(cfg!(feature = "translations-rus"), "Russian should only appear if feature is enabled"),
            "slv" => assert!(cfg!(feature = "translations-slv"), "Slovenian should only appear if feature is enabled"),
            "spa" => assert!(cfg!(feature = "translations-spa"), "Spanish should only appear if feature is enabled"),
            "swe" => assert!(cfg!(feature = "translations-swe"), "Swedish should only appear if feature is enabled"),
            _ => panic!("Unexpected language: {}", lang),
        }
    }
}