#[test]
fn test_sense_count() {
    use jmdict::{self, Enum};
    
    // Use entry 1001820 (お金 - money) which has translations in all languages
    let test_entry_num = 1001820;
    
    let entry = jmdict::entries()
        .find(|e| e.number == test_entry_num)
        .expect("Entry 1001820 (お金) should exist in all configurations");
    
    println!("Testing entry {}", entry.number);
    
    let mut sense_count = 0;
    let mut total_gloss_count = 0;
    
    for sense in entry.senses() {
        sense_count += 1;
        let mut gloss_count = 0;
        let mut languages = Vec::new();
        
        for gloss in sense.glosses() {
            gloss_count += 1;
            total_gloss_count += 1;
            languages.push(gloss.language.code());
            println!("  Sense {}: {} ({})", sense_count, gloss.text, gloss.language.code());
        }
        
        println!("  Sense {} has {} glosses in languages: {:?}", sense_count, gloss_count, languages);
    }
    
    println!("\nTotal senses: {}", sense_count);
    println!("Total glosses: {}", total_gloss_count);
    
    // The entry should have only 1 sense, not multiple senses for different languages
    assert_eq!(sense_count, 1, "Expected 1 sense, found {}", sense_count);
}