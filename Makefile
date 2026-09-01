GITIGNORE_PARTS := $(sort $(wildcard .gitignore.d/*.gitignore))

.PHONY: check-gitignore FORCE

FORCE:

.gitignore: FORCE $(GITIGNORE_PARTS)
	@printf '%s\n\n' '# This file is generated from .gitignore.d/*.gitignore.' > $@
	@first=1; for part in $(GITIGNORE_PARTS); do \
		if [ "$$first" -eq 0 ]; then printf '\n'; fi; \
		cat "$$part"; \
		first=0; \
	done >> $@

check-gitignore: .gitignore
	@git ls-files --error-unmatch .gitignore > /dev/null
	@if ! git diff --quiet -- .gitignore; then \
		printf '%s\033[1;31m%s\033[0m\n' 'The generated .gitignore has changed. Run: ' 'git add .gitignore'; \
		exit 1; \
	fi
	@printf '%s\n' 'The generated .gitignore is up to date.'
