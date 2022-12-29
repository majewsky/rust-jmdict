/*******************************************************************************
* SPDX-License-Identifier: Apache-2.0 Refer to the file "LICENSE" for details.
* example illustrating how to access kanji and reading information for a word
*******************************************************************************/

fn main() {
    let input = "一日";
    let count = jmdict::entries()
        .filter(|e| {
            if e.kanji_elements().any(|k| k.text == input) {
                // note two entries have identical Kanji and reading
                // yet differ in "sense" not shown in this example
                println!("--- entry #{} ---", e.number);
                for kanji in e.kanji_elements() {
                    println!("kanji element: {}", kanji.text);
                    println!("   priority: {:?}\n", kanji.priority);
                }

                for reading in e.reading_elements() {
                    println!("reading_form: {}", reading.text);
                    println!("   priority: {:?}\n", reading.priority);

                    for info in reading.infos() {
                        println!("info: {:?}", info);
                    }
                }

                return true
            }
            false
        })
        .count();
    println!("{} entries for {}", count, input);
}
