/*******************************************************************************
* Copyright 2021 Stefan Majewsky <majewsky@gmx.net>
* SPDX-License-Identifier: Apache-2.0
* Refer to the file "LICENSE" for details.
*******************************************************************************/

fn main() {
    let input = "日曜日";
    let count = jmdict::entries()
        .filter(|e| {
            e.kanji_elements().any(|k| k.text == input)
                || e.reading_elements().any(|r| r.text == input)
        })
        .count();
    println!("{} entries for {}", count, input);
}
