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
local lastx, lasty, lastz

function widget:GameFrame(n)
	local mx, my = Spring.GetMouseState()
	local _, pos = Spring.TraceScreenRay(mx, my, true)
	if pos then
		local x, y, z = pos[1], pos[2], pos[3]
		if x ~= lastx or y ~= lasty or z ~= lastz then
			lastx, lasty, lastz = x, y, z
			Spring.SendLuaRulesMsg('movegun|' .. x .. '|' .. y .. '|' .. z )
		end
	end
end

function widget:MousePress(mx, my, button)
	local alt, ctrl, meta, shift = Spring.GetModKeyState()
	if button == 1 and not Spring.IsAboveMiniMap(mx, my) and ctrl then
		local _, pos = Spring.TraceScreenRay(mx, my, true)
		if pos then
			local x, y, z = pos[1], pos[2], pos[3]
			Spring.SendLuaRulesMsg('shotgun|' .. x .. '|' .. y .. '|' .. z )
			return true
		end
	end	
end