#!/usr/bin/env bash
set -eu -o pipefail

git init
rm -Rf .git/hooks

function baseline() {
  local ours=$DIR/${1:?1: our file}.blob;
  local base=$DIR/${2:?2: base file}.blob;
  local theirs=$DIR/${3:?3: their file}.blob;
  local output=$DIR/${4:?4: the name of the output file}.merged;

  shift 4
  git merge-file --stdout "$@" "$ours" "$base" "$theirs" > "$output" || true

  echo "$ours" "$base" "$theirs" "$output" "$@" >> baseline.cases
}

mkdir simple
(cd simple
  echo -e "line1-changed-by-both\nline2-to-be-changed-in-incoming" > ours.blob
  echo -e "line1-to-be-changed-by-both\nline2-to-be-changed-in-incoming" > base.blob
  echo -e "line1-changed-by-both\nline2-changed" > theirs.blob
)

# one big change includes multiple smaller ones
mkdir multi-change
(cd multi-change
  cat <<EOF > base.blob
0
1
2
3
4
5
6
7
8
9
EOF

  cat <<EOF > ours.blob
0
1
X
X
4
5
Y
Y
8
Z
EOF

  cat <<EOF > theirs.blob
T
T
T
T
T
T
T
T
T
T
EOF
)

# a change with deletion/clearing our file
mkdir clear-ours
(cd clear-ours
  cat <<EOF > base.blob
0
1
2
3
4
5
EOF

  touch ours.blob

  cat <<EOF > theirs.blob
T
T
T
T
T
EOF
)

# a change with deletion/clearing their file
mkdir clear-theirs
(cd clear-theirs
  cat <<EOF > base.blob
0
1
2
3
4
5
EOF

  cat <<EOF > ours.blob
O
O
O
O
O
EOF

  touch theirs.blob
)

# differently sized changes
mkdir ours-2-lines-theirs-1-line
(cd ours-2-lines-theirs-1-line
  cat <<EOF > base.blob
0
1
2
3
4
5
EOF

  cat <<EOF > ours.blob
0
1
X
X
4
5
EOF

  cat <<EOF > theirs.blob
0
1
Y
3
4
5
EOF
)

# partial match
mkdir partial-match
(cd partial-match
  cat <<EOF > base.blob
0
1
2
3
4
5
EOF

  cat <<EOF > ours.blob
0
X1
X2
X3
X4
5
EOF

  cat <<EOF > theirs.blob
0
X1
2
X3
X4
5
EOF
)

# based on 'unique merge base' from 'diff3-conflict-markers'
mkdir unique-merge-base-with-insertion
(cd unique-merge-base-with-insertion
  cat <<EOF > base.blob
1
2
3
4
5
EOF

  # no trailing newline
  echo -n $'1\n2\n3\n4\n5\n7' > ours.blob
  echo -n $'1\n2\n3\n4\n5\nsix' > theirs.blob
)

for dir in  simple \
            multi-change \
            clear-ours \
            clear-theirs \
            ours-2-lines-theirs-1-line \
            partial-match \
            unique-merge-base-with-insertion; do
  DIR=$dir
  baseline ours base theirs merge
  baseline ours base theirs diff3 --diff3
  baseline ours base theirs zdiff3 --zdiff3
  baseline ours base theirs merge-ours --ours
  baseline ours base theirs merge-theirs --theirs
  baseline ours base theirs merge-union --union
done