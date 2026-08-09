#!/usr/bin/env bash

set -euo pipefail

if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  echo "merge-bot: must be run inside a git repository" >&2
  exit 1
fi

target_branch="${MERGE_BOT_TARGET_BRANCH:-integration}"
poll_seconds="${MERGE_BOT_POLL_SECONDS:-5}"
watch_patterns_raw="${MERGE_BOT_WATCH_PATTERNS:-frontend action controller domain}"
worktree_root="$(git rev-parse --show-toplevel)"
state_dir="${MERGE_BOT_STATE_DIR:-$worktree_root/.merge-bot}"
state_file="$state_dir/branches.tsv"
agent_doc="${MERGE_BOT_AGENT_DOC:-docs/merge-bot-agent.md}"
agent_name="${MERGE_BOT_AGENT:-codex}"
agent_model="${MERGE_BOT_MODEL:-gpt-5.4-mini}"
agent_effort="${MERGE_BOT_EFFORT:-medium}"
agent_command="${MERGE_BOT_COMMAND:-}"

mkdir -p "$state_dir"
touch "$state_file"
load_branches() {
  git for-each-ref --format='%(refname:short)	%(objectname)' refs/heads
}

get_previous_sha() {
  local branch="$1"
  awk -F '\t' -v branch="$branch" '$1 == branch { print $2; exit }' "$state_file"
}

is_watched_branch() {
  local branch="$1"
  local pattern
  for pattern in $watch_patterns_raw; do
    if [[ "$branch" == $pattern ]]; then
      return 0
    fi
  done
  return 1
}

build_prompt() {
  local branch="$1"
  local old_sha="$2"
  local new_sha="$3"
  local diff_stat diff_summary

  diff_stat="$(git diff --stat "$target_branch...$branch" || true)"
  diff_summary="$(git log --oneline --decorate=short --no-merges --max-count=8 "$target_branch..$branch" || true)"

  cat <<EOF
Merge request context

Repository: $(pwd)
Agent spec: $agent_doc
Agent: $agent_name
Model: $agent_model
Reasoning effort: $agent_effort
Target branch: $target_branch
Source branch: $branch
Previous commit: ${old_sha:-unknown}
Current commit: $new_sha

What changed:
$diff_stat

Recent commits:
$diff_summary

Task:
- Decide whether this branch should be merged into $target_branch.
- Resolve any merge conflicts only if the resolution is obvious from the code
  and existing project conventions.
- Run the repository's relevant checks if you can do so from the current
  environment.
- Leave the repository in a clean state and report the final merge status.
EOF
}

run_agent() {
  local prompt_file="$1"

  if [[ -z "$agent_command" ]]; then
    echo "merge-bot: no MERGE_BOT_COMMAND configured; prompt written to $prompt_file" >&2
    cat "$prompt_file"
    return 0
  fi

  bash -lc "$agent_command" < "$prompt_file"
}

while true; do
  current_state="$(mktemp)"
  load_branches > "$current_state"
  next_state="$(mktemp)"

  while IFS=$'\t' read -r branch sha; do
    [[ -n "${branch:-}" && -n "${sha:-}" ]] || continue
    if ! is_watched_branch "$branch"; then
      printf '%s\t%s\n' "$branch" "$sha" >> "$next_state"
      continue
    fi

    old_sha="$(get_previous_sha "$branch")"
    if [[ "$old_sha" == "$sha" ]]; then
      printf '%s\t%s\n' "$branch" "$sha" >> "$next_state"
      continue
    fi

    prompt_file="$state_dir/${branch//\//_}.prompt"
    build_prompt "$branch" "$old_sha" "$sha" > "$prompt_file"
    echo "merge-bot: branch update detected on $branch" >&2
    run_agent "$prompt_file"
    printf '%s\t%s\n' "$branch" "$sha" >> "$next_state"
  done < "$current_state"

  mv "$next_state" "$state_file"

  rm -f "$current_state"
  sleep "$poll_seconds"
done
