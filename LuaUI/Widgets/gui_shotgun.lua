function widget:GetInfo()
	return {
		name 	= "Shotgun UI",
		desc	= "Placeholder which sends shotgun firings events to luarules",
		author	= "Google Frog",
		date	= "8 August 2015",
		license	= "GNU GPL, v2 or later",
		layer	= 20, -- Placed after other click stealing widgets (eg structure placer)
		enabled = true
	}
end

-------------------------------------------------------------------
-------------------------------------------------------------------
local lastx, lasty, lastz
local benchmarkNoInput = tonumber((Spring.GetModOptions() or {}).benchmark_no_input) == 1

function widget:Initialize()
	if benchmarkNoInput then Spring.Echo("[HuntedBenchmark] mouse_input=off") end
end

function widget:GameFrame(n)
	if benchmarkNoInput then return end
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
	if benchmarkNoInput then return false end
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