# If you want to list all the make command targets with a description execute the command:
# -----------------------------------------------------
#  make help
# -----------------------------------------------------

# Detect OS, and set OS-specific variables
ifeq ($(OS),Windows_NT)
	FIND_CMD := powershell -Command "Get-ChildItem -Recurse"
	GREP_CMD := findstr
	SED_CMD := powershell -Command "(Get-Content -Raw)"
	SEP := \\
	SHELL_CMD := cmd /c
	ifeq ($(SHELL_CMD), powershell)
	    GREEN := "`e[32m"
	    YELLOW := "`e[33m"
	    RESET := "`e[0m"
	endif
else
	FIND_CMD := find
	GREP_CMD := grep
	SED_CMD := sed
	SEP := /
	SHELL_CMD := sh -c
	STR_FIND := awk
    GREEN  := $(shell tput -Txterm setaf 2)
    YELLOW := $(shell tput -Txterm setaf 3)
    RESET  := $(shell tput -Txterm sgr0)
endif


.PHONY: help
## List available make targets with descriptions extracted from comments in this Makefile
help:
	@echo ''
	@echo 'Usage:'
	@echo '  ${YELLOW}make${RESET} ${GREEN}<target>${RESET}'
	@echo ''
	@echo 'Targets:'
	@$(STR_FIND) "/^[a-zA-Z0-9_\/-]+:/ { \
		helpCommand = substr(\$$1, 1, index(\$$1, \":\")-1); \
		if (NR > 1) { \
			printf \"  $(YELLOW)%-40s $(GREEN)%s$(RESET)\n\", helpCommand, helpMessage; \
		} \
		helpMessage=\"\"; \
	} \
	/^##/ { \
		if (helpMessage != \"\") { \
			helpMessage = helpMessage \"\\n\\t\\t\\t\\t\\t\\t- $(GREEN)\" substr(\$$0, 4); \
		} else { \
			helpMessage = substr(\$$0, 4); \
		} \
	} \
	END { \
		if (helpMessage != \"\") { \
			printf \"  $(YELLOW)%-60s $(GREEN)%s$(RESET)\n\", helpCommand, helpMessage; \
		} \
	}" Makefile

.PHONY: setup-local
## Setup local environment
setup-local:
	bash "$(shell pwd)/scripts/setup-local.sh"