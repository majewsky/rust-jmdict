/*******************************************************************************
* SPDX-License-Identifier: Apache-2.0 Refer to the file "LICENSE" for details.
* example illustrating how to access kanji and reading information for a word
*******************************************************************************/
use jmdict::GlossLanguage;

fn main() {
    let input = "一日";
    let count = jmdict::entries()
        .filter(|e| {
            if e.kanji_elements().any(|k| k.text == input) {
                println!("\n--- entry #{} ---", e.number);
                for kanji in e.kanji_elements() {
                    println!("{}", kanji.text);
                }

                for reading in e.reading_elements() {
                    print!("{}\t", reading.text);
                    for info in reading.infos() {
                        print!("{:?}, ", info);
                    }
                }
                print!("\n");

                for (index, sense) in e.senses().enumerate() {

                    let parts_of_speech = sense.parts_of_speech().map(|part| format!("{}", part)).collect::<Vec<String>>().join(", ");
                    let english_meaning = sense.glosses()
                        .filter(|g| g.language == GlossLanguage::English)
                        .map(|g| g.text)
                        .collect::<Vec<&str>>().join("; ");
                    println!("{}. {}: {}", index+1, parts_of_speech, english_meaning);

                    for info in sense.topics() {
                        print!("{:?}, ", info);
                    }

                   // println!("sense: {:?}", sense);
                }

                return true
            }
            false
        })
        .count();
    println!("\n\n{} entries for {}", count, input);
}
