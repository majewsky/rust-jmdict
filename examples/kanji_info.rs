/*******************************************************************************
* SPDX-License-Identifier: Apache-2.0 Refer to the file "LICENSE" for details.
* example illustrating how to access kanji and reading information for a word
*******************************************************************************/
use jmdict::{KanjiElement, ReadingElement};

fn main() {
    let input = "一日";
    let count = jmdict::entries()
        .filter(|e| {
            let mut result = false;
            if e.kanji_elements().any(|k| k.text == input) {
                result = true;
                println!("--- entry #{} ---", e.number);
                let kanji_elements: Vec<KanjiElement> = e.kanji_elements().map(|ke| ke).collect();
                for kanji in &kanji_elements {
                    println!("kanji element: {}", kanji.text);
                    println!("   priority: {:?}\n", kanji.priority);
                }

                let reading_forms: Vec<ReadingElement> = e.reading_elements().map(|item| item).collect();
                for reading in reading_forms {
                    println!("reading_form: {}", reading.text);
                    println!("   priority: {:?}\n", reading.priority);

                    for info in reading.infos() {
                        println!("info: {:?}", info);
                    }
                }


                // let sense = e.senses().next().unwrap().text;

            }
            result
        })
        .count();
    println!("{} entries for {}", count, input);
}
