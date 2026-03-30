# zsh-highlighter — minimal syntax highlighting via a compiled Rust binary

# Locate binary: adjacent bin/ directory first, then PATH
_zh_bin="${0:A:h}/../bin/zsh-highlighter"
if [[ ! -x "$_zh_bin" ]]; then
  _zh_bin="${commands[zsh-highlighter]:-}"
  if [[ -z "$_zh_bin" ]]; then
    unset _zh_bin
    return 0
  fi
fi

# Build and export the known-commands list
_zh_rebuild_cmds() {
  local -a _cmds
  _cmds=( ${(k)commands} ${(k)builtins} ${(k)aliases} ${(k)functions} )
  export _ZH_CMDS="${(pj:\n:)_cmds}"
}
_zh_rebuild_cmds

# Core highlighting hook — called on each line-pre-redraw
_zh_highlight() {
  [[ -z "$BUFFER" ]] && return 0
  [[ "$BUFFER" = "$_zh_prev_buffer" ]] && return 0
  (( ${#BUFFER} > 10000 )) && return 0
  _zh_prev_buffer="$BUFFER"
  local _zh_output
  _zh_output="$("$_zh_bin" "$BUFFER" 2>/dev/null)"
  if (( $? != 0 )); then
    region_highlight=()
    return 0
  fi
  region_highlight=( "${(@f)_zh_output}" )
}

autoload -Uz add-zle-hook-widget
add-zle-hook-widget line-pre-redraw _zh_highlight

# Refresh command cache after installing new tools
zh-reload() {
  rehash
  _zh_rebuild_cmds
  unset _zh_prev_buffer
  echo 'zsh-highlighter: command cache reloaded'
}

# Rebuild command cache on directory change
_zh_chpwd() { _zh_rebuild_cmds; unset _zh_prev_buffer; }
autoload -Uz add-zsh-hook
add-zsh-hook chpwd _zh_chpwd
