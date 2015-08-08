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
local lastX, lastY

function widget:Update(dt)
	local mx, my = Spring.GetMouseState()
	if mx ~= lastX or my ~= lastY then
		lastX, lastY = mx, my
		local _, pos = Spring.TraceScreenRay(mx, my, true)
		if pos then
			local x, y, z = pos[1], pos[2], pos[3]
			Spring.SendLuaRulesMsg('movegun|' .. x .. '|' .. y .. '|' .. z )
		end
	end
end

function widget:MousePress(mx, my, button)
	local alt, ctrl, meta, shift = Spring.GetModKeyState()
	if button == 1 and not Spring.IsAboveMiniMap(mx, my) then
		local _, pos = Spring.TraceScreenRay(mx, my, true)
		if pos then
			local x, y, z = pos[1], pos[2], pos[3]
			Spring.SendLuaRulesMsg('shotgun|' .. x .. '|' .. y .. '|' .. z )
			return true
		end
	end	
end