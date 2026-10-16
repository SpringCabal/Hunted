-- Core-WASM owns the Hunted gameplay and unit-script callins.  The base
-- gadget loader remains active for the other content-side gadgets.
return VFS.Include("luagadgets/gadgets.lua", nil, VFS.BASE)
