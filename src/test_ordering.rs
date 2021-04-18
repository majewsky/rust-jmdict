/*******************************************************************************
* Copyright 2021 Stefan Majewsky <majewsky@gmx.net>
* SPDX-License-Identifier: Apache-2.0
* Refer to the file "LICENSE" for details.
*******************************************************************************/

use crate::entries;

#[test]
fn test_entry_order() {
    let mut prev = 0;
    for entry in entries() {
        assert!(entry.number > prev, "{} comes after {}", entry.number, prev);
        prev = entry.number;
    }
}
