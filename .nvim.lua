local source = debug.getinfo(1, "S").source:sub(2)
local repository_root = vim.fs.dirname(vim.fs.normalize(vim.fn.fnamemodify(source, ":p")))

vim.g.mal_repository_root = repository_root
vim.opt.runtimepath:prepend(repository_root .. "/.nvim")
vim.opt.runtimepath:prepend(repository_root .. "/.artifacts/editor-runtime/neovim")

vim.filetype.add({ extension = { mal = "mal" } })

vim.api.nvim_create_autocmd("FileType", {
  pattern = "mal",
  callback = function(args)
    vim.bo[args.buf].commentstring = "// %s"
    vim.bo[args.buf].expandtab = true
    vim.bo[args.buf].shiftwidth = 4
    vim.bo[args.buf].softtabstop = 4
    vim.treesitter.start(args.buf, "mal")
  end,
})

vim.lsp.enable("mal")
