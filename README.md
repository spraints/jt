# Journal time!

Track what I do each week in a Git repository that `jt` maintains for me. It's
easy to edit and version the files on my own, but `jt` makes it easier.

## TODO:

- `jt today` - open `$EDITOR` for the current week.
- `jt $APP recent` - list recent APP (slack, github, gdrive) activity
  (mentions, messages, particular rooms) in a UI, jump from activity item to
  journal.
- `jt $APP config` - edit things like list of interesting github repos,
  interesting slack channels, etc.
- `jt view` - open a UI listing recent journals
- `jt sync -n|-y` - make it so that the journal is completely committed and
  synced with the remote.
- After editing, sync.

If I were doing this in Go, I would use bubbletea. Maybe there's something similar for Rust?
- https://www.reddit.com/r/rust/comments/pxhl4d/cli_ui_library_for_rust/
- https://www.google.com/search?q=bubbletea+for+rust