default:
	@printf '%s\n' '>> Usage:' '      make import JMDICT_PATH=/path/to/jmdict' '      make export' '>> Refer to README.md for details.'

import:
ifeq ($(origin JMDICT_PATH),undefined)
	@echo "ERROR: Run as \`make import JMDICT_PATH=/path/to/JMdict\`".
	@false
endif
	go run preprocess-jmdict.go $(JMDICT_PATH)

EXPORT_FILENAME ?= entrypack-v1-$(shell cat entrypack.json | grep -o 'Creation Date: [0-9-]*' | awk '{print$$3}').json.gz

export:
	gzip -9 < entrypack.json > $(EXPORT_FILENAME)

.PHONY: default import export
