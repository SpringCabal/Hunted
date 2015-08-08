function widget:GetInfo()
	return {
		name 	= "Shotgun UI",
		desc	= "Placeholder which sends shotgun firings events to luarules",
		author	= "Google Frog",
		date	= "8 August 2015",
		license	= "GNU GPL, v2 or later",
		layer	= 0,
		enabled = true
	}
end

-------------------------------------------------------------------
-------------------------------------------------------------------

function widget:MousePress(mx, my, button)
	local alt, ctrl, meta, shift = Spring.GetModKeyState()
	if button == 1 and not Spring.IsAboveMiniMap(mx, my) then
		local _, pos = Spring.TraceScreenRay(mx, my, true)
		if pos then
			local x, y, z = pos[1], pos[2], pos[3]
			Spring.SendLuaRulesMsg('shotgun|' .. x .. '|' .. y .. '|' .. z )
			local mx, my = Spring.GetMouseState()
			Spring.WarpMouse(mx + 30, my + 30)
			return true
		end
	end	
end