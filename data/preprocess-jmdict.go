/*******************************************************************************
* Copyright 2021 Stefan Majewsky <majewsky@gmx.net>
* SPDX-License-Identifier: Apache-2.0
* Refer to the file "LICENSE" for details.
*******************************************************************************/

package main

import (
	"bufio"
	"bytes"
	"encoding/json"
	"encoding/xml"
	"fmt"
	"io/ioutil"
	"os"
	"regexp"
	"strings"
)

func main() {
	if len(os.Args) != 2 {
		fmt.Fprintf(os.Stderr, "usage: %s <path-to-JMdict>\n", os.Args[0])
		os.Exit(1)
	}

	//open input file for line-wise reading
	file, err := os.Open(os.Args[1])
	must(err)
	fileBuffered := bufio.NewReaderSize(file, 65536)
	nextLine := func() string {
		line, err := fileBuffered.ReadString('\n')
		must(err)
		return strings.TrimSpace(line)
	}

	processOpening(nextLine)
	processEntries(nextLine)
}

func must(err error) {
	if err != nil {
		panic(err.Error())
	}
}

////////////////////////////////////////////////////////////////////////////////
// process opening (everything until <JMdict>)

var (
	entityHeaderRx = regexp.MustCompile(`^<!-- <(\S+)> .*entities -->$`)
	entityDefRx    = regexp.MustCompile(`^<!ENTITY (\S+) "(.+)">$`)
)

func processOpening(nextLine func() string) {
	var (
		sets       = make(map[string]map[string]string)
		currentSet = ""
	)

	for {
		line := nextLine()

		//This loop sees all the lines of the DTD up to the opener of the actual
		//document contents.
		if line == "<JMdict>" {
			break
		}

		//Start a new entity set when encountering its header comment.
		match := entityHeaderRx.FindStringSubmatch(line)
		if match != nil {
			currentSet = match[1]
			sets[currentSet] = make(map[string]string)
		}

		//When inside an entity set, add all subsequent entities to the set.
		match = entityDefRx.FindStringSubmatch(line)
		if match != nil {
			key, value := match[1], match[2]
			if currentSet == "" {
				panic("entity definition outside of set: " + line)
			}
			sets[currentSet][key] = value
			//the XML decoder also needs to know about these entities, but we have it
			//expand the entity into the XML representation of the entity (e.g.
			//"&arch;" expands into "arch" rather than "archaism")
			decoderEntities[key] = key
		}
	}

	//dump collected data
	buf, err := json.Marshal(sets)
	must(err)
	var indented bytes.Buffer
	must(json.Indent(&indented, buf, "", "\t"))
	must(ioutil.WriteFile("../jmdict-enums/data/entities.json", indented.Bytes(), 0666))
}

////////////////////////////////////////////////////////////////////////////////
// process contents (everything between <JMdict> and </JMdict>)

func processEntries(nextLine func() string) {
	outputFile, err := os.Create("entrypack.json")
	must(err)
	defer outputFile.Close()

	buf := ""
	for {
		line := nextLine()

		//This loop ends when we encounter the end of the file.
		if line == "</JMdict>" {
			if buf != "" {
				//we should have had </entry> just before and thus have an empty buffer
				panic("reached </JMdict> with non-empty buffer: " + buf)
			}
			break
		}

		//Collect lines until we have a full entry to process.
		buf += line
		if line == "</entry>" {
			_, err := outputFile.Write([]byte(processEntry(buf)))
			must(err)
			buf = ""
		}
	}
}

////////////////////////////////////////////////////////////////////////////////
// convert individual entries from XML to JSON
//
// NOTE: In the JSON, common fields have single-letter keys because that
// actually saves several MiB. Going from 90 MiB to 75 MiB is quite
// significant since it gives us more headroom before running into GitHub's hard
// limit of 100 MiB per object.

type dictEntry struct {
	SeqNo uint64      `xml:"ent_seq" json:"n"`
	KEle  []dictKEle  `xml:"k_ele" json:"K,omitempty"`
	REle  []dictREle  `xml:"r_ele" json:"R"`
	Sense []dictSense `xml:"sense" json:"S"`
}

type dictKEle struct {
	Keb   string   `xml:"keb" json:"t"`
	KeInf []string `xml:"ke_inf" json:"i,omitempty"`
	KePri []string `xml:"ke_pri" json:"p,omitempty"`
}

type dictREle struct {
	Reb       string         `xml:"reb" json:"t"`
	ReNokanji boolByPresence `xml:"re_nokanji" json:"n,omitempty"`
	ReRestr   []string       `xml:"re_restr" json:"r,omitempty"`
	ReInf     []string       `xml:"re_inf" json:"i,omitempty"`
	RePri     []string       `xml:"re_pri" json:"p,omitempty"`
}

type dictSense struct {
	Stagk   []string      `xml:"stagk" json:"stagk,omitempty"`
	Stagr   []string      `xml:"stagr" json:"stagr,omitempty"`
	Pos     []string      `xml:"pos" json:"p,omitempty"`
	Xref    []string      `xml:"xref" json:"xref,omitempty"`
	Ant     []string      `xml:"ant" json:"ant,omitempty"`
	Field   []string      `xml:"field" json:"f,omitempty"`
	Misc    []string      `xml:"misc" json:"m,omitempty"`
	SInf    []string      `xml:"s_inf" json:"i,omitempty"`
	Lsource []dictLsource `xml:"lsource" json:"L,omitempty"`
	Dial    []string      `xml:"dial" json:"dial,omitempty"`
	Gloss   []dictGloss   `xml:"gloss" json:"G,omitempty"`
}

type dictLsource struct {
	Text    string `xml:",chardata" json:"t"`
	Lang    string `xml:"lang,attr" json:"l,omitempty"`
	LsType  string `xml:"ls_type,attr" json:"type,omitempty"`
	LsWasei string `xml:"ls_wasei,attr" json:"wasei,omitempty"`
}

type dictGloss struct {
	Text  string   `xml:",chardata" json:"t"`
	Lang  string   `xml:"lang,attr" json:"l,omitempty"`
	GGend string   `xml:"g_gend,attr" json:"g_gend,omitempty"`
	GType string   `xml:"g_type,attr" json:"g_type,omitempty"`
	Pri   []string `xml:"pri" json:"pri,omitempty"`
	//NOTE: g_gend and <pri> are defined in the DTD, but do not actually occur in any entry.
}

var decoderEntities = make(map[string]string)

func processEntry(xmlStr string) string {
	var e dictEntry
	dec := xml.NewDecoder(strings.NewReader(xmlStr))
	dec.Entity = decoderEntities
	must(dec.Decode(&e))
	jsonBytes, err := json.Marshal(e)
	must(err)
	return string(jsonBytes) + "\n"
}

////////////////////////////////////////////////////////////////////////////////
// helper types for XML decoding

//boolByPresence decodes to true when the corresponding element is present.
type boolByPresence bool

func (b *boolByPresence) UnmarshalXML(d *xml.Decoder, start xml.StartElement) error {
	//This is only called if the element is present. Otherwise, it stays at its default value of false.
	*b = true
	//The xml.Decoder will croak unless we consume the element.
	var foo struct{}
	return d.DecodeElement(&foo, &start)
}
