--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

function gadget:GetInfo()
   return {
      name      = "Field Spawner",
      desc      = "Spawns fields of carrots and supporting structures.",
      author    = "Google Frog",
      date      = "9 August 2015",
      license   = "GNU GPL, v2 or later",
      layer     = 15,
      enabled   = true
   }
end

--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

--SYNCED
if (not gadgetHandler:IsSyncedCode()) then
   return false
end

--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

local carrotDefID = UnitDefNames["carrot"].id
local dropDefID = UnitDefNames["carrot_dropped"].id

-- Fields are manually placed groups of carrots. They need to span the whole
-- map to attract rabbits.
local desirableFieldAttributes = {
	radius = 9000,
	radiusSq = 9000^2,
	edgeMagnitude = 0, -- Magnitude once within radius
	proximityMagnitude = 0.1, -- Maximum agnitude gained by being close
	thingType = 1, -- Food
}

--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

local function SpawnField(topX, topZ, botX, botZ, rows, cols)
	
	local colSpace = (botX - topX)/math.max(1, cols - 1) 
	local rowSpace = (botZ - topZ)/math.max(1, rows - 1) 
	
	local midX = (topX + botX)/2
	local midZ = (topZ + botZ)/2
	
	GG.AddDesirableArea({x = midX, z = midZ, attributes = desirableFieldAttributes})
	
	local carrotCount = Spring.GetGameRulesParam("carrot_count") or 0
	Spring.SetGameRulesParam("carrot_count", carrotCount + rows*cols)

	local x, z = topX, topZ
	for i = 1, rows do
		for j = 1, cols do
			local carrotID = Spring.CreateUnit(carrotDefID, x, 0, z, 0, 0, false, false)
			Spring.SetUnitRotation(carrotID, 0, math.random()*2*math.pi, 0)
			x = x + colSpace
		end
		z = z + rowSpace
		x = topX
	end
end

function gadget:UnitDestroyed(unitID, unitDefID)
	if unitDefID ~= carrotDefID and unitDefID ~= dropDefID then
		return
	end
	if Spring.GetUnitRulesParam(unitID, "internalDestroy") then
		return
	end
	
	if not Spring.GetUnitRulesParam(unitID, "carrotScored") then
		local destroyed = Spring.GetGameRulesParam("carrots_destroyed")
		Spring.SetGameRulesParam("carrots_destroyed", destroyed + 1)
	end
	
	local carrotCount = Spring.GetGameRulesParam("carrot_count") or 0
	Spring.SetGameRulesParam("carrot_count", carrotCount - 1)
end

function gadget:Initialize()
	-- Clean up carried carrots as rabbits will not remember that they exist.
	for _, unitID in ipairs(Spring.GetAllUnits()) do
		local unitDefID = Spring.GetUnitDefID(unitID)
		if unitDefID == dropDefID or unitDefID == carrotDefID then
			Spring.SetUnitRulesParam(unitID, "internalDestroy", 1)
			Spring.DestroyUnit(unitID, false, false)
		end
	end
	
	Spring.SetGameRulesParam("carrots_destroyed", 0)
	Spring.SetGameRulesParam("carrots_stolen", 0)
	Spring.SetGameRulesParam("carrot_count", 0)
	GG.SpawnField = SpawnField
end