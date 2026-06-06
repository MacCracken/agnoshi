#!/bin/sh
# version-bump.sh -- bump the VERSION file (single source of truth).
#
# cyrius.cyml reads it via `version = "${file:VERSION}"`, so the
# manifest does not need a separate edit. CI's version-consistency
# gate confirms VERSION matches the git tag at release time.

set -eu

NEW_VERSION="${1:?Usage: $0 <new-version>}"

echo "$NEW_VERSION" > VERSION

# Keep the runtime banner literal in sync with VERSION. The `version` builtin
# and the boot banner read `var VERSION_STR = "agnoshi X.Y.Z"` in src/agnsh.cyr.
# Before this, a VERSION bump did NOT touch that literal, so the banner silently
# desynced from VERSION (the 14115 iron burn showed `agnoshi 1.4.0` while VERSION
# was 1.4.1). The agnos kernel solves the identical problem by regenerating
# version.cyr from VERSION on bump; agnoshi's single literal is a one-line sed.
# The count-guard fails loud if the literal moves or its shape changes, rather
# than silently skipping (which would re-introduce the desync).
AGNSH_CYR="src/agnsh.cyr"
VS_MATCHES=$(grep -c '^var VERSION_STR = "agnoshi ' "$AGNSH_CYR" || true)
if [ "$VS_MATCHES" -ne 1 ]; then
    echo "ERROR: expected exactly 1 'var VERSION_STR = \"agnoshi …\"' in $AGNSH_CYR, found $VS_MATCHES" >&2
    echo "       the banner literal moved/changed shape — version-bump.sh can no longer keep it in sync with VERSION." >&2
    exit 1
fi
sed -i -E "s/^var VERSION_STR = \"agnoshi [0-9]+\\.[0-9]+\\.[0-9]+\";/var VERSION_STR = \"agnoshi $NEW_VERSION\";/" "$AGNSH_CYR"
echo "Synced src/agnsh.cyr VERSION_STR -> \"agnoshi $NEW_VERSION\""

echo "Bumped VERSION to $NEW_VERSION"
echo ""
echo "Next steps:"
echo "  git add VERSION CHANGELOG.md"
echo "  git commit -m 'release: $NEW_VERSION'"
echo "  git tag $NEW_VERSION"
echo "  git push origin main --tags"
