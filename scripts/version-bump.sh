#!/bin/sh
# version-bump.sh -- bump the VERSION file (single source of truth).
#
# cyrius.cyml reads it via `version = "${file:VERSION}"`, so the
# manifest does not need a separate edit. CI's version-consistency
# gate confirms VERSION matches the git tag at release time.

set -eu

NEW_VERSION="${1:?Usage: $0 <new-version>}"

echo "$NEW_VERSION" > VERSION

echo "Bumped VERSION to $NEW_VERSION"
echo ""
echo "Next steps:"
echo "  git add VERSION CHANGELOG.md"
echo "  git commit -m 'release: $NEW_VERSION'"
echo "  git tag $NEW_VERSION"
echo "  git push origin main --tags"
