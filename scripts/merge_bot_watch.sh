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
agent_sandbox="${MERGE_BOT_SANDBOX:-workspace-write}"
agent_approval="${MERGE_BOT_APPROVAL:-never}"
herdr_workspace="${MERGE_BOT_HERDR_WORKSPACE:-${HERDR_WORKSPACE_ID:-}}"
herdr_tab_prefix="${MERGE_BOT_HERDR_TAB_PREFIX:-merge-bot}"
codex_bin="${MERGE_BOT_CODEX_BIN:-codex}"

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

shell_quote() {
  printf '%q' "$1"
}

sanitize_name() {
  local raw="$1"
  local sanitized

  sanitized="$(printf '%s' "$raw" | tr -c '[:alnum:]_-' '_')"
  sanitized="${sanitized##_}"
  sanitized="${sanitized%%_}"
  printf '%s' "${sanitized:-merge_bot}"
}

run_codex_in_herdr() {
  local branch="$1"
  local prompt_file="$2"
  local log_file message_file branch_slug tab_label
  local effort_config output_file output_safe exit_file exit_safe message_safe
  local status_file status_safe child_cmd tab_json tab_id agent_json agent_target
  local tab_create_cmd agent_start_cmd close_cmd waited_seconds

  if ! command -v herdr >/dev/null 2>&1; then
    echo "merge-bot: herdr is required when MERGE_BOT_COMMAND is unset" >&2
    return 1
  fi

  if ! command -v "$codex_bin" >/dev/null 2>&1; then
    echo "merge-bot: codex binary '$codex_bin' was not found" >&2
    return 1
  fi

  if [[ -z "$herdr_workspace" ]]; then
    echo "merge-bot: no active herdr workspace found; set MERGE_BOT_HERDR_WORKSPACE or run inside herdr" >&2
    return 1
  fi

  branch_slug="$(sanitize_name "$branch")"
  tab_label="${herdr_tab_prefix}-${branch_slug}"
  log_file="$state_dir/${branch//\//_}.log"
  message_file="$state_dir/${branch//\//_}.last-message.txt"
  output_file="$state_dir/${branch//\//_}.codex.stdout"
  exit_file="$state_dir/${branch//\//_}.exit"
  status_file="$state_dir/${branch//\//_}.status"
  effort_config="model_reasoning_effort=\"$agent_effort\""

  output_safe="$(shell_quote "$output_file")"
  exit_safe="$(shell_quote "$exit_file")"
  message_safe="$(shell_quote "$message_file")"
  status_safe="$(shell_quote "$status_file")"
  rm -f "$output_file" "$exit_file" "$status_file"

  tab_create_cmd=(
    herdr tab create
    --workspace "$herdr_workspace"
    --cwd "$worktree_root"
    --label "$tab_label"
    --no-focus
  )
  tab_json="$("${tab_create_cmd[@]}")"
  tab_id="$(printf '%s' "$tab_json" | sed -n 's/.*"tab_id":"\([^"]*\)".*/\1/p' | head -n 1)"

  if [[ -z "$tab_id" ]]; then
    echo "merge-bot: failed to create herdr tab for $branch" >&2
    return 1
  fi

  child_cmd="$codex_bin exec --cd $(shell_quote "$worktree_root") --model $(shell_quote "$agent_model") --sandbox $(shell_quote "$agent_sandbox") --ask-for-approval $(shell_quote "$agent_approval") --color never --output-last-message $message_safe -c $(shell_quote "$effort_config") - < $(shell_quote "$prompt_file") > $output_safe 2>&1; status=\$?; printf '%s\n' \"\$status\" > $exit_safe; exit \$status"

  agent_start_cmd=(
    herdr agent start "$tab_label"
    --workspace "$herdr_workspace"
    --tab "$tab_id"
    --cwd "$worktree_root"
    --no-focus
    --
    /bin/zsh -lc "$child_cmd"
  )
  agent_json="$("${agent_start_cmd[@]}")"
  agent_target="$(printf '%s' "$agent_json" | sed -n 's/.*"pane_id":"\([^"]*\)".*/\1/p' | head -n 1)"
  if [[ -z "$agent_target" ]]; then
    agent_target="$(printf '%s' "$agent_json" | sed -n 's/.*"agent_id":"\([^"]*\)".*/\1/p' | head -n 1)"
  fi

  if [[ -z "$agent_target" ]]; then
    herdr tab close "$tab_id" >/dev/null 2>&1 || true
    echo "merge-bot: failed to start herdr agent for $branch" >&2
    return 1
  fi

  waited_seconds=0
  while [[ ! -f "$exit_file" ]]; do
    sleep 1
    waited_seconds=$((waited_seconds + 1))
    if (( waited_seconds >= 3600 )); then
      herdr tab close "$tab_id" >/dev/null 2>&1 || true
      echo "merge-bot: herdr agent timed out for $branch" >&2
      return 1
    fi
  done

  close_cmd=(herdr tab close "$tab_id")
  "${close_cmd[@]}" >/dev/null

  if [[ -f "$output_file" ]]; then
    mv "$output_file" "$log_file"
  fi

  if [[ ! -f "$exit_file" ]]; then
    echo "merge-bot: missing herdr agent exit file for $branch" >&2
    return 1
  fi

  printf '%s\n' "$(cat "$exit_file")" > "$status_file"

  if [[ "$(cat "$exit_file")" != "0" ]]; then
    echo "merge-bot: codex run failed for $branch; see $log_file" >&2
    return 1
  fi

  return 0
}

run_agent() {
  local branch="$1"
  local prompt_file="$2"

  if [[ -z "$agent_command" ]]; then
    run_codex_in_herdr "$branch" "$prompt_file"
    return $?
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
    run_agent "$branch" "$prompt_file"
    printf '%s\t%s\n' "$branch" "$sha" >> "$next_state"
  done < "$current_state"

  mv "$next_state" "$state_file"

  rm -f "$current_state"
  sleep "$poll_seconds"
done
