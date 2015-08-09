local base = piece "base"

local color = {0.69, 0.61, 0.85, 0.3}
local lightRange = 450
local lightRadius = 140
local rotationSpeed = 2*math.pi/(300)
 
local ux, uz, lightAngle, flashlightID

local function GetLightCoordinates()
	return ux + lightRange*math.cos(lightAngle), uz + lightRange*math.sin(lightAngle)
end

local function LightSpin()
	
	while true do
		lightAngle = lightAngle + rotationSpeed
		
		local x,z = GetLightCoordinates()
		Turn(base, y_axis, -lightAngle + math.pi/2, rotationSpeed*30)
	
		Spring.SetUnitRulesParam(unitID, "lighthouse_x", x)
		Spring.SetUnitRulesParam(unitID, "lighthouse_z", z)
		
		Sleep(33)
	end
end

function script.Create()
	lightAngle = math.random()*2*math.pi
	ux,_,uz = Spring.GetUnitPosition(unitID)
	
	Move(base, y_axis, 10)
	Turn(base, x_axis, math.pi*0.5)
	Turn(base, y_axis, -lightAngle + math.pi/2)
	local x,z = GetLightCoordinates()
	
	Spring.SetUnitRulesParam(unitID, "lighthouse_x", x)
	Spring.SetUnitRulesParam(unitID, "lighthouse_z", z)
	Spring.SetUnitRulesParam(unitID, "lighthouse_size", lightRadius)
	Spring.SetUnitRulesParam(unitID, "lighthouse_color", 1)
	
	StartThread(LightSpin)
end

function script.Killed(recentDamage, maxHealth)
	return 0
end