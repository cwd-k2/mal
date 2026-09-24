local repository_root = assert(
  vim.g.mal_repository_root,
  "mal repository root is not configured by .nvim.lua"
)

return {
  cmd = { repository_root .. "/target/release/mal-lsp" },
  filetypes = { "mal" },
  root_markers = { ".git" },
}
