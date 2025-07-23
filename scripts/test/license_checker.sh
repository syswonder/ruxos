#!/bin/bash

copyright_line="Copyright (c)"
exit_code=0

# Use git ls-files to find all Git-tracked .rs files
# --cached: Only show files in the index (i.e., Git-tracked files)
# --exclude-standard: Respect standard exclusion patterns (.gitignore, .git/info/exclude, etc.)
# -- *.rs: Limit the search to .rs files only
while IFS= read -r file; do
  # Ensure the file exists and is a regular file (git ls-files usually returns correct file paths)
  if [[ -f "$file" ]]; then
    if ! grep -qF "$copyright_line" "$file"; then
      echo "Error: File does not contain the required copyright line: $file"
      exit_code=1
    fi
  else
    # Theoretically, git ls-files should not return non-existent files, but kept for robustness
    echo "Warning: git ls-files returned a non-existent file path: $file"
  fi
done < <(git ls-files --cached --exclude-standard -- "*.rs" "*.c" "*.h")

echo "Script finished with exit code: $exit_code"
exit "$exit_code"
