function widget:GetInfo()
	return {
		name    = 'Structue Placer',
		desc    = 'Forces the game to start and sets globallos on',
		author  = 'GoogleFrog',
		date    = '9 August, 2015',
		license = 'GNU GPL v2',
        layer = 0,
		enabled = true,
	}
end

--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

local placeLocation
local placeDefID

function widget:MousePress(mx, my, button)
	local alt, ctrl, meta, shift = Spring.GetModKeyState()
	if button == 3 and not Spring.IsAboveMiniMap(mx, my) then
		local _, pos = Spring.TraceScreenRay(mx, my, true)
		if pos then
			placeDefID = UnitDefNames["mine"].id
			
			local x, y, z = pos[1], pos[2], pos[3]
			Spring.SendLuaRulesMsg('placeStructure|' .. placeDefID .. "|" .. x .. '|' .. y .. '|' .. z )
			return true
		end
	end	
end

function widget:Update()
	local alt, ctrl, meta, shift = Spring.GetModKeyState()
	local mx, my = Spring.GetMouseState()
	if ctrl and not Spring.IsAboveMiniMap(mx, my) then
		local _, pos = Spring.TraceScreenRay(mx, my, true)
		if pos then
			placeLocation = pos
			placeDefID = UnitDefNames["mine"].id
			return
		end
	end
	placeLocation = false
end

--------------------------------------------------------------------------------
--------------------------------------------------------------------------------
-- Drawing

local function DrawUnitDef(unitDefID, teamID, ux, uy, uz)
	gl.Color(1.0, 1.0, 1.0, 1.0)
	gl.DepthTest(GL.LEQUAL)
	gl.DepthMask(true)
	gl.Lighting(true)

	gl.PushMatrix()
		gl.Translate(ux, uy, uz)
		gl.UnitShape(unitDefID, teamID)
	gl.PopMatrix()

	gl.Lighting(false)
	gl.DepthTest(false)
	gl.DepthMask(false)
end

local function DrawWorldFunc()
	if placeLocation and placeDefID then
		DrawUnitDef(placeDefID, 0, placeLocation[1], placeLocation[2], placeLocation[3])
	end
end

function widget:DrawWorld()
	DrawWorldFunc()
end

function widget:DrawWorldRefraction()
	DrawWorldFunc()
end