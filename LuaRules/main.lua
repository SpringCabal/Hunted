-- WASM owns gameplay. Lua loads the unchanged unit scripts and their adapter.
return VFS.Include("luagadgets/gadgets.lua", nil, VFS.BASE)
