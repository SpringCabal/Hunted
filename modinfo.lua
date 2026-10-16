local modinfo = {
	name = "Hunted WASM",
	shortname = "HHW",
	game = "Hunted WASM",
	shortgame = "HHW",
	version = "$VERSION",
	description = "Hunted LD33 prep game",
	modtype = "1",
	-- Single-player only: without this the engine binds a UDP socket and warns
	-- that nobody else can reach it. Nothing here is ever hosted or joined.
	onlyLocal = true,
	depend = {
		"cursors.sdz",
	}
}

return modinfo
