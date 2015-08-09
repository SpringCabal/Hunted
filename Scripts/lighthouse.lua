local lighthead = piece "LightHead"
local lightpitch = piece "LightPitch"

local color = {0.69, 0.61, 0.85, 0.3}
local lightRange = 300
local lightRadius = 250
local rotationSpeed = 2*math.pi/(250)
 
 
local torchAttributes = {
	radius = 250,
	radiusSq = 250^2,
	edgeMagnitude = 0.1,
	proximityMagnitude = 1.6,
	thingType = 2, -- torch
}
 
local ux, uz, lightAngle, torchScaryArea

local function GetLightCoordinates()
	return ux + lightRange*math.cos(lightAngle), uz + lightRange*math.sin(lightAngle)
end

local function LightSpin()
	
	while true do
		lightAngle = lightAngle + rotationSpeed
		
		local x,z = GetLightCoordinates()
		local y = Spring.GetGroundHeight(x, z)
		Turn(lighthead, y_axis, -lightAngle, rotationSpeed*30)
		
		local _, lighty, _ = Spring.GetUnitPiecePosDir(unitID, lightpitch)
		local dx, dy, dz = x - ux, lighty - y, z - uz
		local dist = math.sqrt(dx * dx + dz * dz)
		local pitch = math.atan2(dy, dist)
		Turn(lightpitch, x_axis, pitch + math.pi)
		
		Spring.SetUnitRulesParam(unitID, "lighthouse_x", x)
		Spring.SetUnitRulesParam(unitID, "lighthouse_z", z)	
		
		if torchScaryArea then
			torchScaryArea.x = x
			torchScaryArea.z = z
		end
		
		Sleep(33)
		
		if not torchScaryArea then
			torchScaryArea = GG.AddScaryArea({x = x, z = x, attributes = torchAttributes})
		end
	end
end

function script.Create()
	lightAngle = math.random()*2*math.pi
	ux,_,uz = Spring.GetUnitPosition(unitID)
	--Turn(lightpitch, x_axis, math.pi*0.5)
	Turn(lighthead, y_axis, -lightAngle + math.pi/2)
	local x,z = GetLightCoordinates()
	
	if GG.AddScaryArea then
		torchScaryArea = GG.AddScaryArea({x = x, z = x, attributes = torchAttributes})
	end
	
	Spring.SetUnitRulesParam(unitID, "lighthouse_x", x)
	Spring.SetUnitRulesParam(unitID, "lighthouse_z", z)
	Spring.SetUnitRulesParam(unitID, "lighthouse_size", lightRadius)
	Spring.SetUnitRulesParam(unitID, "lighthouse_color", 1)
	
	StartThread(LightSpin)
end

function script.Killed(recentDamage, maxHealth)
	if torchScaryArea then
		GG.RemoveScaryArea(torchScaryArea)
	end
	return 0
end